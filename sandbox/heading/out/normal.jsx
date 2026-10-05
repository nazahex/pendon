export const headings = [
  { id: "foo-bar", text: "Foo Bar", level: 2 },
  { id: "bar-qux", text: "Bar Qux", level: 2 },
]
export default function PendonView() {
  return (
    <>
      <section id="foo-bar">
        <h2>Foo Bar</h2>
        <p>Quis pariatur reprehenderit cupidatat minim velit.</p>
      </section>
      <section id="bar-qux">
        <h2>Bar Qux</h2>
        <p>
          Eiusmod non duis cillum aute esse eiusmod ullamco reprehenderit anim veniam laborum
          eiusmod.
        </p>
      </section>
    </>
  )
}
