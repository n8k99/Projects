# G105 — Database Backup Script Maker

> The Rosetta Stone's fifth **Databases project**. Introduces **code generation** — the first Rosetta Stone project where structured input produces *executable text output* (a bash script) meant for a different runtime. The hard problems are **shell-safe quoting** (single-quote escape via `'\''`), **identifier validation** (no SQL/shell injection via table names), and **deterministic emission** (same spec → same bytes, always). G105 is the "write a script that writes a script" pattern every ops team eventually builds.

```yaml
id: G105
title: Database Backup Script Maker
category: databases
requires: [G083-template-maker, G092-bulk-renamer, G101-sql-query-analyzer]
provides: [code-generation, shell-safe-quoting, identifier-allow-list, validated-then-emit]
```

## Insight: Code Generation Is Template + Validation

A code generator has two halves: a validator that rejects malformed configs, and an emitter that turns valid configs into deterministic text. G105 separates them: `validate(spec) -> errors`, then `emit_script(spec) -> script | errors`. Invalid configs never reach the emitter; valid ones produce output that's byte-reproducible.

First Rosetta Stone project where **the output is code, not data**. G085's quiz text was data; G094's logs were data. G105's bash script is **executable** — it runs in a different interpreter and has its own security surface.

## Insight: Shell Quoting Uses Single Quotes + `'\''` Escape

Bash single-quoted strings are literal — no interpolation, no escape sequences, except that a single quote cannot appear inside. The canonical workaround: close the quote, emit a backslash-escaped quote, reopen. `it's` becomes `'it'\''s'`.

G105's `shell_quote` does this mechanically: replace `'` with `'\''` in the input, then wrap the result in `'`. Handles embedded quotes, spaces, shell metacharacters, and newlines safely. Every shell script generator must implement this — mistakes here are CVEs.

First Rosetta Stone project where **shell-safe quoting is a security primitive**. G101's tokeniser handled string literals inside SQL; G105's quoter handles string literals for shell. Different syntax, same class of problem.

## Insight: Identifier Allow-List Beats Shell-Quoting Alone

Table names and database names go into **unquoted** positions in pg_dump flags: `--table=users`. You could shell-quote them, but that doesn't help if the name itself contains SQL-syntactic characters. Instead, G105 validates identifiers up front: must match `[a-zA-Z_][a-zA-Z0-9_]*`. Anything else is rejected with `InvalidIdentifier`.

This is defence-in-depth: the identifier is quoted in the shell (so metacharacters are literal to bash), AND the identifier is validated (so no SQL metacharacters reach pg_dump). Either layer alone has holes; both together are robust.

First Rosetta Stone project with **an explicit allow-list for user-provided strings that flow into multiple interpreters**. G101 analysed SQL but didn't generate it; G105 generates it and protects the border.

## Insight: Determinism Enables Diff-and-Review

`emit_script(spec) == emit_script(spec)` is a hard invariant. Same inputs → same bytes. That enables the pattern every ops team uses:
* Version-control the config, not the script.
* Regenerate the script on config change.
* `diff` old vs new script to review what changed.
* Reject non-deterministic outputs (timestamps, hashes of random) as they break the workflow.

G105 keeps determinism by sorting table lists before emission, quoting deterministically, and not embedding any timestamps or random values in the script text (the `STAMP=$(date ...)` in the script itself runs at script-execution time, not at script-emission time — that's the right place for it).

First Rosetta Stone project where **deterministic emission is load-bearing** for downstream diff review.

## Insight: Validation Errors Are a List, Not a Single Exception

`validate(spec)` returns a `Vec<ValidationError>`. If there are multiple problems, the caller sees all of them in one shot — not "fix one, resubmit, find the next". UIs render the list; CI fails with all reasons; docs can generate test cases for each variant.

This mirrors G101's `Finding` pattern: errors are data, and plural. The severity ladder isn't used here (all are effectively errors that block emission), but the "list, not exception" choice is the same.

First Rosetta Stone project where **config validation surfaces all errors in one pass**. G085's quiz parser stopped at the first error; G101 and G105 emit everything the checker found, so iteration converges fast.

## Choreographic Case: Vault Scheduled Backups

```innate
(@vault-scheduled-backups){
  @configs <- @vault/read-configs{glob: "backup/*.yaml"}
  @for config in @configs {
    @spec <- @bsm/spec-from-yaml{yaml: @config.content}
    @result <- @bsm/emit-script{spec: @spec}
    @when (@result.errors){
      @ui/show-errors{config: @config.path, errors: @result.errors}
    }
    @else {
      @vault/save{path: "backup-scripts/${@spec.database}.sh",
                   content: @result.script,
                   mode: 0755}
      @cron/install{schedule: "0 2 * * *", script: "backup-scripts/${@spec.database}.sh"}
    }
  }
}
```

The vault's backup pane reads YAML configs, emits scripts, installs cron entries. Validation surfaces misconfigurations before any script hits the filesystem.

## Structures

```innate
(defenum format SQL | CSV_PER_TABLE)

(defstruct backup-spec
  database        : String
  host            : String
  port            : Int
  user            : String
  tables-include  : [String]
  tables-exclude  : [String]
  destination-dir : String
  compress        : Bool
  format          : Format)

(defenum validation-error
  EMPTY_DATABASE | INVALID_IDENTIFIER | EMPTY_DESTINATION | PORT_ZERO)
```

## Resolver Natives

```innate
@bsm/spec{database}                   -> BackupSpec      ;; with defaults
@bsm/validate{spec}                   -> [ValidationError]
@bsm/shell-quote{str}                 -> String
@bsm/emit-script{spec}                -> String | [ValidationError]
```

## Demo

```innate
(@demo){
  @spec <- @bsm/spec{database: "appdb"}
  @spec.tables-include <- ["users", "orders"]
  @spec.tables-exclude <- ["logs"]
  @spec.compress <- true
  @bsm/emit-script{spec: @spec}
  ;; -> "#!/usr/bin/env bash
  ;;     set -euo pipefail
  ;;     DB='appdb' ...
  ;;     pg_dump ... --exclude-table='logs' --table='orders' --table='users' | gzip > ..."

  @bsm/emit-script{spec: {database: "drop; --"}}
  ;; -> [INVALID_IDENTIFIER 'drop; --']
}
```

## Where

Shell strings MUST be single-quoted with `'\''` escape for embedded quotes — any other quoting scheme has known bypasses. Identifiers (database + table names) MUST be validated against `[a-zA-Z_][a-zA-Z0-9_]*` before emission — shell-quoting alone doesn't prevent SQL syntax injection when identifiers flow into SQL contexts unquoted. Validation MUST return all errors, not stop at the first — iterative config debugging converges faster when all problems are visible at once. Emission MUST be deterministic — same spec → same script bytes, because ops teams diff-and-review generated scripts and non-determinism breaks that loop. Sort-then-emit for table lists — unordered table flags produce unnecessary diffs. No timestamps or random values MUST appear in the emitted script text — those belong inside the script (evaluated at runtime), not in the emission path.
