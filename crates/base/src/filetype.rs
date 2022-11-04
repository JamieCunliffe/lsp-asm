use once_cell::sync::Lazy;

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

        let filedata = &filedata[..filedata
            .match_indices('\n')
            .nth(5)
            .map(|(idx, _)| idx)
            .unwrap_or(filedata.len())];

        static OBJDUMP_DETECTION: Lazy<Vec<Regex>> = Lazy::new(|| {
            vec![
                Regex::new(r#".*:[\t ]+file format .*\n\n\nDisassembly of section .*:"#).unwrap(),
                Regex::new(r#"\n.*:\tfile format .*\n\nDisassembly of section .*:"#).unwrap(),
            ]
        });

        OBJDUMP_DETECTION
            .iter()
            .find_map(|regex| regex.is_match(filedata).then_some(FileType::ObjDump))
            .unwrap_or_default()
    }
}
