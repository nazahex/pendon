export const headings = [{ id: "intro", text: "Intro", level: 1 }]
export default function PendonView() {
  return (
    <>
      <h1>Intro</h1>
      <p>This is the English intro.</p>
      <ul>
        <li>a</li>
        <li>b</li>
        <li>c</li>
        <li>d</li>
      </ul>
      <p>
        Text with <strong>strong</strong> and <strong>bold</strong>, <i>em</i> and <i>italic</i>,{" "}
        <code>code</code>, and <a href="https://example.com">link</a>.
      </p>
      <pre>
        <code>fn main() {}</code>
      </pre>
      <hr />
    </>
  )
}
