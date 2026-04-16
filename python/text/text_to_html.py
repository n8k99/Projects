"""Text to HTML Generator — convert formatted plain text to HTML."""

import re


def _escape_html(text: str) -> str:
    """Escape HTML entities in content."""
    return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def _apply_inline(text: str) -> str:
    """Apply inline formatting: bold, italic, code, links.

    Order matters — code first (to protect its contents), then bold,
    italic, and finally links.
    """
    # Code spans: `code` -> <code>code</code>
    text = re.sub(r"`([^`]+)`", r"<code>\1</code>", text)
    # Bold: **text** -> <strong>text</strong>
    text = re.sub(r"\*\*(.+?)\*\*", r"<strong>\1</strong>", text)
    # Italic: *text* -> <em>text</em>
    text = re.sub(r"\*(.+?)\*", r"<em>\1</em>", text)
    # Links: [text](url) -> <a href="url">text</a>
    text = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", r'<a href="\2">\1</a>', text)
    return text


def text_to_html(text: str) -> str:
    """Convert formatted plain text to HTML.

    Supported formatting:
    - Paragraphs separated by blank lines -> <p> tags
    - Bold: **text** -> <strong>text</strong>
    - Italic: *text* -> <em>text</em>
    - Headers: # H1, ## H2, ### H3
    - Links: [text](url) -> <a href="url">text</a>
    - Unordered lists: lines starting with "- " -> <ul><li>
    - Code: `code` -> <code>code</code>
    - Line breaks within paragraphs -> <br>
    - HTML entity escaping for &, <, >
    """
    lines = text.split("\n")
    blocks: list[str] = []
    current: list[str] = []

    def flush_paragraph():
        if not current:
            return
        joined = "<br>\n".join(current)
        blocks.append(f"<p>{_apply_inline(joined)}</p>")
        current.clear()

    def flush_list(items: list[str]):
        inner = "\n".join(f"<li>{_apply_inline(item)}</li>" for item in items)
        blocks.append(f"<ul>\n{inner}\n</ul>")

    list_items: list[str] = []
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        # Blank line — flush whatever we're accumulating
        if not stripped:
            if list_items:
                flush_list(list_items)
                list_items = []
            flush_paragraph()
            i += 1
            continue

        # Header
        header_match = re.match(r"^(#{1,3})\s+(.+)$", stripped)
        if header_match:
            if list_items:
                flush_list(list_items)
                list_items = []
            flush_paragraph()
            level = len(header_match.group(1))
            content = _escape_html(header_match.group(2))
            blocks.append(f"<h{level}>{_apply_inline(content)}</h{level}>")
            i += 1
            continue

        # List item
        if stripped.startswith("- "):
            flush_paragraph()
            list_items.append(_escape_html(stripped[2:]))
            i += 1
            continue

        # If we were in a list but this line isn't a list item, flush it
        if list_items:
            flush_list(list_items)
            list_items = []

        # Regular text — accumulate into current paragraph
        current.append(_escape_html(stripped))
        i += 1

    # Flush any remaining content
    if list_items:
        flush_list(list_items)
    flush_paragraph()

    return "\n".join(blocks)


if __name__ == "__main__":
    sample = """\
# Welcome to the Forge

This is a **bold statement** about *italic dreams*.

## Features

- Convert `markdown` to HTML
- Support **bold** and *italic*
- Handle [links](https://example.com)

### Details

Paragraphs with multiple lines
get line breaks between them.

Use special characters like <, >, and & safely."""

    print(text_to_html(sample))
