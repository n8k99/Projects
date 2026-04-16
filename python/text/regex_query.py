"""Regex Query Tool — pattern matching, extraction, and replacement."""

import re
from typing import NamedTuple


class Match(NamedTuple):
    """A match with its position in the source text."""
    text: str
    start: int
    end: int


# Pre-built patterns for common structured data.
PATTERNS = {
    "EMAIL": r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",
    "URL": r"https?://[^\s<>\"']+",
    "PHONE": r"\+?1?[-.\s]?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}",
    "DATE": r"\d{4}-\d{2}-\d{2}",
    "IP_ADDRESS": r"\b(?:\d{1,3}\.){3}\d{1,3}\b",
}


class RegexTool:
    """Apply regex patterns to text — find, extract, replace, split, validate."""

    patterns = PATTERNS

    def find_matches(self, text: str, pattern: str) -> list[str]:
        """Return all substrings matching *pattern*."""
        return re.findall(pattern, text)

    def find_with_positions(self, text: str, pattern: str) -> list[Match]:
        """Return matches with their (start, end) positions."""
        return [Match(m.group(), m.start(), m.end()) for m in re.finditer(pattern, text)]

    def replace(self, text: str, pattern: str, replacement: str) -> str:
        """Replace all occurrences of *pattern* with *replacement*."""
        return re.sub(pattern, replacement, text)

    def split(self, text: str, pattern: str) -> list[str]:
        """Split *text* on every occurrence of *pattern*."""
        return re.split(pattern, text)

    def is_match(self, text: str, pattern: str) -> bool:
        """Return True if *pattern* matches anywhere in *text*."""
        return re.search(pattern, text) is not None

    def extract_groups(self, text: str, pattern: str) -> list[tuple[str, ...]]:
        """Return capture-group tuples for every match of *pattern*."""
        return re.findall(pattern, text)


if __name__ == "__main__":
    tool = RegexTool()

    sample = (
        "Contact alice@example.com or bob@work.org. "
        "Visit https://example.com and https://docs.python.org/3 for info. "
        "Event on 2026-04-16, backup on 2026-05-01. "
        "Call 555-867-5309 or (212) 555-1234. "
        "Server at 192.168.1.1 and 10.0.0.1."
    )

    print("=== Find emails ===")
    emails = tool.find_matches(sample, PATTERNS["EMAIL"])
    for e in emails:
        print(f"  {e}")

    print("\n=== Extract URLs with positions ===")
    urls = tool.find_with_positions(sample, PATTERNS["URL"])
    for m in urls:
        print(f"  {m.text}  [{m.start}:{m.end}]")

    print("\n=== Replace dates (YYYY-MM-DD -> DD/MM/YYYY) ===")
    replaced = tool.replace(sample, r"(\d{4})-(\d{2})-(\d{2})", r"\3/\2/\1")
    print(f"  {replaced}")

    print("\n=== Split on commas ===")
    csv_line = "apples, bananas, cherries, dates"
    parts = tool.split(csv_line, r",\s*")
    print(f"  {parts}")

    print("\n=== Validate IP addresses ===")
    tests = ["192.168.1.1", "not-an-ip", "10.0.0.1", "999.999.999.999"]
    for t in tests:
        valid = tool.is_match(t, r"^" + PATTERNS["IP_ADDRESS"] + r"$")
        print(f"  {t}: {valid}")

    print("\n=== Extract groups (parse log entries) ===")
    log = "[2026-04-16 08:30:00] ERROR: disk full\n[2026-04-16 09:15:22] INFO: recovered"
    entries = tool.extract_groups(log, r"\[(\d{4}-\d{2}-\d{2}) (\d{2}:\d{2}:\d{2})\] (\w+): (.+)")
    for date, time, level, msg in entries:
        print(f"  {date} {time} [{level}] {msg}")
