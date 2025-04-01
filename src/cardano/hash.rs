use cbored::CborRepr;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, CborRepr)]
pub struct Hash([u8; 32]);

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Hash")
            .field(&hex::encode(self.as_ref()))
            .finish()
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.as_ref()))
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
