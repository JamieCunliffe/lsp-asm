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
    CodeAction,
    InlayHints,
    SyntaxTree,
    Completion,
    SignatureHelp,
    Rename,
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
            "codeaction" => Ok(Self::CodeAction),
            "inlay hints" => Ok(Self::InlayHints),
            "syntax tree" => Ok(Self::SyntaxTree),
            "completion" => Ok(Self::Completion),
            "signature help" => Ok(Self::SignatureHelp),
            "rename" => Ok(Self::Rename),
            _ => Err(String::from("Unknown Command")),
        }
    }
}
