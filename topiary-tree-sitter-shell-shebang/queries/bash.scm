; Query for detecting bash interpreters
; Matches shebangs that contain "bash" in the path or as an argument

; Direct path to bash: #!/bin/bash, #!/usr/bin/bash
((interpreter_path) @bash
  (#match? @bash "bash$"))

; Via env: #!/usr/bin/env bash
((argument) @bash
  (#match? @bash "\\sbash$"))
