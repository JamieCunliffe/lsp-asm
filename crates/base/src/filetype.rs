use once_cell::sync::Lazy;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FileType {
    Assembly,
    ObjDump(ObjDumpOptions),
}

impl Default for FileType {
    fn default() -> Self {
        Self::Assembly
    }
}

impl FileType {
    pub fn from_contents(contents: &str) -> Self {
        use regex::Regex;

        let filedata = &contents[..contents
            .match_indices('\n')
            .nth(5)
            .map(|(idx, _)| idx)
            .unwrap_or(contents.len())];

        static OBJDUMP_DETECTION: Lazy<Vec<Regex>> = Lazy::new(|| {
            vec![
                Regex::new(r#".*:[\t ]+file format .*\n\n\nDisassembly of section .*:"#).unwrap(),
                Regex::new(r#"\n.*:\tfile format .*\n\nDisassembly of section .*:"#).unwrap(),
            ]
        });

        OBJDUMP_DETECTION
            .iter()
            .find_map(|regex| {
                regex
                    .is_match(filedata)
                    .then(|| FileType::ObjDump(ObjDumpOptions::from_contents(contents)))
            })
            .unwrap_or_default()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ObjDumpOptions {
    /// false if the objdump was created with --no-show-raw-insn
    pub show_raw_insn: bool,
}

impl Default for ObjDumpOptions {
    fn default() -> Self {
        Self {
            show_raw_insn: true,
        }
    }
}

impl ObjDumpOptions {
    pub fn from_contents(contents: &str) -> Self {
        use regex::RegexBuilder;

        let sample_data = &contents[..contents
            .match_indices('\n')
            .nth(20)
            .map(|(idx, _)| idx)
            .unwrap_or(contents.len())];

        macro_rules! regex_detect {
            ($regex:literal) => {{
                static REGEX: Lazy<regex::Regex> =
                    Lazy::new(|| RegexBuilder::new($regex).multi_line(true).build().unwrap());
                !REGEX.is_match(sample_data)
            }};
        }

        let show_raw_insn = regex_detect!(r#"^([0-9a-fA-F]*:)? *\t"#);

        Self { show_raw_insn }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::ObjDumpOptions;

    #[test]
    fn default_objdump_options() {
        let input = r#"
a.out:     file format elf64-x86-64


Disassembly of section .text:

00000000002015a0 <_start>:
  2015a0:	f3 0f 1e fa          	endbr64
  2015a4:	31 ed                	xor    %ebp,%ebp"#;
        assert_eq!(ObjDumpOptions::from_contents(input), Default::default());
    }

    #[test]
    fn no_show_raw_insn() {
        let input = r#"
target/release/test:	file format mach-o arm64

Disassembly of section __TEXT,__text:

000000010002ad84 <__ZN4core3ptr100drop_in_place$LT$core..option..Option$LT$lsp_types..completion..CompletionClientCapabilities$GT$$GT$17h599609cf084734b7E>:
10002ad84:     	ldr	x8, [x0]
"#;
        assert_eq!(
            ObjDumpOptions::from_contents(input),
            ObjDumpOptions {
                show_raw_insn: false
            }
        );
    }
}
