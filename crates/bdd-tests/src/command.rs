use std::str::FromStr;

#[derive(Debug)]
pub enum LSPCommand {
    GotoDefinition,
    FindReferences,
    DocumentHighlight,
    DocumentHover,
    SemanticTokens,
    DocumentSymbols,
    Codelens,
    SyntaxTree,
    Completion,
    SignatureHelp,
    NoCommand,
}

impl FromStr for LSPCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "goto definition" => Ok(Self::GotoDefinition),
            "find references" => Ok(Self::FindReferences),
            "document highlight" => Ok(Self::DocumentHighlight),
            "document hover" => Ok(Self::DocumentHover),
            "semantic tokens" => Ok(Self::SemanticTokens),
            "document symbols" => Ok(Self::DocumentSymbols),
            "codelens" => Ok(Self::Codelens),
            "syntax tree" => Ok(Self::SyntaxTree),
            "completion" => Ok(Self::Completion),
            "signature help" => Ok(Self::SignatureHelp),
            _ => Err(String::from("Unknown Command")),
        }
    }
}
