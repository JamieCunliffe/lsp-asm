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

(defcustom lsp-asm-codelens-filesize-threshold "1mb"
  "The maximum filesize that codelens are applied to."
  :group 'lsp-asm)

(defcustom lsp-asm-codelens-loc-enabled 't
  "Provide a code lens showing the line a .loc directive refers to."
  :type 'boolean
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
  `(:architecture ,lsp-asm-default-architecture
    :codelens (:enabled_filesize ,lsp-asm-codelens-filesize-threshold
               :loc_enabled ,(lsp-json-bool lsp-asm-codelens-loc-enabled))))

(lsp-defun lsp-asm--open-loc
  ((&Command :title :arguments? [location]))
  (lsp-show-xrefs (lsp--locations-to-xref-items location) nil nil))

(add-to-list 'lsp-language-id-configuration '(asm-mode . "Assembly"))
(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection "lsp-asm")
                  :major-modes '(asm-mode)
                  :notification-handlers (ht ("client/registerCapability" 'ignore))
                  :priority 1
                  :initialization-options 'lsp-asm--make-init-options
                  :action-handlers (ht ("lsp-asm.loc" #'lsp-asm--open-loc))
                  :environment-fn (lambda ()
                                    '(("RUST_LOG" . lsp-asm-log-level)))
                  :server-id 'lsp-asm))

(defface lsp-face-semhl-register
  '((t (:inherit font-lock-type-face)))
  "Face used for semantic highlighting registers."
  :group 'lsp-faces)

(defface lsp-face-semhl-gp-register
  '((t (:inherit lsp-face-semhl-register)))
  "Face used for general purpose register"
  :group 'lsp-faces)

(defface lsp-face-semhl-fp-register
  '((t (:inherit lsp-face-semhl-register)))
  "Face used for floating point register"
  :group 'lsp-faces)

(defface lsp-face-semhl-metadata
  '((t (:inherit font-lock-comment-face)))
  "Face used for semantic highlighting metadata."
  :group 'lsp-faces)

(add-to-list 'lsp-semantic-token-faces
             '("register" . lsp-face-semhl-register))
(add-to-list 'lsp-semantic-token-faces
             '("gp-register" . lsp-face-semhl-gp-register))
(add-to-list 'lsp-semantic-token-faces
             '("fp-register" . lsp-face-semhl-fp-register))
(add-to-list 'lsp-semantic-token-faces
             '("metadata" . lsp-face-semhl-metadata))

(provide 'lsp-asm)
;;; lsp-asm.el ends here
