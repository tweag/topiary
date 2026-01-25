; Injection query for shell shebang detection
; Extracts interpreter and injects script content with appropriate language

; Direct bash interpreter: /bin/bash, /usr/bin/bash, etc.
((direct_interpreter) @_bash
  (#match? @_bash "bash$")
  (#set! injection.language "bash"))

; Direct zsh interpreter: /bin/zsh, /usr/bin/zsh, etc.
((direct_interpreter) @_zsh
  (#match? @_zsh "zsh$")
  (#set! injection.language "zsh"))

; Env-style interpreter - captures just "bash" or "zsh" directly
(env_interpreter) @injection.language

; The script content (everything after shebang) gets injected
(script_content) @injection.content

