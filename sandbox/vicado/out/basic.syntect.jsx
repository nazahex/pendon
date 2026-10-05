import { Vicado } from "vicado"

export default function PendonView() {
  return (
    <>
      <h1>Vicado Basic</h1>
      <p>Dokumen ini berisi contoh Vicado minimal dan lanjutan.</p>
      <h2>Minimal</h2>
      <Vicado language="typescript" code={"const a = 1\nconst b = 2\nconsole.log(a + b)"} />
      <h2>Full Props</h2>
      <Vicado
        language="tsx"
        code={"export function greet(name: string) {\n  return `hello ${name}`\n}"}
        class="hero is-vicado"
        id="editor-main"
        fontScale={1.1}
        lineNumbers={true}
        mount="visible"
        tabSize={2}
        theme="solarized"
      />
      <h2>Multiple Languages</h2>
      <Vicado
        language="javascript"
        code={"const now = new Date()\nconsole.log(now.toISOString())"}
        cache={true}
        mount="defer"
      />
      <Vicado
        language="css"
        code={".button {\n  color: #222;\n  border: 1px solid #888;\n}"}
        class="snippet"
        mount="visible"
      />
    </>
  )
}
