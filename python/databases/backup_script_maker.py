"""Database Backup Script Maker — code generation with shell-safe quoting.

BackupSpec → bash script. Identifier validation; shell single-quote
escaping; include/exclude table flags; SQL or CSV output; compress flag.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class Format(Enum):
    SQL = "sql"
    CSV_PER_TABLE = "csv_per_table"


@dataclass
class BackupSpec:
    database: str
    host: str = "localhost"
    port: int = 5432
    user: str = "postgres"
    tables_include: list[str] = field(default_factory=list)
    tables_exclude: list[str] = field(default_factory=list)
    destination_dir: str = "/var/backups"
    compress: bool = True
    format: Format = Format.SQL


class ValidationError(Enum):
    EMPTY_DATABASE = "empty_database"
    INVALID_IDENTIFIER = "invalid_identifier"
    EMPTY_DESTINATION = "empty_destination"
    PORT_ZERO = "port_zero"


@dataclass
class Error:
    kind: ValidationError
    detail: str = ""


def is_valid_identifier(s: str) -> bool:
    if not s:
        return False
    if not (s[0].isascii() and (s[0].isalpha() or s[0] == "_")):
        return False
    for c in s[1:]:
        if not (c.isascii() and (c.isalnum() or c == "_")):
            return False
    return True


def validate(spec: BackupSpec) -> list[Error]:
    errors: list[Error] = []
    if not spec.database:
        errors.append(Error(ValidationError.EMPTY_DATABASE))
    if not spec.destination_dir:
        errors.append(Error(ValidationError.EMPTY_DESTINATION))
    if spec.port == 0:
        errors.append(Error(ValidationError.PORT_ZERO))
    if not is_valid_identifier(spec.database):
        errors.append(Error(ValidationError.INVALID_IDENTIFIER, spec.database))
    for t in spec.tables_include + spec.tables_exclude:
        if not is_valid_identifier(t):
            errors.append(Error(ValidationError.INVALID_IDENTIFIER, t))
    return errors


def shell_quote(s: str) -> str:
    return "'" + s.replace("'", "'\\''") + "'"


def emit_script(spec: BackupSpec):
    errors = validate(spec)
    if errors:
        return errors
    lines = []
    lines.append("#!/usr/bin/env bash")
    lines.append("set -euo pipefail")
    lines.append("")
    lines.append(f"DB={shell_quote(spec.database)}")
    lines.append(f"HOST={shell_quote(spec.host)}")
    lines.append(f"PORT={spec.port}")
    lines.append(f"USER={shell_quote(spec.user)}")
    lines.append(f"DEST={shell_quote(spec.destination_dir)}")
    lines.append("STAMP=$(date -u +%Y%m%dT%H%M%SZ)")
    lines.append('mkdir -p "$DEST"')
    lines.append("")

    if spec.format == Format.SQL:
        cmd = 'pg_dump -h "$HOST" -p "$PORT" -U "$USER" "$DB"'
        includes = set(spec.tables_include) - set(spec.tables_exclude)
        for t in sorted(spec.tables_exclude):
            cmd += f" --exclude-table={shell_quote(t)}"
        for t in sorted(includes):
            cmd += f" --table={shell_quote(t)}"
        if spec.compress:
            lines.append(f'{cmd} | gzip > "$DEST/${{DB}}_${{STAMP}}.sql.gz"')
        else:
            lines.append(f'{cmd} > "$DEST/${{DB}}_${{STAMP}}.sql"')
    else:
        tables = (
            [shell_quote(t) for t in sorted(set(spec.tables_include) - set(spec.tables_exclude))]
            if spec.tables_include
            else ['$(psql -h "$HOST" -p "$PORT" -U "$USER" -t -c "\\\\dt" "$DB" | awk \'{print $3}\')']
        )
        for t in tables:
            base = f'psql -h "$HOST" -p "$PORT" -U "$USER" -c "\\\\copy {t} TO STDOUT WITH CSV HEADER" "$DB"'
            if spec.compress:
                lines.append(f'{base} | gzip > "$DEST/${{DB}}_{t}_${{STAMP}}.csv.gz"')
            else:
                lines.append(f'{base} > "$DEST/${{DB}}_{t}_${{STAMP}}.csv"')

    lines.append("")
    lines.append('echo "backup complete: $DEST"')
    return "\n".join(lines) + "\n"


def demo() -> None:
    spec = BackupSpec(
        database="appdb",
        tables_include=["users", "orders"],
        tables_exclude=["logs"],
        compress=True,
    )
    result = emit_script(spec)
    if isinstance(result, list):
        for e in result:
            print(f"  error: {e.kind.value} ({e.detail})")
    else:
        print(result, end="")


if __name__ == "__main__":
    demo()
