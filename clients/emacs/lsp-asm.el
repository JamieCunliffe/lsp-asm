;;; package --- lsp-asm.el
;;; Commentary:

;; LSP client for the LSP-ASM language server

;;; Code:

(require 'lsp-mode)
(require 'lsp-semantic-tokens)

(defcustom lsp-asm-default-architecture "native"
  "The architecture to use if it can't be determined from the file contents."
  :type '(choice (const "native")
                 (const "aarch64")
                 (const "x86-64"))
  :group 'lsp-asm)

(defcustom lsp-asm-log-level "error"
  "The logging level to use."
  :type '(choice (const "error")
                 (const "warn")
                 (const "info")
                 (const "debug")
                 (const "trace"))
  :group 'lsp-asm)


(defun lsp-asm--make-init-options ()
  "Init options for lsp-asm."
  `(:architecture ,lsp-asm-default-architecture))

(add-to-list 'lsp-language-id-configuration '(asm-mode . "Assembly"))
(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection "lsp-asm")
                  :major-modes '(asm-mode)
                  :notification-handlers (ht ("client/registerCapability" 'ignore))
                  :priority 1
                  :initialization-options 'lsp-asm--make-init-options
                  :environment-fn (lambda ()
                                    '(("RUST_LOG" . lsp-asm-log-level)))
                  :server-id 'lsp-asm))

(defface lsp-face-semhl-register
  '((t (:inherit font-lock-type-face)))
  "Face used for semantic highlighting registers."
  :group 'lsp-faces)

(defface lsp-face-semhl-metadata
  '((t (:inherit font-lock-comment-face)))
  "Face used for semantic highlighting metadata."
  :group 'lsp-faces)

(add-to-list 'lsp-semantic-token-faces
             '("register" . lsp-face-semhl-register))
(add-to-list 'lsp-semantic-token-faces
             '("metadata" . lsp-face-semhl-metadata))

(provide 'lsp-asm)
;;; lsp-asm.el ends here
