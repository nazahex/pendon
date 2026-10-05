import Hint from "./Hint"
import HtmlViewer from "./HtmlViewer"
import You from "./You"

export default function PendonView() {
  return (
    <>
      <h1>Custom Syntax Demo 2</h1>
      <p>
        Hello <You />!<br />
        <You c /> are great!
      </p>
      <Hint type="!">
        <p>
          This is a <strong>warning</strong> hint
        </p>
      </Hint>
      <Hint type="?">
        <p>
          This is an <i>info</i> <b>hint</b>
        </p>
      </Hint>
      <Hint type="x">
        <p>
          This is an error hint and <You />
        </p>
      </Hint>
      <HtmlViewer value={"<h1>HTML Viewer</h1> <p>This should be rendered inside an iframe</p>"} />
      <pre lang="html">
        <code>
          &lt;h1&gt;HTML Viewer&lt;/h1&gt; &lt;p&gt;This should be rendered inside a codeblock as
          regular&lt;/p&gt;
        </code>
      </pre>
    </>
  )
}
