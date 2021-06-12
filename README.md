An LSP for various Assembly Languages, this has mostly been designed to improve reading of assembly output from compilers (think -S option).

It currently supports the following LSP messages:
* Goto definition (label)
* Find references (registers, labels)
* Document Hover (numeric tokens to show decimal and hex)
* Document highlights (registers, labels)
* Semantic Tokens/Syntax Highlighting (labels, registers, numbers, comments, directives, instructions)
* Document Symbols

# Installing
This can be installed by running `cargo install --path ./lsp` from the root of this repository, the resulting binary will be placed within `~/.cargo/bin/`. If another location is desired this this can be build with `cargo build --release` and then copy the binary from `./target/release/lsp-asm` to the desired location.
