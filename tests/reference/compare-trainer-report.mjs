// Compares a headless trainer report against the committed reference (TCK-186).
// Ignores `perf` (timing) and `meta.version` (bumped every release); everything else must match exactly.
// Usage: node tests/reference/compare-trainer-report.mjs <reference.json> <actual.json>
import { readFileSync } from "node:fs";

const [referencePath, actualPath] = process.argv.slice(2);
if (!referencePath || !actualPath) {
  console.error("usage: compare-trainer-report.mjs <reference.json> <actual.json>");
  process.exit(2);
}

const normalize = (path) => {
  const report = JSON.parse(readFileSync(path, "utf8"));
  delete report.perf;
  delete report.meta.version;
  return report;
};

const reference = normalize(referencePath);
const actual = normalize(actualPath);

const diffs = [];
const walk = (a, b, at) => {
  if (typeof a !== typeof b || Array.isArray(a) !== Array.isArray(b) || (a === null) !== (b === null)) {
    diffs.push(`${at}: ${JSON.stringify(a)} != ${JSON.stringify(b)}`);
  } else if (a !== null && typeof a === "object") {
    for (const key of new Set([...Object.keys(a), ...Object.keys(b)])) walk(a[key], b[key], `${at}.${key}`);
  } else if (a !== b) {
    diffs.push(`${at}: ${JSON.stringify(a)} != ${JSON.stringify(b)}`);
  }
};
walk(reference, actual, "report");

if (diffs.length > 0) {
  console.error(`Trainer report differs from the reference (${diffs.length} values). First differences:`);
  diffs.slice(0, 20).forEach((d) => console.error(`  ${d}`));
  console.error("An intended model change updates tests/reference/trainer-ci.json in the same commit (see TCK-186).");
  process.exit(1);
}
console.log("Trainer report matches the reference.");
