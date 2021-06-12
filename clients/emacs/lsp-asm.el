;;; package --- lsp-asm.el
;;; Commentary:

;; LSP client for the LSP-ASM language server

;;; Code:

(require 'lsp-mode)
(require 'lsp-semantic-tokens)

(add-to-list 'lsp-language-id-configuration '(asm-mode . "Assembly"))
(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection "lsp-asm")
                  :major-modes '(asm-mode)
                  :notification-handlers (ht ("client/registerCapability" 'ignore))
                  :priority 1
                  :server-id 'lsp-asm))

(defface lsp-face-semhl-register
  '((t (:inherit font-lock-type-face)))
  "Face used for semantic highlighting registers."
  :group 'lsp-faces)

(add-to-list 'lsp-semantic-token-faces
             '("register" . lsp-face-semhl-register))

(provide 'lsp-asm)
;;; lsp-asm.el ends here
