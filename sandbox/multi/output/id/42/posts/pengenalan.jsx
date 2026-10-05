export const headings = [{ id: "pengenalan", text: "Pengenalan", level: 1 }]
export default function PendonView() {
  return (
    <>
      <h1>Pengenalan</h1>
      <p>Ini adalah pengenalan Bahasa Indonesia.</p>
      <ul>
        <li>satu</li>
        <li>dua</li>
        <li>tiga</li>
        <li>empat</li>
      </ul>
      <p>
        Teks dengan <strong>kuat</strong> dan <strong>tebal</strong>, <i>miring</i> dan{" "}
        <i>italik</i>, <code>kode</code>, dan <a href="https://contoh.id">tautan</a>.
      </p>
      <pre>
        <code>fn utama() {}</code>
      </pre>
      <hr />
    </>
  )
}
