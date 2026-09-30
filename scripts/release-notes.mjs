import { readFileSync, appendFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";

export function formatReleaseNotes(manifest, version) {
  if (manifest.schemaVersion !== 1 || !Array.isArray(manifest.releases)) throw new Error("Invalid release notes schema");
  const entries = manifest.releases.filter((release) => release.version === version);
  if (entries.length !== 1) throw new Error("Exactly one release note must match the package version");
  const release = entries[0];
  const text = (value) => {
    if (typeof value !== "string" || !value.trim() || value.length > 4096 || /[\r\n\x00-\x08]/.test(value)) throw new Error("Invalid release note text");
    return value;
  };
  if (!/^\d{4}-\d{2}-\d{2}$/.test(release.date) || !Number.isFinite(Date.parse(release.date))) throw new Error("Invalid release date");
  if (!Array.isArray(release.changes) || !release.changes.length || release.changes.length > 20 || !Array.isArray(release.notes) || release.notes.length > 10) throw new Error("Invalid release sections");
  return [text(release.summary), "", ...release.changes.flatMap((change) => [`## ${text(change.title)}`, text(change.description), ""]), ...release.notes.map((note) => `- ${text(note)}`)].join("\n");
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const manifest = JSON.parse(readFileSync(new URL("../src/release-notes.json", import.meta.url), "utf8"));
  const version = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8")).version;
  const body = formatReleaseNotes(manifest, version);
  if (process.argv.includes("--github-output")) {
    const delimiter = "DISROUTE_RELEASE_NOTES_END";
    if (!process.env.GITHUB_OUTPUT || body.includes(delimiter)) throw new Error("Invalid GitHub output environment");
    appendFileSync(process.env.GITHUB_OUTPUT, `body<<${delimiter}\n${body}\n${delimiter}\n`, "utf8");
  } else if (!process.argv.includes("--check")) process.stdout.write(`${body}\n`);
}
