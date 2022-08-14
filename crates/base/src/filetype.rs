#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FileType {
    Assembly,
    ObjDump,
}

impl Default for FileType {
    fn default() -> Self {
        Self::Assembly
    }
}

impl FileType {
    pub fn from_contents(filedata: &str) -> Self {
        use regex::Regex;

        lazy_static! {
            static ref OBJDUMP_DETECTION: Regex =
                Regex::new(r#".*:[\t ]+file format .*\n\n\nDisassembly of section .*:"#).unwrap();
        }

        if OBJDUMP_DETECTION.is_match(filedata) {
            FileType::ObjDump
        } else {
            Default::default()
        }
    }
}
