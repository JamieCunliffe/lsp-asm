An LSP for various Assembly Languages, this has mostly been designed to improve reading of assembly output from compilers (-S option). It also supports objdump disassembly.

It currently supports the following LSP messages:
* Goto definition (label, `.loc` directive)
* Find references (registers, labels)
* Document Hover (numeric tokens to show decimal and hex, demangled names for labels, instruction description (see [Documentation](#documentation) for installation instructions)
* Document highlights (registers, labels)
* Semantic Tokens/Syntax Highlighting (labels, registers, numbers, comments, directives, instructions)
* Document Symbols
* Codelens (shows line `.loc` directive refers to)
* Completion (based on the documentation, experimental)
* Signature help (based on the documentation, experimental)

## Additional commands
* `runAnalysis` Run LLVM MCA on the file/region

# Installing
This can be installed by running `cargo install --path crates/lsp` from the root of this repository, the resulting binary will be placed within `~/.cargo/bin/`. If another location is desired this can be built with `cargo build --release` and then copy the binary from `./target/release/lsp-asm` to the desired location.

## emacs client
There is an emacs lsp-mode client contained within the directory `clients/emacs/`. Just load this package with your preferred method.

## vscode client
The vscode client is located in the `clients/code/` directory. This can be installed by running `npm install` from that directory.

## Documentation
Additional hovers are provided by using third party documentation. This can be downloaded and installed by using `cargo x build-docs` see `cargo x build-docs --help` for a list of available documentation.

