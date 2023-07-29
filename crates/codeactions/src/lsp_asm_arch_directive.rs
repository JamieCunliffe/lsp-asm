use base::Architecture;
use syntax::utils::find_token_containing;

use crate::{ActionContext, CodeAction, Edit};

pub(crate) fn run(ctx: &ActionContext) -> Option<Vec<CodeAction>> {
    // Only run the action if the cursor is at the start of the file.
    if ctx.start != 0.into() {
        return None;
    }

    // If we already have the directive no need to insert another
    if find_token_containing(&ctx.root, "lsp-asm-architecture").is_some() {
        return None;
    }

    Some(
        Architecture::iter()
            .filter(|arch| !matches!(arch, Architecture::Unknown))
            .map(|arch| CodeAction {
                name: format!("Insert lsp-asm-architecture: {arch} directive"),
                edit: vec![Edit {
                    start: 0.into(),
                    end: 0.into(),
                    text: format!(
                        "{comment_start} lsp-asm-architecture: {arch}\n",
                        comment_start = arch.default_comment_start(),
                    ),
                }],
            })
            .collect(),
    )
}
