import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
const css = readFileSync(new URL("../src/apple-interface.css", import.meta.url), "utf8");

function tokens(theme) {
  const block = css.split(`.shell[data-appearance="${theme}"] {`)[1].split("}")[0];
  return Object.fromEntries([...block.matchAll(/(--[\w-]+):\s*(#[0-9a-f]{3,6});/gi)].map(m => [m[1], m[2]]));
}
function luminance(hex) {
  assert.ok(hex, "A required semantic theme token is missing");
  const rgb = hex.slice(1).length === 3 ? hex.slice(1).split("").map(c => c + c).join("") : hex.slice(1);
  const values = [0, 2, 4].map(i => parseInt(rgb.slice(i, i + 2), 16) / 255).map(v => v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4);
  return values[0] * .2126 + values[1] * .7152 + values[2] * .0722;
}
function contrast(a, b) {
  const values = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (values[0] + .05) / (values[1] + .05);
}
describe("theme legibility", () => {
  for (const theme of ["light", "dark"]) {
    it(`${theme}: small text and semantic messages have readable opaque surfaces`, () => {
      const t = tokens(theme);
      for (const fg of ["--text", "--muted", "--faint"]) {
        for (const bg of ["--bg", "--surface", "--surface-inset"]) {
          assert.ok(contrast(t[fg], t[bg]) >= 4.5, `${fg} on ${bg}`);
        }
      }
      for (const state of ["primary", "success", "warning", "danger"]) {
        assert.ok(contrast(t[`--${state}`], t[`--${state}-soft`]) >= 4.5, state);
      }
      assert.ok(contrast(t["--control-border"], t["--surface-inset"]) >= 3, "input boundary");
      assert.ok(contrast(t["--action-text"], t["--action-bg"]) >= 4.5, "primary action label");
    });
  }
});
