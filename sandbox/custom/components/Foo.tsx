import type { JSX } from "solid-js"

export interface FooProps {
  children: JSX.Element
  argument?: "bar" | "baz" | "zoo"
  attributes?: {
    qux?: string
    waldo?: number
    isBar?: boolean
  }
}

export default function Foo(props: FooProps) {
  return (
    <div class={`foo ${props.argument ?? ""}`} {...props.attributes}>
      {props.children}
    </div>
  )
}
