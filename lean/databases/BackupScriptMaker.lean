-- Database Backup Script Maker — code generation with shell-safe quoting.

namespace Databases.BSM

inductive Format where | sql | csvPerTable
  deriving Repr, BEq

structure BackupSpec where
  database : String
  host : String := "localhost"
  port : Nat := 5432
  user : String := "postgres"
  tablesInclude : List String := []
  tablesExclude : List String := []
  destinationDir : String := "/var/backups"
  compress : Bool := true
  format : Format := .sql
  deriving Repr

inductive ValidationError where
  | emptyDatabase
  | invalidIdentifier (s : String)
  | emptyDestination
  | portZero
  deriving Repr

def Char.isIdentStart (c : Char) : Bool :=
  c.isAlpha ∨ c == '_'

def Char.isIdentCont (c : Char) : Bool :=
  c.isAlphanum ∨ c == '_'

def isValidIdentifier (s : String) : Bool :=
  match s.toList with
  | [] => false
  | c :: rest =>
    Char.isIdentStart c ∧ rest.all Char.isIdentCont

def validate (spec : BackupSpec) : List ValidationError := Id.run do
  let mut errs : List ValidationError := []
  if spec.database.isEmpty then errs := errs ++ [.emptyDatabase]
  if spec.destinationDir.isEmpty then errs := errs ++ [.emptyDestination]
  if spec.port == 0 then errs := errs ++ [.portZero]
  if !isValidIdentifier spec.database then
    errs := errs ++ [.invalidIdentifier spec.database]
  for t in spec.tablesInclude ++ spec.tablesExclude do
    if !isValidIdentifier t then
      errs := errs ++ [.invalidIdentifier t]
  return errs

def shellQuote (s : String) : String :=
  "'" ++ (s.replace "'" "'\\''") ++ "'"

def emitScript (spec : BackupSpec) : Except (List ValidationError) String := do
  let errs := validate spec
  if !errs.isEmpty then throw errs
  let header := "#!/usr/bin/env bash\nset -euo pipefail\n\n" ++
    s!"DB={shellQuote spec.database}\n" ++
    s!"HOST={shellQuote spec.host}\n" ++
    s!"PORT={spec.port}\n" ++
    s!"USER={shellQuote spec.user}\n" ++
    s!"DEST={shellQuote spec.destinationDir}\n" ++
    "STAMP=$(date -u +%Y%m%dT%H%M%SZ)\n" ++
    "mkdir -p \"$DEST\"\n\n"
  let body := match spec.format with
    | .sql =>
      let mut cmd := "pg_dump -h \"$HOST\" -p \"$PORT\" -U \"$USER\" \"$DB\""
      let excs := spec.tablesExclude.mergeSort (· < ·)
      for t in excs do
        cmd := cmd ++ " --exclude-table=" ++ shellQuote t
      let incs := (spec.tablesInclude.filter (fun t => !spec.tablesExclude.contains t)).mergeSort (· < ·)
      for t in incs do
        cmd := cmd ++ " --table=" ++ shellQuote t
      let tail := if spec.compress
        then " | gzip > \"$DEST/${DB}_${STAMP}.sql.gz\""
        else " > \"$DEST/${DB}_${STAMP}.sql\""
      cmd ++ tail ++ "\n"
    | .csvPerTable =>
      let tables := if spec.tablesInclude.isEmpty then ["__introspect__"]
        else (spec.tablesInclude.filter (fun t => !spec.tablesExclude.contains t)).mergeSort (· < ·) |>.map shellQuote
      tables.foldl (init := "") fun acc t =>
        let base := s!"psql -h \"$HOST\" -p \"$PORT\" -U \"$USER\" -c \"\\\\copy {t} TO STDOUT WITH CSV HEADER\" \"$DB\""
        let tail := if spec.compress
          then s!" | gzip > \"$DEST/${{DB}}_{t}_${{STAMP}}.csv.gz\""
          else s!" > \"$DEST/${{DB}}_{t}_${{STAMP}}.csv\""
        acc ++ base ++ tail ++ "\n"
  return header ++ body ++ "\necho \"backup complete: $DEST\"\n"

end Databases.BSM
