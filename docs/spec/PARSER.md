# Parser Spec: Paragraph Invariants

Status: Draft (MVP)

## Paragraph Boundaries

- Open: first non-newline character after zero or more newlines starts a paragraph and emits `StartNode(Paragraph)`.
- Close: a blank line (two consecutive newline tokens) closes the current paragraph via `EndNode(Paragraph)`.
- Trailing close: at end-of-input, any open paragraph is closed.

## Newlines

- CRLF (\r\n) is normalized to a single LF (\n) at tokenization.
- Every newline is preserved as a `Text("\n")` event for output fidelity, regardless of paragraph state (leading/trailing newlines are retained).

## Leading/Trailing Blank Lines

- Leading newlines before the first paragraph are emitted as `Text("\n")` but do not open a paragraph.
- Trailing newlines after the last paragraph are emitted as `Text("\n")` but do not open a paragraph.

## Blank Run Guard

- Option `max_blank_run: Option<usize>` controls diagnostics for consecutive newlines.
- When set and a newline sequence exceeds the limit, a single `Event::Diagnostic { severity: Warning }` is emitted at the first exceedance.
- Text is not altered by this guard; concatenation remains identical. In strict mode (future), this may become an error.

## Line Length Guard

- Option `max_line_len: Option<usize>` compares the number of non-newline characters between newline tokens.
- When exceeded, a `Diagnostic { severity: Warning }` is emitted at newline time.

## Document Events

- Stream always begins with `StartNode(Document)` and ends with `EndNode(Document)`.
