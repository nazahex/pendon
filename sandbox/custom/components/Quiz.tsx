import { createSignal, For, type JSX, Show } from "solid-js"
import "@style/course/quiz.scss"

// import Mars from "@mars" // Uncomment jika ada komponen markdown
// import { triggerConfettiFromCursor } from "@lib/confetti" // Uncomment jika ada

type Choice = { content: string; correct: boolean }
type Feedback = { correct?: string; wrong?: string }

interface QuizProps {
  children: JSX.Element | HTMLElement | string
  choices: Choice[]
  feedback?: Feedback
}

export default function Quiz(props: QuizProps): JSX.Element {
  const [status, setStatus] = createSignal<"ready" | "correct" | "wrong">("ready")
  const [selectedIndex, setSelectedIndex] = createSignal<number | null>(null)
  const [feedbackMsg, setFeedbackMsg] = createSignal("")

  const handleChoice = (i: number, _e: MouseEvent) => {
    if (status() === "correct") return
    setSelectedIndex(i)
    if (props.choices[i].correct) {
      setStatus("correct")
      setFeedbackMsg(props.feedback?.correct || "Correct!")
      // triggerConfettiFromCursor?.(_e) // Uncomment jika ada
    } else {
      setStatus("wrong")
      setFeedbackMsg(props.feedback?.wrong || "Incorrect, try again.")
    }
  }

  return (
    <aside class={`quiz ${status()}`}>
      <div class="cn-artes">
        <div class="wrapper">
          <div class="quiz-body">{props.children}</div>
          <div class="quiz-input">
            <For each={props.choices}>
              {(choice, i) => (
                <button
                  type="button"
                  class={[
                    selectedIndex() === i() ? "selected" : "",
                    selectedIndex() === i() && status() === "correct" && choice.correct
                      ? "correct"
                      : "",
                    selectedIndex() === i() && status() === "wrong" && !choice.correct
                      ? "wrong"
                      : "",
                  ]
                    .filter(Boolean)
                    .join(" ")}
                  onClick={(e) => handleChoice(i(), e)}
                  disabled={status() === "correct"}
                  tabindex="0"
                  aria-pressed={selectedIndex() === i()}>
                  {/* <Mars markdown={choice.text} /> */}
                  <div innerHTML={choice.content} />
                </button>
              )}
            </For>
          </div>
          <Show when={selectedIndex() !== null}>
            <div class="feedback" aria-live="polite">
              {feedbackMsg()}
            </div>
          </Show>
        </div>
      </div>
    </aside>
  )
}
