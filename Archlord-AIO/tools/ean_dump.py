#!/usr/bin/env python3
"""
Quick-and-dirty EAN inspector.

Goal (step 1–3):
- Dump header words (u32/f32) for basic format forensics.
- Heuristically guess a table section using header[3] (header size?) and header[5]
  (often an offset) to derive entry size and emit a per-entry dump.
- Emit sample float blocks for candidate data regions.

Usage:
    python tools/ean_dump.py path1.ean [path2.ean ...]

Outputs JSON to stdout with one object per file.
"""
from __future__ import annotations

import argparse
import math
import json
import struct
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple


def read_u32(data: bytes, offset: int) -> int:
    return struct.unpack_from("<I", data, offset)[0]


def read_f32(data: bytes, offset: int) -> float:
    return struct.unpack_from("<f", data, offset)[0]


def chunk_words(data: bytes, offset: int, count: int) -> List[int]:
    return [read_u32(data, offset + i * 4) for i in range(count)]


def chunk_floats(data: bytes, offset: int, count: int) -> List[float]:
    return [read_f32(data, offset + i * 4) for i in range(count)]


def analyze_file(path: Path, sample_floats: int = 32) -> Dict[str, Any]:
    data = path.read_bytes()
    size = len(data)

    header_u32 = chunk_words(data, 0, min(16, size // 4))
    header_f32 = chunk_floats(data, 0, min(16, size // 4))

    count = header_u32[0] if header_u32 else 0
    header_size_guess = header_u32[3] if len(header_u32) > 3 and 0 < header_u32[3] < size else None
    table_offset_guess = header_u32[5] if len(header_u32) > 5 and 0 < header_u32[5] < size else None

    entry_size_guess: Optional[int] = None
    if (
        count > 0
        and header_size_guess is not None
        and table_offset_guess is not None
        and table_offset_guess > header_size_guess
    ):
        span = table_offset_guess - header_size_guess
        if span % count == 0:
            entry_size_guess = span // count

    table_entries: List[Dict[str, Any]] = []
    if entry_size_guess and entry_size_guess > 0 and header_size_guess is not None:
        words_per = entry_size_guess // 4
        for i in range(count):
            off = header_size_guess + i * entry_size_guess
            if off + entry_size_guess > size:
                break
            entry_bytes = data[off : off + entry_size_guess]
            table_entries.append(
                {
                    "index": i,
                    "offset": off,
                    "u32": list(struct.unpack("<" + "I" * words_per, entry_bytes)),
                    "f32": list(struct.unpack("<" + "f" * words_per, entry_bytes)),
                }
            )

    sample_blocks: List[Dict[str, Any]] = []
    for off in {256, header_size_guess, table_offset_guess}:
        if off is None:
            continue
        if not (0 <= off < size):
            continue
        count_f = min(sample_floats, (size - off) // 4)
        sample_blocks.append(
            {
                "offset": off,
                "f32": chunk_floats(data, off, count_f),
                "u32": chunk_words(data, off, count_f),
            }
        )

    # Collect small u32 that look like offsets (multi-of-4 and < size)
    offsets = []
    for i in range(0, size - 3, 4):
        v = read_u32(data, i)
        if v < size and v % 4 == 0:
            offsets.append(v)
    unique_offsets = sorted(set(offsets))

    # Heuristic: gcd of offset deltas as potential stride
    stride_gcd: Optional[int] = None
    if len(unique_offsets) > 2:
        deltas = [b - a for a, b in zip(unique_offsets, unique_offsets[1:]) if b > a]
        stride_gcd = deltas[0]
        for d in deltas[1:]:
            stride_gcd = math.gcd(stride_gcd, d)
        if stride_gcd == 0:
            stride_gcd = None

    # Heuristic: split float stream by sentinel (very large negative)
    data_start = header_size_guess or 0
    groups: List[Dict[str, Any]] = []
    current_f: List[float] = []
    current_start = 0
    sentinel_u32 = 0xE6719FE0  # observed separator
    u32_stream = [read_u32(data, data_start + i) for i in range(0, size - data_start, 4)]
    for idx, raw in enumerate(u32_stream):
        if raw == sentinel_u32:
            if current_f:
                groups.append({"start": data_start + current_start * 4, "count": len(current_f), "values": current_f})
            current_f = []
            current_start = idx + 1
        else:
            if not current_f:
                current_start = idx
            current_f.append(struct.unpack("<f", struct.pack("<I", raw))[0])
    if current_f:
        groups.append({"start": data_start + current_start * 4, "count": len(current_f), "values": current_f})

    groups_preview = [
        {
            "start": g["start"],
            "count": g["count"],
            "values": g["values"][: min(len(g["values"]), 16)],
        }
        for g in groups[: min(len(groups), 8)]
    ]

    # Histogram of group lengths (keyframe vector lengths)
    from collections import Counter

    group_len_hist = Counter(g["count"] for g in groups)

    # Build tracks from dense frame-major block of 8 floats (quat+pos+extra)
    fps_guess = header_f32[7] if len(header_f32) > 7 and math.isfinite(header_f32[7]) else 30.0
    dt = 1.0 / fps_guess if fps_guess else 1.0 / 30.0
    frames_est = 0
    if count and table_offset_guess:
        frames_est = round((size - table_offset_guess) / (count * 32))
    data_block_start = size - frames_est * count * 32 if frames_est else data_start
    blocks = []
    for off in range(data_block_start, size, 32):
        if off + 32 > size:
            break
        vals = chunk_floats(data, off, 8)
        blocks.append(vals)
    frames = len(blocks) // count if count else 0

    tracks: List[Dict[str, Any]] = [{"joint": j, "keyframes": []} for j in range(count)]
    for f in range(frames):
        for j in range(count):
            idx = f * count + j
            vals = blocks[idx]
            rot = vals[0:4]
            pos = vals[4:7]
            extra = vals[7]
            tracks[j]["keyframes"].append({"time": f * dt, "rot": rot, "pos": pos, "extra": extra})

    return {
        "file": path.name,
        "size": size,
        "header": {
            "u32": header_u32,
            "f32": header_f32,
        },
        "guesses": {
            "count": count,
            "header_size": header_size_guess,
            "table_offset": table_offset_guess,
            "entry_size": entry_size_guess,
        },
        "table_entries": table_entries,
        "samples": sample_blocks,
        "offsets_first_64": unique_offsets[:64],
        "offset_stride_gcd": stride_gcd,
        "groups_preview": groups_preview,
        "group_len_hist": group_len_hist,
        "frames": frames,
        "data_block_start": data_block_start,
        "tracks": tracks,
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Heuristic EAN inspector -> JSON.")
    ap.add_argument("files", nargs="+", type=Path, help="EAN files to inspect")
    ap.add_argument("--sample-floats", type=int, default=32, help="Floats to dump per sample block")
    args = ap.parse_args()

    reports = [analyze_file(p, sample_floats=args.sample_floats) for p in args.files]
    json.dump(reports, fp=sys.stdout, indent=2)


if __name__ == "__main__":
    import sys

    main()
