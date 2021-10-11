use itertools::Itertools;
use std::error::Error;
use std::ffi::OsStr;
use std::io::prelude::*;
use textwrap::fill;

use documentation::registers::to_documentation_name;
use documentation::{Instruction, InstructionTemplate, OperandInfo};
use log::warn;

const A64_ISA: &str = "https://developer.arm.com/-/media/developer/products/architecture/armv8-a-architecture/2021-06/A64_ISA_xml_v87A-2021-06.tar.gz";
const A64_ISA_DIR: &str = "ISA_A64_xml_v87A-2021-06";

fn process_isa_ref(data: &str, file: &str) -> Vec<Instruction> {
    let doc = roxmltree::Document::parse(data).unwrap();
    let instruction_section = match doc
        .descendants()
        .find(|d| d.tag_name().name() == "instructionsection")
    {
        Some(i) => i,
        None => {
            warn!("No instruction data found: {:?}", file);
            return Vec::new();
        }
    };

    let mnemonic: Vec<&str> = instruction_section
        .descendants()
        .filter(|d| d.tag_name().name() == "docvar" && d.attribute("key").unwrap() == "mnemonic")
        .filter_map(|d| d.attribute("value"))
        .unique()
        .collect();

    let variant = instruction_section
        .descendants()
        .filter(|d| d.tag_name().name() == "arch_variant")
        .find_map(|d| d.attribute("name"))
        .map(String::from);

    let authored = match instruction_section
        .descendants()
        .find(|d| d.tag_name().name() == "authored" || d.tag_name().name() == "description")
    {
        Some(a) => a,
        None => {
            warn!("No description data found: {:?}", file);
            return Vec::new();
        }
    };

    let header = instruction_section
        .descendants()
        .find(|d| d.tag_name().name() == "heading")
        .map(|x| {
            x.descendants()
                .filter_map(|d| (!d.is_element()).then(|| d.text().unwrap_or("").to_string()))
                .collect()
        });

    let description: String = authored
        .descendants()
        .filter_map(|d| {
            if !d.is_element() {
                Some(d.text().unwrap_or("").to_string())
            } else {
                match d.tag_name().name() {
                    "para" | "list" => Some(String::from("\n")),
                    "listitem" => Some(String::from("\n* ")),
                    _ => None,
                }
            }
        })
        .collect::<String>()
        .lines()
        .map(|x| x.trim_start())
        .join("\n");

    let asm_template = instruction_section
        .descendants()
        .filter(|d| d.tag_name().name() == "asmtemplate")
        .map(|x| x.children());

    let asm = asm_template
        .clone()
        .map(|x| x.map(|x| x.text().unwrap_or("").to_string()).join(""));

    let operands = asm_template.map(|x| {
        x.filter(|c| c.tag_name().name() == "a")
            .map(|n| {
                let description = n
                    .attribute_node("hover")
                    .map(|a| a.value())
                    .unwrap_or("")
                    .to_string();

                let completion_values = get_completion_values(&description);

                OperandInfo {
                    name: n.text().unwrap_or("").to_string(),
                    description,
                    completion_values,
                }
            })
            .collect_vec()
    });

    let asm_template = asm
        .zip(operands)
        .map(|(asm, items)| InstructionTemplate {
            asm: parse_template(&asm, &items)
                .drain(..)
                .map(|mut t| {
                    crate::register_replacements::REGISTER_REPLACEMENTS
                        .iter()
                        .for_each(|(f, s, k)| t = t.replace(f, &to_documentation_name(k, s)));
                    t
                })
                .collect_vec(),
            display_asm: asm.clone(),
            items,
        })
        .collect::<Vec<_>>();

    mnemonic
        .iter()
        .map(|mnemonic| Instruction {
            opcode: mnemonic.to_string(),
            header: header.clone(),
            architecture: variant.clone(),
            description: fill(description.trim(), 100),
            asm_template: asm_template.clone(),
        })
        .collect()
}

