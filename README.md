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


# Enabling in emacs
```lisp
(defface lsp-face-semhl-register
  '((t (:inherit font-lock-type-face)))
  "Face used for semantic highlighting registers"
  :group 'lsp-faces)

(add-to-list 'lsp-semantic-token-faces
             '("register" . lsp-face-semhl-register))

(add-to-list 'lsp-language-id-configuration '(asm-mode . "Assembly"))
(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection "/path/to/lsp-asm")
                  :major-modes '(asm-mode)
                  :notification-handlers (ht ("client/registerCapability" 'ignore))
                  :priority 1
                  :server-id 'asm-ls))
```
