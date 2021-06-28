pub type LineNumber = u32;
pub type ColumnNumber = u32;

#[derive(Debug, PartialEq, Clone)]
pub struct DocumentPosition {
    pub line: LineNumber,
    pub column: ColumnNumber,
}

#[derive(Debug, PartialEq)]
pub struct DocumentRange {
    pub start: DocumentPosition,
    pub end: DocumentPosition,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Architecture {
    X86_64,
    AArch64,
    Unknown,
}

impl From<&str> for Architecture {
    /// Converts known text based names for architectures into the `Architecture` enum variant for it.
    fn from(arch: &str) -> Self {
        debug!("Architecture::from: {:?}", arch);
        match arch.to_lowercase().as_str() {
            "x86_64" | "x86-64" => Architecture::X86_64,
            "aarch64" | "littleaarch64" | "armv8-a" | "arm64" => Architecture::AArch64,
            _ => Architecture::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architectures() {
        assert_eq!(Architecture::from("armv8-a"), Architecture::AArch64);
        assert_eq!(Architecture::from("aarch64"), Architecture::AArch64);
        assert_eq!(Architecture::from("littleaarch64"), Architecture::AArch64);
        assert_eq!(Architecture::from("arm64"), Architecture::AArch64);
        assert_eq!(Architecture::from("x86_64"), Architecture::X86_64);
    }
}
