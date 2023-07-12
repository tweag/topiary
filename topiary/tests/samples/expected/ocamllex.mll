(**************************************************************************)
(*  Taken from https://github.com/colis-anr/morbig/                       *)
(**************************************************************************)
{
  open TextFormulaParser

  exception UnexpectedCharacter of char
  exception UnterminatedQuote

  let keywords =
    [ "true",   TRUE;
      "false",  FALSE;
      "and",    AND;
      "or",     OR;
      "not",    NOT ]
}

let alpha = ['a'-'z' 'A'-'Z']
let symbol = ['_' '-']
let digit = ['0'-'9']

let identifier = (alpha | symbol | digit)+

rule token = parse

  | ' ' { token lexbuf }

  | '(' { LPAR }
  | ')' { RPAR }

  | ":" (identifierasid) {
      match List.assoc_opt id keywords with
      | Some kw -> kw
      | _ -> NULLARY_PREDICATE id
    }

  | '!' { NOT }
  | "&&" { AND }
  | "||" { OR }

  | (identifierasid) ":" {
      PREDICATE id
    }

  | '"' {
      let buf = Buffer.create 8 in
      LITERAL (string buf lexbuf)
    }

  |
  (identifieraslit) {
      LITERAL lit
    }

  | _asc { raise (UnexpectedCharacter c) }

  | eof { EOF }

and stringbuf = parse
  | '"' { Buffer.contents buf }
  | _asc { Buffer.add_char buf c; string buf lexbuf }
  | eof { raise UnterminatedQuote }

and erin = parse "erin" { Erin }
