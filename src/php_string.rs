#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhpString {
    bytes: Vec<u8>,
}

impl PhpString {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn from_text(value: &str) -> Self {
        Self::new(value.as_bytes().to_vec())
    }

    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl From<&str> for PhpString {
    fn from(value: &str) -> Self {
        Self::from_text(value)
    }
}

impl From<String> for PhpString {
    fn from(value: String) -> Self {
        Self::new(value.into_bytes())
    }
}

impl PartialEq<&str> for PhpString {
    fn eq(&self, other: &&str) -> bool {
        self.bytes == other.as_bytes()
    }
}

impl PartialEq<str> for PhpString {
    fn eq(&self, other: &str) -> bool {
        self.bytes == other.as_bytes()
    }
}
