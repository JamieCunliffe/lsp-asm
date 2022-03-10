use itertools::Itertools;
use log::{debug, warn};
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::io::prelude::*;
use textwrap::fill;

use documentation::registers::to_documentation_name;
use documentation::{
    CompletionValue, Instruction, InstructionTemplate, OperandAccessType, OperandInfo,
};

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

    let mut mnemonic: Vec<String> = instruction_section
        .descendants()
        .filter(|d| d.tag_name().name() == "docvar" && d.attribute("key").unwrap() == "mnemonic")
        .filter_map(|d| d.attribute("value"))
        .unique()
        .map(String::from)
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
        .filter(|d| d.tag_name().name() == "asmtemplate");

    let asm = asm_template.clone().map(|x| {
        let asm = x
            .children()
            .map(|x| x.text().unwrap_or("").to_string())
            .join("");
        let doc_vars = x
            .parent()
            .unwrap()
            .descendants()
            .filter_map(|d| {
                (d.tag_name().name() == "docvar")
                    .then(|| (d.attribute("key").unwrap(), d.attribute("value").unwrap()))
            })
            .into_group_map();
        (asm, doc_vars)
    });

    let operands = asm_template.map(|x| {
        x.children()
            .filter(|c| c.tag_name().name() == "a")
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
        .map(|((asm, vars), items)| {
            let templates = parse_template(&asm, &items)
                .drain(..)
                .map(|mut t| {
                    crate::register_replacements::REGISTER_REPLACEMENTS
                        .iter()
                        .for_each(|(f, s, k)| t = t.replace(f, &to_documentation_name(k, s)));
                    t
                })
                .collect_vec();
            let access_map = build_access_map(&asm, &items, &vars);
            InstructionTemplate {
                asm: templates,
                display_asm: asm,
                items,
                access_map,
            }
        })
        .collect::<Vec<_>>();

    if let Some(asm) = asm_template
        .iter()
        .find_map(|temp| temp.asm.iter().find(|asm| asm.contains("{2}")))
    {
        let asm = asm[..asm.find(' ').unwrap()].replace("{2}", "2");
        mnemonic.push(asm);
    }

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

fn extract_brackets(template: &str) -> Vec<&str> {
    let mut template = template;
    let mut ret = Vec::with_capacity(3);

    while let Some(start) = template.find('(') {
        let end = template.find(')').unwrap() + 1;
        let brackets = &template[start..end];
        ret.push(brackets);

        template = &template[end..];
    }

    ret
}

fn expand_template(template: &str, expansions: &[(&str, &Vec<&str>)]) -> Vec<String> {
    if let Some(((find, replace), remaining)) = expansions.split_last() {
        replace
            .iter()
            .flat_map(|replace| expand_template(&template.replace(find, replace), remaining))
            .collect_vec()
    } else {
        vec![template.to_string()]
    }
}

fn build_access_map(
    asm: &str,
    operands: &[OperandInfo],
    vars: &HashMap<&str, Vec<&str>>,
) -> Vec<OperandAccessType> {
    let start = if let Some(start) = asm.trim().find(' ') {
        start
    } else {
        return Default::default();
    };

    let parts = asm[start..].split(',');
    parts
        .map(|operand| {
            let operand = operand.trim();
            if operand.starts_with('#') || operand.starts_with("<label>") {
                OperandAccessType::Text
            } else {
                let desc = operands
                    .iter()
                    .filter(|op| operand.contains(&op.name))
                    .map(|x| &x.description)
                    .join(" ");

                access_type(asm, &desc, vars)
            }
        })
        .collect()
}

fn access_type(asm: &str, desc: &str, vars: &HashMap<&str, Vec<&str>>) -> OperandAccessType {
    let access = [
        // Write access:
        ("destination register", OperandAccessType::Write),
        (
            "destination general-purpose register",
            OperandAccessType::Write,
        ),
        (
            "destination scalable vector register",
            OperandAccessType::Write,
        ),
        (
            "destination scalable predicate register",
            OperandAccessType::Write,
        ),
        ("destination simd&fp register", OperandAccessType::Write),
        ("source and destination", OperandAccessType::Write),
        ("register to be loaded", OperandAccessType::Write),
        (
            "register to be compared and loaded",
            OperandAccessType::Write,
        ),
        ("accumulator output register", OperandAccessType::Write),
        (
            "register into which the status result of store exclusive is written",
            OperandAccessType::Write,
        ),
        (
            "status result of this instruction is written",
            OperandAccessType::Write,
        ),
        // Read access
        ("source register", OperandAccessType::Read),
        ("source general-purpose register", OperandAccessType::Read),
        ("source scalable vector register", OperandAccessType::Read),
        (
            "source scalable predicate register",
            OperandAccessType::Read,
        ),
        (
            "governing scalable predicate register",
            OperandAccessType::Read,
        ),
        ("simd&fp table register", OperandAccessType::Read),
        (
            "holding data value to be operated on with the contents of memory location",
            OperandAccessType::Read,
        ),
        ("register to be stored", OperandAccessType::Read),
        (
            "register to be conditionally stored",
            OperandAccessType::Read,
        ),
        ("general-purpose offset register", OperandAccessType::Read),
        ("offset scalable vector register", OperandAccessType::Read),
        ("accumulator input register", OperandAccessType::Read),
        (
            "register holding address to be branched to",
            OperandAccessType::Read,
        ),
        ("index register", OperandAccessType::Read),
        ("to be tested", OperandAccessType::Read),
        // Text access
        ("width specifier", OperandAccessType::Text),
        ("immediate index", OperandAccessType::Text),
        ("element index", OperandAccessType::Text),
        ("immediate multiplier", OperandAccessType::Text),
        ("field \"imm", OperandAccessType::Text),
    ];

    let desc = desc.to_lowercase();
    if desc.contains("register to be transferred")
        || desc.contains("scalable predicate transfer register")
    {
        if asm.trim().to_lowercase().starts_with("ld") {
            OperandAccessType::Write
        } else {
            OperandAccessType::Read
        }
    } else if desc.contains("general-purpose base register") {
        let addr_form = vars.get("address-form").and_then(|form| {
            assert_eq!(form.len(), 1);
            form.first().copied()
        });

        match addr_form {
            Some("pre-indexed") | Some("post-indexed") => OperandAccessType::Write,
            Some("base-register")
            | Some("base-plus-offset")
            | Some("signed-scaled-offset")
            | Some("unsigned-scaled-offset") => OperandAccessType::Read,
            Some(form) => {
                warn!("Unknown form ({}) for asm: {}", form, asm);
                OperandAccessType::Unknown
            }
            None => OperandAccessType::Read,
        }
    } else {
        access
            .iter()
            .find_map(|(x, a)| desc.as_str().contains(*x).then(|| a))
            .cloned()
            .unwrap_or(OperandAccessType::Unknown)
    }
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

    let expansions = extract_brackets(template)
        .into_iter()
        .filter(|exp| exp.contains('|'))
        .map(|exp| {
            (
                exp,
                exp.trim_start_matches('(')
                    .trim_end_matches(')')
                    .split('|')
                    .collect_vec(),
            )
        })
        .collect_vec();

    if expansions.is_empty() {
        ret
    } else {
        debug!("Performing expansions for template: {}", template);
        let expansions = expansions
            .iter()
            .map(|(expand, parts)| (*expand, parts))
            .collect_vec();

        ret.iter()
            .flat_map(|template| expand_template(template, expansions.as_slice()))
            .collect_vec()
    }
}

fn get_completion_values(description: &str) -> Option<Vec<CompletionValue>> {
    let start = description.find('[')?;
    let process_item = |part: &str| -> Option<CompletionValue> {
        if let Some(stripped) = part.strip_prefix('#') {
            // If we start with a # only allow it if the part following it is numeric
            stripped.parse::<i64>().ok()?;
            Some(CompletionValue::Left(part.to_string()))
        } else if part.contains('-') {
            crate::util::process_range(part)
        } else {
            Some(CompletionValue::Left(part.to_string()))
        }
    };

    if let Some(end) = description.find(']') {
        let items = &description[(start + 1)..end];
        Some(
            items
                .split(',')
                .filter_map(process_item)
                .collect::<Vec<_>>(),
        )
    } else {
        warn!("Description: {} doesn't have a closing ]", description);
        None
    }
}

pub(crate) async fn get_instructions() -> Result<Vec<Instruction>, Box<dyn Error>> {
    let isa_data = if let Ok(data) = std::fs::read(format!(
        "data/{}",
        A64_ISA.split('/').last().unwrap_or_default()
    )) {
        data
    } else {
        println!("Downloading XML reference from {}", A64_ISA);
        reqwest::get(A64_ISA).await?.bytes().await?.to_vec()
    };

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
    fn test_parse_template_or_token() {
        let input = "A <Wt>, [<Xn|SP>, (<Wm>|<Xm>), <extend> {<amount>}]";
        let operands = vec![OperandInfo {
            name: "<amount>".into(),
            description: "Optional".into(),
            completion_values: Default::default(),
        }];
        let result = parse_template(input, &operands);

        assert_eq!(
            result,
            vec![
                String::from("A <Wt>, [<Xn|SP>, <Wm>, <extend> <amount>]"),
                String::from("A <Wt>, [<Xn|SP>, <Xm>, <extend> <amount>]"),
                String::from("A <Wt>, [<Xn|SP>, <Wm>, <extend> ]"),
                String::from("A <Wt>, [<Xn|SP>, <Xm>, <extend> ]"),
            ]
        );
    }

    #[test]
    fn test_parse_template_multiple_or_token() {
        let input = "A (<op>|#<imm5>), [<Xn|SP>, (<Wm>|<Xm>){, <extend> {<amount>}}]";
        let operands = vec![
            OperandInfo {
                name: "<extend>".into(),
                description: "Optional".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<amount>".into(),
                description: "Optional".into(),
                completion_values: Default::default(),
            },
        ];
        let result = parse_template(input, &operands);

        assert_eq!(
            result,
            vec![
                String::from("A <op>, [<Xn|SP>, <Wm>, <extend> <amount>]"),
                String::from("A #<imm5>, [<Xn|SP>, <Wm>, <extend> <amount>]"),
                String::from("A <op>, [<Xn|SP>, <Xm>, <extend> <amount>]"),
                String::from("A #<imm5>, [<Xn|SP>, <Xm>, <extend> <amount>]"),
                String::from("A <op>, [<Xn|SP>, <Wm>, <extend> ]"),
                String::from("A #<imm5>, [<Xn|SP>, <Wm>, <extend> ]"),
                String::from("A <op>, [<Xn|SP>, <Xm>, <extend> ]"),
                String::from("A #<imm5>, [<Xn|SP>, <Xm>, <extend> ]"),
                String::from("A <op>, [<Xn|SP>, <Wm>]"),
                String::from("A #<imm5>, [<Xn|SP>, <Wm>]"),
                String::from("A <op>, [<Xn|SP>, <Xm>]"),
                String::from("A #<imm5>, [<Xn|SP>, <Xm>]"),
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

    #[test]
    fn test_build_access_map() {
        let asm = "NOT_REAL  <R><t>, #<imm>, <label>, <Pg>, <Xt>";
        let operands = vec![
            OperandInfo {
                name: "<R>".into(),
                description: "Width specifier".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<t>".into(),
                description: "to be tested or ZR".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<imm>".into(),
                description: "Bit number to be tested".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<label>".into(),
                description: "branched to".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<Pg>".into(),
                description: "governing scalable predicate register".into(),
                completion_values: Default::default(),
            },
            OperandInfo {
                name: "<Xt>".into(),
                description: "destination register".into(),
                completion_values: Default::default(),
            },
        ];
        let expected_map = vec![
            OperandAccessType::Read,
            OperandAccessType::Text,
            OperandAccessType::Text,
            OperandAccessType::Read,
            OperandAccessType::Write,
        ];
        assert_eq!(
            expected_map,
            build_access_map(asm, &operands, &Default::default())
        );
    }
}