fn parse_template(template: &str, operands: &[OperandInfo]) -> Vec<String> {
    let mut positions = Vec::new();
    for (i, c) in template.char_indices() {
        if c == '{' {
            positions.push((i, 0));
        } else if c == '}' {
            let (_, ref mut e) = positions.iter_mut().rev().find(|(_, e)| *e == 0).unwrap();
            *e = i
        }
    }
    let positions = positions.iter().filter(|(open, close)| {
        let text = &template[(*open + 1)..*close];
        if text.starts_with(',') {
            return true;
        }
        let start = text.chars().take_while(|c| c != &'<').count();
        let text = &text[start..];
        let text = text
            .find(|c| c == ' ' || c == ',')
            .map(|p| &text[..p])
            .unwrap_or(text)
            .trim_end_matches(|c| c == '{' || c == '}');

        let optional = operands
            .iter()
            .find(|op| op.name == text)
            .map(|op| {
                assert!(!op.name.starts_with('#'));
                op.description.to_lowercase().contains("optional")
                    || op.description.to_lowercase().contains("default")
            })
            .unwrap_or(false);

        optional
    });

    let mut full = template.to_string();
    positions
        .clone()
        .flat_map(|(s, e)| vec![s, e])
        .sorted()
        .rev()
        .for_each(|idx| full.replace_range(*idx..(idx + 1), ""));
    let mut ret = positions
        .enumerate()
        .map(|(idx, (open, close))| {
            full.chars()
                .take(open - idx)
                .chain(template.chars().skip(close + 1 + idx))
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect_vec();
    ret.push(full);
    ret.reverse();
    ret
}

fn get_completion_values(description: &str) -> Option<Vec<String>> {
    let start = description.find('[')?;
    let process_item = |part: &str| -> Option<Vec<String>> {
        if let Some(stripped) = part.strip_prefix('#') {
            // If we start with a # only allow it if the part following it is numeric
            stripped.parse::<i64>().ok()?;
            Some(vec![part.to_string()])
        } else if part.contains('-') {
            crate::util::range_to_list(part)
        } else {
            Some(vec![part.to_string()])
        }
    };

    if let Some(end) = description.find(']') {
        let items = &description[(start + 1)..end];
        Some(
            items
                .split(',')
                .filter_map(process_item)
                .flatten()
                .collect::<Vec<_>>(),
        )
    } else {
        warn!("Description: {} doesn't have a closing ]", description);
        None
    }
}

pub(crate) async fn get_instructions() -> Result<Vec<Instruction>, Box<dyn Error>> {
    println!("Downloading XML reference from {}", A64_ISA);
    let isa_data = reqwest::get(A64_ISA).await?.bytes().await?.to_vec();

    println!("Processing arm instruction set reference");
    let mut a = tar::Archive::new(flate2::read::GzDecoder::new(isa_data.as_slice()));

    Ok(a.entries()?
        .filter_map(|file| file.ok())
        .filter(|file| {
            let path = file.header().path().unwrap();
            path.extension() == Some(OsStr::new("xml")) && path.starts_with(A64_ISA_DIR)
        })
        .flat_map(|mut file| {
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();
            process_isa_ref(
                &s,
                file.path().unwrap().file_name().unwrap().to_str().unwrap(),
            )
        })
        .collect::<Vec<_>>())
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_template_expansion() {
        let operands = vec![
            OperandInfo {
                name: "<Xt1>".into(),
                description: "Register".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<Xt2>".into(),
                description: "Another register".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<Xn|SP>".into(),
                description: "Another register".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<imm>".into(),
                description: "Optional immediate".into(),
                completion_values: Default::default(),
            },
        ];
        let input = "STP  <Xt1>, <Xt2>, [<Xn|SP>{, #<imm>}]";
        let result = parse_template(input, &operands);
        assert_eq!(
            result,
            vec![
                String::from("STP  <Xt1>, <Xt2>, [<Xn|SP>, #<imm>]"),
                String::from("STP  <Xt1>, <Xt2>, [<Xn|SP>]")
            ]
        );
    }

    #[test]
    fn test_parse_template_expansion_multiple() {
        let operands = vec![
            OperandInfo {
                name: "<Xd>".into(),
                description: "Register".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<Xn|SP>".into(),
                description: "Another register".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<R><m>".into(),
                description: "Something".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<extend>".into(),
                description: "Optional extend".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<amount>".into(),
                description: "Optional immediate".into(),
                completion_values: Default::default(),
            },
        ];
        let input = "ADDS  <Xd>, <Xn|SP>, <R><m>{, <extend> {#<amount>}}";
        let result = parse_template(input, &operands);

        assert_eq!(
            result,
            vec![
                String::from("ADDS  <Xd>, <Xn|SP>, <R><m>, <extend> #<amount>"),
                String::from("ADDS  <Xd>, <Xn|SP>, <R><m>, <extend>"),
                String::from("ADDS  <Xd>, <Xn|SP>, <R><m>"),
            ]
        );
    }

    #[test]
    fn test_parse_template_expansion_without_optional() {
        let operands = vec![
            OperandInfo {
                name: "<Zt>".into(),
                description: "Register".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<Pg>".into(),
                description: "Predicate".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<Zn>".into(),
                description: "Another register".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<imm>".into(),
                description: "Optional immediate".into(),
                completion_values: Default::default(),
            },
        ];
        let input = "LD1W    { <Zt>.S }, <Pg>/Z, [<Zn>.S{, #<imm>}]";
        let result = parse_template(input, &operands);

        assert_eq!(
            result,
            vec![
                String::from("LD1W    { <Zt>.S }, <Pg>/Z, [<Zn>.S, #<imm>]"),
                String::from("LD1W    { <Zt>.S }, <Pg>/Z, [<Zn>.S]"),
            ]
        );
    }

    #[test]
    fn test_parse_template_expansion_extra_in_opt() {
        let operands = vec![
            OperandInfo {
                name: "<Zt>".into(),
                description: "Register".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<Pg>".into(),
                description: "Predicate".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<Xn|SP>".into(),
                description: "Another register".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<imm>".into(),
                description: "Optional immediate".into(),
                completion_values: Default::default(),
            },
        ];
        let input = "LD1W    { <Zt>.S }, <Pg>/Z, [<Xn|SP>{, #<imm>, MUL VL}]";
        let result = parse_template(input, &operands);

        assert_eq!(
            result,
            vec![
                String::from("LD1W    { <Zt>.S }, <Pg>/Z, [<Xn|SP>, #<imm>, MUL VL]"),
                String::from("LD1W    { <Zt>.S }, <Pg>/Z, [<Xn|SP>]"),
            ]
        );
    }

    #[test]
    fn test_parse_template_expansion_with_default() {
        let operands = vec![
            OperandInfo {
                name: "<Xdn>".into(),
                description: "general-purpose register".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<pattern>".into(),
                description: "Optional pattern specifier".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<imm>".into(),
                description: "multiplier, default 1 ".into(),
                completion_values: Default::default(),
            },
        ];
        let input = "INCB    <Xdn>{, <pattern>{, MUL #<imm>}}";
        let result = parse_template(input, &operands);

        assert_eq!(
            result,
            vec![
                String::from("INCB    <Xdn>, <pattern>, MUL #<imm>"),
                String::from("INCB    <Xdn>, <pattern>"),
                String::from("INCB    <Xdn>"),
            ]
        );
    }
}
