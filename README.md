An LSP for various Assembly Languages, this has mostly been designed to improve reading of assembly output from compilers (-S option). It also supports objdump disassembly.

It currently supports the following LSP messages:
* Goto definition (label)
* Find references (registers, labels)
* Document Hover (numeric tokens to show decimal and hex, demangled names for labels)
* Document highlights (registers, labels)
* Semantic Tokens/Syntax Highlighting (labels, registers, numbers, comments, directives, instructions)
* Document Symbols

# Installing
This can be installed by running `cargo install --path crates/lsp` from the root of this repository, the resulting binary will be placed within `~/.cargo/bin/`. If another location is desired this can be built with `cargo build --release` and then copy the binary from `./target/release/lsp-asm` to the desired location.

## emacs client
There is an emacs lsp-mode client contained within the directory `clients/emacs/`. Just load this package with your preferred method.

## vscode client
The vscode client is located in the `clients/code/` directory. This can be installed by running `npm install` from that directory.
