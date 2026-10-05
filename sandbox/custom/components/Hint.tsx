import type { JSX } from "solid-js"

type HintType = "info" | "warning" | "error" | "!" | "?" | "x"

function normalizeType(type: HintType): "info" | "warning" | "error" {
  if (type === "!") return "warning"
  if (type === "?") return "info"
  if (type === "x") return "error"
  return type
}

export default function Hint(props: {
  type: HintType
  children: JSX.Element | HTMLElement | string
}) {
  const kind = normalizeType(props.type)
  return <aside class={`hint ${kind}`}>{props.children}</aside>
}
