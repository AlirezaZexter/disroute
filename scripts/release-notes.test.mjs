import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { formatReleaseNotes } from "./release-notes.mjs";

const manifest = JSON.parse(readFileSync(new URL("../src/release-notes.json", import.meta.url)));
const version = JSON.parse(readFileSync(new URL("../package.json", import.meta.url))).version;
test("published body matches the offline in-app release notes", () => {
  const body = formatReleaseNotes(manifest, version);
  for (const change of manifest.releases[0].changes) assert.ok(body.includes(change.title) && body.includes(change.description));
  assert.ok(body.includes("گواهی ناشر ویندوز ندارد"));
});
test("missing/duplicate versions and unsafe output are rejected", () => {
  assert.throws(() => formatReleaseNotes(manifest, "99.0.0"));
  assert.throws(() => formatReleaseNotes({ ...manifest, schemaVersion: 2 }, version));
  assert.throws(() => formatReleaseNotes({ ...manifest, releases: [...manifest.releases, ...manifest.releases] }, version));
  const invalid = structuredClone(manifest);
  invalid.releases[0].summary = "injected\nbody=value";
  assert.throws(() => formatReleaseNotes(invalid, version));
});
