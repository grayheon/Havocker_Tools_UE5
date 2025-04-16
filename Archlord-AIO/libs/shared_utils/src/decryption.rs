use md5::{Digest, Md5};
use rc4::{KeyInit, Rc4, StreamCipher};
use rc4::consts::U16;

#[derive(Debug, Clone, Copy)]
pub enum DecryptKey {
    Default,
    Texture,
}

impl DecryptKey {
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            DecryptKey::Default => b"1111",
            DecryptKey::Texture => b"asdfqwer",
        }
    }
}

pub fn decrypt_data_pure(buffer: &mut [u8], key: DecryptKey) -> usize {
    let key_hash = Md5::digest(key.as_bytes());
    let mut cipher: Rc4<U16> = Rc4::new_from_slice(&key_hash).expect("Invalid RC4 key");
    cipher.apply_keystream(buffer);
    buffer.len()
}