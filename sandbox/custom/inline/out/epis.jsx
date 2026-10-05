import Epis from "@comp/content/Epis"

export default function PendonView() {
  return (
    <>
      <p>
        Non excepteur velit quis velit aliqua sint deserunt enim. Ut quis anim sit anim. Consectetur
        id labore officia tempor ea dolore ad magna ut anim proident nostrud. Proident nisi nisi
        duis commodo ad enim dolor nisi tempor proident nulla proident.{" "}
        <Epis level={3} phase={3} ref={"foo-bar"}>
          Nostrud non exercitation amet dolore <strong>exercitation</strong> <i>reprehenderit</i>{" "}
          est{" "}
          <a href="https://example.com" title="Aliqua">
            aliqua
          </a>{" "}
          aliquip amet.
        </Epis>
      </p>
      <p>
        Duis aliqua ut id sunt cupidatat excepteur Lorem id ipsum.{" "}
        <Epis level={4} ref={"sunt"}>
          Sunt non veniam minim adipisicing fugiat id ad est. Sit dolor mollit laborum exercitation
          adipisicing sunt.
        </Epis>{" "}
        Laboris laboris ullamco laboris excepteur laborum. Dolore cupidatat officia sunt
        reprehenderit veniam sunt ea amet quis incididunt et.{" "}
        <Epis level={3}>
          Adaptasi pasar: Motif kaligrafi dan bentuk mangkuk menunjukkan adaptasi spesifik terhadap
          permintaan konsumen Dinasti Abasiyah di Baghdad.
        </Epis>
      </p>
    </>
  )
}
