; Query for detecting zsh interpreters
; Matches shebangs that contain "zsh" in the path or as an argument

; Direct path to zsh: #!/bin/zsh, #!/usr/bin/zsh
((interpreter_path) @zsh
  (#match? @zsh "zsh$"))

; Via env: #!/usr/bin/env zsh
((argument) @zsh
  (#match? @zsh "\\szsh$"))
