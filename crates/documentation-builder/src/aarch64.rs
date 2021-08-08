use std::error::Error;
use std::ffi::OsStr;
use std::io::prelude::*;

use itertools::Itertools;

use log::warn;
use lsp_asm::documentation::{Instruction, InstructionTemplate, OperandInfo};

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
            .map(|n| OperandInfo {
                name: n.text().unwrap_or("").to_string(),
                description: n
                    .attribute_node("hover")
                    .map(|a| a.value())
                    .unwrap_or("")
                    .to_string(),
            })
            .collect()
    });

    let asm_template = asm
        .zip(operands)
        .map(|(asm, items)| InstructionTemplate {
            asm: parse_template(&asm),
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
            description: description.trim().to_string(),
            asm_template: asm_template.clone(),
        })
        .collect()
}

fn parse_template(template: &str) -> Vec<String> {
    let mut ret = vec![template.replace(|a| a == '{' || a == '}', "")];

    let open = template
        .chars()
        .enumerate()
        .filter(|(_, a)| a == &'{')
        .map(|(idx, _)| idx)
        .clone()
        .collect::<Vec<_>>();
    let close = template
        .chars()
        .enumerate()
        .filter(|(_, a)| a == &'}')
        .map(|(idx, _)| idx)
        .clone()
        .collect::<Vec<_>>();

    ret.extend(
        open.iter()
            .zip(close.iter().rev())
            .map(|(open, close)| {
                template
                    .chars()
                    .take(*open)
                    .chain(template.chars().skip(close + 1))
                    .collect::<String>()
                    .replace(|a| a == '{' || a == '}', "")
                    .trim_end()
                    .to_string()
            })
            .rev(),
    );

    ret
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

    #[test]
    fn test_parse_template_expansion() {
        let input = "STP  <Xt1>, <Xt2>, [<Xn|SP>{, #<imm>}]";
        let result = parse_template(&input);
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
        let input = "ADDS  <Xd>, <Xn|SP>, <R><m>{, <extend> {#<amount>}}";
        let result = parse_template(&input);

        assert_eq!(
            result,
            vec![
                String::from("ADDS  <Xd>, <Xn|SP>, <R><m>, <extend> #<amount>"),
                String::from("ADDS  <Xd>, <Xn|SP>, <R><m>, <extend>"),
                String::from("ADDS  <Xd>, <Xn|SP>, <R><m>"),
            ]
        );
    }
}
