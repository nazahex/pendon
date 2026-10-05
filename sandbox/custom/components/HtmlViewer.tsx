function toStringChunk(value: unknown): string {
  if (value === null || value === undefined) return ""
  if (typeof value === "string" || typeof value === "number" || typeof value === "boolean") {
    return String(value)
  }
  if (Array.isArray(value)) return value.map(toStringChunk).join("")
  if (typeof value === "function") return toStringChunk((value as () => unknown)())
  if (typeof value === "object" && value && "toString" in value) {
    return (value as { toString(): string }).toString()
  }
  return ""
}

export default function HtmlViewer(props: { value?: string | undefined }) {
  const html = () => toStringChunk(props.value)

  return (
    <iframe
      class="html-viewer"
      sandbox=""
      title="HTML Viewer"
      srcdoc={html() || "<p><i>No content</i></p>"}
      style="width: 100%; min-height: 240px; border: 1px solid #ddd; border-radius: 6px;"
    />
  )
}
