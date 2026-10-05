// `c` stands for "capitalized"
export default function You(props: { c?: boolean }) {
  return <>{props.c ? "You" : "you"}</>
}
