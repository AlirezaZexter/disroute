import { currentRelease } from "../releaseNotes";
import { useEffect, useRef } from "react";

function IsolatedText({ text }: { text: string }) {
  return text.split(/([A-Za-z0-9][A-Za-z0-9 .+/_-]*[A-Za-z0-9]|[A-Za-z0-9])/g).map((part, index) =>
    /^[A-Za-z0-9]/.test(part) ? <bdi dir="ltr" key={index}>{part}</bdi> : part);
}

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
      <div className="release-notes">{currentRelease.notes.map((note) => <p key={note}><IsolatedText text={note} /></p>)}</div>
    </section>
  );
}

// GitHub updater bodies are untrusted text. No HTML, remote images or links.
export function ReleasePreview({ notes }: { notes: string }) {
  return <div className="release-preview">{notes.slice(0, 16384).split(/\r?\n/).filter((line) => line.trim()).slice(0, 80).map((line, index) =>
    <p key={index} dir="auto"><IsolatedText text={line.replace(/^#{1,6}\s+/, "").replace(/^[-*]\s+/, "• ")} /></p>)}</div>;
}
