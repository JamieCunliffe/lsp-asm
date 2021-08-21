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

(defcustom lsp-asm-analysis-cpus nil
  "Map of cpus to use for each architecture for running code analysis."
  :type '(alist :key-type (string) :value-type (string))
  :group 'lsp-asm)

(defcustom lsp-asm-log-level "error"
  "The logging level to use."
  :type '(choice (const "error")
                 (const "warn")
                 (const "info")
                 (const "debug")
                 (const "trace"))
  :group 'lsp-asm)

(lsp-interface (asm:SyntaxTreeParams (:textDocument))
               (asm:AnalysisParams (:textDocument) (:range)))
(define-derived-mode lsp-asm-syntax-tree-mode special-mode "Asm-Syntax-Tree"
  "Mode for the asm syntax tree buffer.")
(defun lsp-asm-syntax-tree ()
  "Display the syntax tree for the current buffer."
  (interactive)
  (-let* ((root (lsp-workspace-root default-directory))
          (params (lsp-make-asm-syntax-tree-params
                   :text-document (lsp--text-document-identifier)))
          (results (lsp-send-request (lsp-make-request
                                      "asm/syntaxTree"
                                      params))))
    (let ((buf (get-buffer-create (format "*asm syntax tree %s*" root)))
          (inhibit-read-only t))
      (with-current-buffer buf
        (lsp-asm-syntax-tree-mode)
        (erase-buffer)
        (insert results)
        (goto-char (point-min)))
      (pop-to-buffer buf))))

(define-derived-mode lsp-asm-analysis-mode special-mode "LLVM-MCA"
  "Mode for the llvm mca results buffer.")
(defun lsp-asm-run-analysis ()
  "Run llvm-mca to analyse the assembly."
  (interactive)
  (-let* ((root (lsp-workspace-root default-directory))
          (params (lsp-make-asm-analysis-params
                   :text-document (lsp--text-document-identifier)
                   :range? (if (use-region-p)
                               (lsp--region-to-range (region-beginning) (region-end))
                             (lsp--region-to-range (point-min) (point-max)))))
          (results (lsp-send-request (lsp-make-request
                                      "asm/runAnalysis"
                                      params))))
    (let ((buf (get-buffer-create (format "*llvm-mca %s*" root)))
          (inhibit-read-only t))
      (with-current-buffer buf
        (lsp-asm-analysis-mode)
        (erase-buffer)
        (insert results)
        (goto-char (point-min)))
      (pop-to-buffer buf))))

(defun lsp-asm--make-init-options ()
  "Init options for lsp-asm."
  `(:architecture ,lsp-asm-default-architecture
    :codelens (:enabled_filesize ,lsp-asm-codelens-filesize-threshold
               :loc_enabled ,(lsp-json-bool lsp-asm-codelens-loc-enabled))
    :analysis (:default_cpus ,(json-read-from-string (json-encode-alist lsp-asm-analysis-cpus)))))

(lsp-defun lsp-asm--open-loc
  ((&Command :title :arguments? [location]))
  (lsp-show-xrefs (lsp--locations-to-xref-items location) nil nil))

(defun lsp-asm--resync-document (workspace params)
  "Resync the document."
  (-let* ((uri (gethash "uri" params))
          (path (lsp--uri-to-path uri)))
    (lsp-with-current-buffer (find-buffer-visiting path)
      (lsp-log "Resync requested for: %s" path)
      (lsp-on-revert))))

(add-to-list 'lsp-language-id-configuration '(asm-mode . "Assembly"))
(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection "lsp-asm")
                  :major-modes '(asm-mode)
                  :notification-handlers (ht ("client/registerCapability" 'ignore)
                                             ("textDocument/resync" 'lsp-asm--resync-document))
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
