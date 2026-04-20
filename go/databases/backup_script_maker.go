// Database Backup Script Maker — code generation with shell-safe quoting.
package databases

import (
	"fmt"
	"sort"
	"strings"
)

// BSMFormat selects the output style.
type BSMFormat int

const (
	BSMSql          BSMFormat = 0
	BSMCsvPerTable  BSMFormat = 1
)

// BSMSpec is the backup configuration.
type BSMSpec struct {
	Database       string
	Host           string
	Port           int
	User           string
	TablesInclude  []string
	TablesExclude  []string
	DestinationDir string
	Compress       bool
	Format         BSMFormat
}

// NewBSMSpec returns a spec with sensible defaults.
func NewBSMSpec(database string) BSMSpec {
	return BSMSpec{
		Database:       database,
		Host:           "localhost",
		Port:           5432,
		User:           "postgres",
		DestinationDir: "/var/backups",
		Compress:       true,
		Format:         BSMSql,
	}
}

// BSMValidation tags an error kind.
type BSMValidation int

const (
	BSMErrEmptyDatabase     BSMValidation = 0
	BSMErrInvalidIdentifier BSMValidation = 1
	BSMErrEmptyDestination  BSMValidation = 2
	BSMErrPortZero          BSMValidation = 3
)

// BSMError is one validation error.
type BSMError struct {
	Kind   BSMValidation
	Detail string
}

// IsValidIdentifier returns true if s is a bare SQL/shell identifier.
func IsValidIdentifier(s string) bool {
	if len(s) == 0 { return false }
	c := s[0]
	if !((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_') {
		return false
	}
	for i := 1; i < len(s); i++ {
		c := s[i]
		if !((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
			(c >= '0' && c <= '9') || c == '_') {
			return false
		}
	}
	return true
}

// ValidateBSM runs all checks and returns a list of errors.
func ValidateBSM(s *BSMSpec) []BSMError {
	var errs []BSMError
	if s.Database == "" { errs = append(errs, BSMError{Kind: BSMErrEmptyDatabase}) }
	if s.DestinationDir == "" { errs = append(errs, BSMError{Kind: BSMErrEmptyDestination}) }
	if s.Port == 0 { errs = append(errs, BSMError{Kind: BSMErrPortZero}) }
	if !IsValidIdentifier(s.Database) {
		errs = append(errs, BSMError{Kind: BSMErrInvalidIdentifier, Detail: s.Database})
	}
	for _, t := range s.TablesInclude {
		if !IsValidIdentifier(t) {
			errs = append(errs, BSMError{Kind: BSMErrInvalidIdentifier, Detail: t})
		}
	}
	for _, t := range s.TablesExclude {
		if !IsValidIdentifier(t) {
			errs = append(errs, BSMError{Kind: BSMErrInvalidIdentifier, Detail: t})
		}
	}
	return errs
}

// ShellQuote wraps in single quotes with embedded-quote escape.
func ShellQuote(s string) string {
	return "'" + strings.ReplaceAll(s, "'", "'\\''") + "'"
}

// EmitScript produces the bash script text.
func EmitScript(s *BSMSpec) (string, []BSMError) {
	if errs := ValidateBSM(s); len(errs) > 0 {
		return "", errs
	}
	var b strings.Builder
	b.WriteString("#!/usr/bin/env bash\n")
	b.WriteString("set -euo pipefail\n\n")
	fmt.Fprintf(&b, "DB=%s\n", ShellQuote(s.Database))
	fmt.Fprintf(&b, "HOST=%s\n", ShellQuote(s.Host))
	fmt.Fprintf(&b, "PORT=%d\n", s.Port)
	fmt.Fprintf(&b, "USER=%s\n", ShellQuote(s.User))
	fmt.Fprintf(&b, "DEST=%s\n", ShellQuote(s.DestinationDir))
	b.WriteString("STAMP=$(date -u +%Y%m%dT%H%M%SZ)\n")
	b.WriteString("mkdir -p \"$DEST\"\n\n")

	switch s.Format {
	case BSMSql:
		cmd := "pg_dump -h \"$HOST\" -p \"$PORT\" -U \"$USER\" \"$DB\""
		excluded := map[string]bool{}
		for _, t := range s.TablesExclude { excluded[t] = true }
		excs := append([]string{}, s.TablesExclude...)
		sort.Strings(excs)
		for _, t := range excs {
			cmd += " --exclude-table=" + ShellQuote(t)
		}
		var includes []string
		for _, t := range s.TablesInclude {
			if !excluded[t] { includes = append(includes, t) }
		}
		sort.Strings(includes)
		for _, t := range includes {
			cmd += " --table=" + ShellQuote(t)
		}
		b.WriteString(cmd)
		if s.Compress {
			b.WriteString(" | gzip > \"$DEST/${DB}_${STAMP}.sql.gz\"")
		} else {
			b.WriteString(" > \"$DEST/${DB}_${STAMP}.sql\"")
		}
		b.WriteByte('\n')
	case BSMCsvPerTable:
		var tables []string
		if len(s.TablesInclude) > 0 {
			excluded := map[string]bool{}
			for _, t := range s.TablesExclude { excluded[t] = true }
			for _, t := range s.TablesInclude {
				if !excluded[t] { tables = append(tables, ShellQuote(t)) }
			}
			sort.Strings(tables)
		} else {
			tables = []string{"__introspect__"}
		}
		for _, t := range tables {
			base := fmt.Sprintf("psql -h \"$HOST\" -p \"$PORT\" -U \"$USER\" -c \"\\\\copy %s TO STDOUT WITH CSV HEADER\" \"$DB\"", t)
			b.WriteString(base)
			if s.Compress {
				fmt.Fprintf(&b, " | gzip > \"$DEST/${DB}_%s_${STAMP}.csv.gz\"\n", t)
			} else {
				fmt.Fprintf(&b, " > \"$DEST/${DB}_%s_${STAMP}.csv\"\n", t)
			}
		}
	}

	b.WriteString("\necho \"backup complete: $DEST\"\n")
	return b.String(), nil
}
