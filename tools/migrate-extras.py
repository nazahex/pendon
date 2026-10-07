#!/usr/bin/env python3
"""One-shot migration of the sandbox sources to the reconciled extras syntax.

Legacy `[.class,#id]{key: value}` blocks become the merged `{.class, #id, key: value}`
head, cite heads lose their quotes, and the table slots move the head behind the
alignment code (D3/D4/D5). Not a build step: it exists so the sweep is auditable.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def split_top(text):
    """Split on top-level commas (quote aware)."""
    parts, start, quote = [], 0, None
    for i, ch in enumerate(text):
        if quote:
            if ch == quote:
                quote = None
        elif ch in "\"'":
            quote = ch
        elif ch == ",":
            parts.append(text[start:i])
            start = i + 1
    parts.append(text[start:])
    return [part.strip() for part in parts if part.strip()]


def merge_legacy(text):
    """`[.a,#b]{k: v}` (and `[.a]`) -> `{.a, #b, k: v}`."""
    out = []
    i = 0
    while i < len(text):
        if text[i] == "[":
            close = text.find("]", i + 1)
            if close != -1:
                content = text[i + 1 : close].strip()
                if content == "" or content.startswith(".") or content.startswith("#"):
                    j = close + 1
                    props = ""
                    if j < len(text) and text[j] == "{":
                        end = find_brace(text, j)
                        if end is not None:
                            props = text[j + 1 : end].strip()
                            j = end + 1
                    items = split_top(content) + split_top(props)
                    if items:
                        out.append("{" + ", ".join(items) + "}")
                        i = j
                        continue
        out.append(text[i])
        i += 1
    return "".join(out)


def find_brace(text, start):
    """Index of the `}` matching `text[start] == '{'` (quote aware)."""
    quote = None
    for i in range(start + 1, len(text)):
        ch = text[i]
        if quote:
            if ch == quote:
                quote = None
        elif ch in "\"'":
            quote = ch
        elif ch == "}":
            return i
    return None


def migrate(text):
    # §7.3/D5: `[^^]("ref", "loc")` -> `[^^](ref "loc")`.
    text = re.sub(r'\[\^\^\]\("([^"]+)"\s*,\s*"([^"]*)"\)', r'[^^](\1 "\2")', text)
    text = re.sub(r'\[\^\^\]\("([^"]+)"\)', r"[^^](\1)", text)
    # §7.4: the retired `[.class]` heading bracket folds into the extras.
    text = re.sub(
        r"^(#{1,6}\[[^\]\.][^\]]*\])(\[[^\]]*\])(\{[^}]*\})?",
        lambda m: m.group(1) + merge_legacy(m.group(2) + (m.group(3) or "")),
        text,
        flags=re.M,
    )
    # §8: a legacy caption line becomes `|| extras content ||`.
    text = re.sub(
        r"^\[([^\]]*)\](?:(\[[^\]]*\]))?(\{[^}]*\})?$",
        lambda m: "||" + merge_legacy((m.group(2) or "") + (m.group(3) or ""))
        + " " + m.group(1) + "||",
        text,
        flags=re.M,
    )
    text = re.sub(
        r"^\[(.+)\](\[[^\]]*\])(\{[^}]*\})?$",
        lambda m: "||" + merge_legacy(m.group(2) + (m.group(3) or ""))
        + " " + m.group(1) + "||",
        text,
        flags=re.M,
    )
    # §8: `|| head content ||` loses the space after the fence.
    text = re.sub(
        r"^\|\|[ \t]*(\{[^}]*\}|@@[^\s|]*(?:\{[^}]*\})?)[ \t]*(.*?)[ \t]*\|\|$",
        r"||\1 \2||",
        text,
        flags=re.M,
    )
    # §8/D4: the delimiter-cell head goes behind the alignment code and width.
    text = re.sub(
        r"\|\s*(@@[^\s|]*(?:\{[^}]*\})?)\s*(:\s*-+\s*:?)(\([^)]*\))?",
        r"| \2\3\1",
        text,
    )
    text = re.sub(
        r"(:\s*-+\s*:?)\s*(\[[^\]]*\])(\{[^}]*\})?",
        lambda m: m.group(1) + merge_legacy(m.group(2) + (m.group(3) or "")),
        text,
    )
    # §8/D4: a legacy row head at the end of a row becomes a trailing head.
    text = re.sub(
        r"\|[ \t]*-\[([^\]]*)\](?:\{([^}]*)\})?[ \t]*$",
        lambda m: "|" + merge_legacy("[" + m.group(1) + "]{" + (m.group(2) or "") + "}"),
        text,
        flags=re.M,
    )
    return merge_legacy(text)


def main(paths):
    changed = []
    for path in paths:
        source = Path(path).read_text()
        updated = migrate(source)
        if updated != source:
            Path(path).write_text(updated)
            changed.append(path)
    for path in changed:
        print("migrated", path)
    print(f"{len(changed)} file(s) changed")


if __name__ == "__main__":
    main(sys.argv[1:] or [str(p) for p in sorted(ROOT.glob("sandbox/*/src/*.md"))])
