use std::io::{self};
use std::ptr::null;
use winapi::shared::minwindef::DWORD;
use winapi::um::wincrypt::{
    CryptAcquireContextA, CryptCreateHash, CryptDecrypt, CryptDeriveKey, CryptDestroyHash,
    CryptDestroyKey, CryptHashData, CryptReleaseContext,
    CALG_MD5, CALG_RC4, CRYPT_CREATE_SALT,
    HCRYPTHASH, HCRYPTKEY, HCRYPTPROV, PROV_RSA_FULL,
};

/// Entschlüsselt die Daten per WinAPI mit MD5 + RC4.
///
/// `use_texture_key = true` → Key `"asdfqwer"` (Texturen)
/// `use_texture_key = false` → Key `"1111"` (Default für normale Dateien)
pub fn decrypt_data(data: &mut [u8], use_texture_key: bool) -> io::Result<usize> {
    unsafe {
        let mut h_provider: HCRYPTPROV = 0;
        let mut h_hash: HCRYPTHASH = 0;
        let mut h_key: HCRYPTKEY = 0;

        let provider_name = b"Microsoft Base Cryptographic Provider v1.0\0";

        if CryptAcquireContextA(
            &mut h_provider,
            null::<i8>(),
            provider_name.as_ptr() as *const i8,
            PROV_RSA_FULL,
            0xF0000000,
        ) == 0 {
            return Err(io::Error::last_os_error());
        }

        if CryptCreateHash(h_provider, CALG_MD5, 0, 0, &mut h_hash) == 0 {
            CryptReleaseContext(h_provider, 0);
            return Err(io::Error::last_os_error());
        }

        let key_data: &[u8] = if use_texture_key {
            b"asdfqwer"
        } else {
            b"1111"
        };

        if CryptHashData(h_hash, key_data.as_ptr(), key_data.len() as DWORD, 0) == 0 {
            CryptDestroyHash(h_hash);
            CryptReleaseContext(h_provider, 0);
            return Err(io::Error::last_os_error());
        }

        if CryptDeriveKey(h_provider, CALG_RC4, h_hash, CRYPT_CREATE_SALT, &mut h_key) == 0 {
            CryptDestroyHash(h_hash);
            CryptReleaseContext(h_provider, 0);
            return Err(io::Error::last_os_error());
        }

        CryptDestroyHash(h_hash);

        let mut data_len: DWORD = data.len() as DWORD;
        if CryptDecrypt(h_key, 0, 1, 0, data.as_mut_ptr(), &mut data_len) == 0 {
            CryptDestroyKey(h_key);
            CryptReleaseContext(h_provider, 0);
            return Err(io::Error::last_os_error());
        }

        CryptDestroyKey(h_key);
        CryptReleaseContext(h_provider, 0);

        Ok(data_len as usize)
    }
}
