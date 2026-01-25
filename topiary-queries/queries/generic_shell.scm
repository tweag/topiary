; Topiary formatting query for shell shebang files
; Extracts interpreter from shebang and formats script content with that language

; Format shebang line components
"#!"
(env_path) @append_space
(whitespace) @delete
(direct_interpreter) @append_hardline

; Env-style interpreter
(env_interpreter) @injection.language @append_hardline @prepend_space

; Direct interpreter - extract just the interpreter name (bash/zsh)
(interpreter_name) @injection.language

; The script content (everything after shebang) gets formatted as the detected language
; Add a blank line before the script content
(script_content) @injection.content @prepend_hardline
