use lsp_types::{
    CodeLensOptions, CompletionOptions, CompletionOptionsCompletionItem, HoverProviderCapability,
    OneOf, SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensServerCapabilities, ServerCapabilities, SignatureHelpOptions,
    TextDocumentSyncCapability, TextDocumentSyncKind, WorkDoneProgressOptions,
};

pub fn get_server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        document_highlight_provider: Some(OneOf::Left(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        signature_help_provider: Some(SignatureHelpOptions {
            trigger_characters: Some(vec![String::from(" ")]),
            retrigger_characters: Some(vec![String::from(",")]),
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
        }),
        code_lens_provider: Some(CodeLensOptions {
            resolve_provider: None,
        }),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                work_done_progress_options: WorkDoneProgressOptions {
                    work_done_progress: None,
                },
                legend: SemanticTokensLegend {
                    token_types: crate::handler::semantic::TOKEN_TYPES.to_vec(),
                    token_modifiers: vec![],
                },

                full: Some(SemanticTokensFullOptions::Bool(true)),
                range: Some(true),
            },
        )),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec![String::from(" "), String::from(",")]),
            all_commit_characters: None,
            work_done_progress_options: Default::default(),
            completion_item: Some(CompletionOptionsCompletionItem {
                label_details_support: Some(true),
            }),
        }),
        ..Default::default()
    }
}
