/- Calculator — evaluate arithmetic expressions. -/

namespace Numbers

/-- Token types for the calculator. -/
inductive CalcToken where
  | num   : Float → CalcToken
  | plus  : CalcToken
  | minus : CalcToken
  | star  : CalcToken
  | slash : CalcToken
  | lparen : CalcToken
  | rparen : CalcToken
deriving Repr

/-- Tokenize an expression string into a list of tokens. -/
partial def calcTokenize (expr : String) : Option (List CalcToken) :=
  let chars := expr.toList
  let rec go (cs : List Char) (acc : List CalcToken) : Option (List CalcToken) :=
    match cs with
    | [] => some acc.reverse
    | c :: rest =>
      if c == ' ' || c == '\t' then go rest acc
      else if c == '+' then go rest (CalcToken.plus :: acc)
      else if c == '-' then go rest (CalcToken.minus :: acc)
      else if c == '*' then go rest (CalcToken.star :: acc)
      else if c == '/' then go rest (CalcToken.slash :: acc)
      else if c == '(' then go rest (CalcToken.lparen :: acc)
      else if c == ')' then go rest (CalcToken.rparen :: acc)
      else if c.isDigit || c == '.' then
        let numChars := (c :: rest).takeWhile (fun ch => ch.isDigit || ch == '.')
        let remaining := (c :: rest).dropWhile (fun ch => ch.isDigit || ch == '.')
        let numStr := String.mk numChars
        match numStr.toFloat? with
        | some n => go remaining (CalcToken.num n :: acc)
        | none => none
      else none
  go chars []

/-- Parser state: a list of remaining tokens. -/
abbrev ParseResult := Option (Float × List CalcToken)

/-- Parse an atom: NUMBER | '(' expr ')'. -/
partial def parseAtom : List CalcToken → ParseResult
  | CalcToken.num n :: rest => some (n, rest)
  | CalcToken.lparen :: rest =>
    match parseExpr rest with
    | some (val, CalcToken.rparen :: rest') => some (val, rest')
    | _ => none
  | CalcToken.minus :: rest =>
    match parseAtom rest with
    | some (val, rest') => some (-val, rest')
    | none => none
  | CalcToken.plus :: rest => parseAtom rest
  | _ => none

/-- Parse a term: unary (('*' | '/') unary)*. -/
partial def parseTerm (tokens : List CalcToken) : ParseResult :=
  match parseAtom tokens with
  | none => none
  | some (left, rest) =>
    let rec loop (acc : Float) (toks : List CalcToken) : ParseResult :=
      match toks with
      | CalcToken.star :: rest' =>
        match parseAtom rest' with
        | some (right, rest'') => loop (acc * right) rest''
        | none => none
      | CalcToken.slash :: rest' =>
        match parseAtom rest' with
        | some (right, rest'') =>
          if right == 0.0 then none
          else loop (acc / right) rest''
        | none => none
      | _ => some (acc, toks)
    loop left rest

/-- Parse an expression: term (('+' | '-') term)*. -/
partial def parseExpr (tokens : List CalcToken) : ParseResult :=
  match parseTerm tokens with
  | none => none
  | some (left, rest) =>
    let rec loop (acc : Float) (toks : List CalcToken) : ParseResult :=
      match toks with
      | CalcToken.plus :: rest' =>
        match parseTerm rest' with
        | some (right, rest'') => loop (acc + right) rest''
        | none => none
      | CalcToken.minus :: rest' =>
        match parseTerm rest' with
        | some (right, rest'') => loop (acc - right) rest''
        | none => none
      | _ => some (acc, toks)
    loop left rest

/-- Evaluate an arithmetic expression string. -/
def evaluate (expr : String) : Option Float :=
  match calcTokenize expr with
  | none => none
  | some [] => none
  | some tokens =>
    match parseExpr tokens with
    | some (result, []) => some result
    | _ => none

end Numbers
