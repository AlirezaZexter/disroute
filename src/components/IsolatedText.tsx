// Render as text only, isolating Latin names/numbers within Persian prose.
export function IsolatedText({ text }: { text: string }) {
  return text.split(/([A-Za-z0-9][A-Za-z0-9 .+/_-]*[A-Za-z0-9]|[A-Za-z0-9])/g).map((part, index) =>
    /^[A-Za-z0-9]/.test(part) ? <bdi dir="ltr" key={index}>{part}</bdi> : part);
}
