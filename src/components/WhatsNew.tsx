import { currentRelease } from "../releaseNotes";
import { useEffect, useRef } from "react";
import { IsolatedText } from "./IsolatedText";

export function WhatsNewNotice({ onOpen, onDismiss }: { onOpen: () => void; onDismiss: () => void }) {
  return (
    <aside className="release-notice" aria-label="تازه‌های نسخهٔ نصب‌شده">
      <p>نسخهٔ <bdi dir="ltr">{currentRelease.version}</bdi><span> · </span><IsolatedText text={currentRelease.summary} /></p>
      <div>
        <button className="text-button" type="button" onClick={onOpen}>دیدن تغییرات</button>
        <button className="text-button release-dismiss" type="button" aria-label="بستن اعلان تازه‌های نسخه" onClick={onDismiss}>بستن</button>
      </div>
    </aside>
  );
}

export function WhatsNewView() {
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => { heading.current?.focus({ preventScroll: true }); }, []);
  return (
    <section className="panel release-panel" aria-labelledby="release-heading" dir="rtl">
      <header className="release-heading">
        <div><span className="section-label" dir="ltr">What's new</span><h2 ref={heading} tabIndex={-1} id="release-heading">تازه‌های نسخه</h2><p><IsolatedText text={currentRelease.summary} /></p></div>
        <div className="release-meta"><bdi className="release-version" dir="ltr">{currentRelease.version}</bdi><time dateTime={currentRelease.date}>{new Intl.DateTimeFormat("fa-IR", { dateStyle: "long", timeZone: "UTC" }).format(new Date(`${currentRelease.date}T12:00:00Z`))}</time></div>
      </header>
      <div className="release-changes">
        {currentRelease.changes.map((change) => <article key={change.title}><h3>{change.title}</h3><p><IsolatedText text={change.description} /></p></article>)}
      </div>
      <div className="release-notes"><h3>نکات مهم</h3><ul>{currentRelease.notes.map((note) => <li key={note}><IsolatedText text={note} /></li>)}</ul></div>
    </section>
  );
}

// GitHub updater bodies are untrusted text. No HTML, remote images or links.
export function ReleasePreview({ notes }: { notes: string }) {
  type Block = { kind: "heading" | "paragraph" | "list"; lines: string[] };
  const blocks: Block[] = [];
  for (const raw of notes.slice(0, 16384).split(/\r?\n/).slice(0, 80)) {
    const line = raw.trim();
    if (!line) continue;
    const heading = /^#{1,6}\s+(.+)$/.exec(line);
    const bullet = /^[-*]\s+(.+)$/.exec(line);
    if (heading) blocks.push({ kind: "heading", lines: [heading[1]] });
    else if (bullet) {
      const previous = blocks.at(-1);
      if (previous?.kind === "list") previous.lines.push(bullet[1]);
      else blocks.push({ kind: "list", lines: [bullet[1]] });
    } else blocks.push({ kind: "paragraph", lines: [line] });
  }
  // A Latin product name at the start must not flip a Persian paragraph.
  const direction = (text: string) => /[\u0600-\u06ff]/.test(text) ? "rtl" : "ltr";
  return <div className="release-preview" dir="rtl" tabIndex={0} role="region" aria-label="توضیحات نسخهٔ جدید">
    {blocks.map((block, index) => block.kind === "heading"
      ? <h3 key={index} dir={direction(block.lines[0])}><IsolatedText text={block.lines[0]} /></h3>
      : block.kind === "list"
        ? <ul key={index}>{block.lines.map((line, i) => <li key={i} dir={direction(line)}><IsolatedText text={line} /></li>)}</ul>
        : <p key={index} dir={direction(block.lines[0])}><IsolatedText text={block.lines[0]} /></p>)}
  </div>;
}
