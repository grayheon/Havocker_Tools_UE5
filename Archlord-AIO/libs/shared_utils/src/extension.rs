pub enum SkipMode {
    Ignore,
    CopyOnly,
    Decrypt,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FileExtension {
    Ini,
    Txt,
    Xml,
    Txd,
    Pk,
    Dds,
    Wav,
    Dat,
    Mp3,
    Bm1,
    Tx1,
    Ecl,
    Bmp,
    Png,
    Jpg,
    Tif,
    Ma1,
    Ma2,
    Other(String),
}

impl FileExtension {
    pub fn from_str(ext: &str) -> Self {
        let lower = ext.to_lowercase();
        match lower.as_str() {
            "ini" => FileExtension::Ini,
            "txt" => FileExtension::Txt,
            "xml" => FileExtension::Xml,
            "txd" => FileExtension::Txd,
            "dat" => FileExtension::Dat,
            "pk" => FileExtension::Pk,
            "dds" => FileExtension::Dds,
            "wav" => FileExtension::Wav,
            "mp3" => FileExtension::Mp3,
            "bm1" => FileExtension::Bm1,
            "tx1" => FileExtension::Tx1,
            "ecl" => FileExtension::Ecl,
            "bmp" => FileExtension::Bmp,
            "tif" => FileExtension::Tif,
            "ma1" => FileExtension::Ma1,
            "ma2" => FileExtension::Ma2,
            _ => FileExtension::Other(lower),
        }
    }

    /// Gibt zurück, ob diese Endung entschlüsselt werden soll (wenn erlaubt)
    pub fn should_decrypt(&self) -> bool {
        matches!(
            self,
            FileExtension::Ini | FileExtension::Txt | FileExtension::Xml
        )
    }

    /// Rückgabe der ggf. umgewandelten Ziel-Endung
    pub fn mapped(&self) -> &str {
        match self {
            FileExtension::Bm1 => "bmp",
            FileExtension::Pk => "wav",
            _ => self.as_str(),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            FileExtension::Ini => "ini",
            FileExtension::Txt => "txt",
            FileExtension::Xml => "xml",
            FileExtension::Txd => "txd",
            FileExtension::Dat => "dat",
            FileExtension::Pk => "pk",
            FileExtension::Dds => "dds",
            FileExtension::Wav => "wav",
            FileExtension::Mp3 => "mp3",
            FileExtension::Bm1 => "bm1",
            FileExtension::Tx1 => "tx1",
            FileExtension::Ecl => "ecl",
            FileExtension::Bmp => "bmp",
            FileExtension::Png => "png",
            FileExtension::Jpg => "jpg",
            FileExtension::Tif => "tif",
            FileExtension::Ma1 => "ma1",
            FileExtension::Ma2 => "ma2",
            FileExtension::Other(ext) => ext,
        }
    }

    pub fn is_relevant(&self) -> bool {
        matches!(
            self,
            FileExtension::Ini
                | FileExtension::Txt
                | FileExtension::Xml
                | FileExtension::Wav
                | FileExtension::Mp3
                | FileExtension::Bmp
                | FileExtension::Png
                | FileExtension::Jpg
                | FileExtension::Pk
                | FileExtension::Tif
                | FileExtension::Dds
                | FileExtension::Dat
                | FileExtension::Ma1
                | FileExtension::Ma2
        )
    }
}
