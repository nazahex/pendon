import Quiz from "@comp/Quiz"

export default function PendonView() {
  return (
    <>
      <h3>Quiz Demo</h3>
      <p>
        Magna sint cillum est sunt incididunt sint excepteur aute. Ipsum aliqua pariatur pariatur
        aute excepteur aliquip elit ea nulla nostrud reprehenderit culpa qui Lorem.
      </p>
      <p>Officia et consectetur ea officia.</p>
      <Quiz
        choices={[
          { content: "right answer", correct: true },
          { content: "wrong answer", correct: false },
          { content: "another <code>right</code> answer", correct: true },
          { content: "another <strong>wrong</strong> answer", correct: false },
        ]}
        feedback={{ correct: "This is correct feedback", wrong: "This is wrong feedback" }}>
        <p>
          Exercitation Lorem consectetur ullamco irure. Lorem reprehenderit nulla enim cupidatat.
        </p>
        <blockquote>
          <p>Ipsum ex deserunt laborum adipisicing velit Lorem et in</p>
        </blockquote>
        <pre lang="js">
          <code>const foo = bar + 3 console.debug(bar)</code>
        </pre>
        <p>Consequat eu ad voluptate ad culpa proident officia qui tempor?</p>
      </Quiz>
      <p>
        Veniam reprehenderit fugiat minim minim voluptate anim. Aliqua incididunt aliqua pariatur in
        dolore ex veniam adipisicing. Qui labore deserunt sit nostrud aliquip ut culpa exercitation
        et exercitation excepteur pariatur ex.
      </p>
    </>
  )
}
