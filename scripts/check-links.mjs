// Doc link check (TCK-197): fails when a wiki article, skill or open/ongoing ticket points to a missing file.
// Checked: wikilinks [[path]] / [[path|alias]] / [[path#heading]] (vault-root relative, outside code), frontmatter
// `sources:` entries, and (wiki and skills only) backtick-quoted repo paths to Markdown files under wiki/, tickets/
// and .agents/.
// Usage: node scripts/check-links.mjs   (run from the repo root; exit code 1 on broken links)
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative, sep } from "node:path";

const root = process.cwd();
const toRepoPath = (abs) => relative(root, abs).split(sep).join("/");

const markdownFiles = (dir, recursive) => {
  const abs = join(root, dir);
  if (!existsSync(abs)) return [];
  return readdirSync(abs).flatMap((name) => {
    const path = join(abs, name);
    if (statSync(path).isDirectory()) return recursive ? markdownFiles(toRepoPath(path), true) : [];
    return name.endsWith(".md") ? [toRepoPath(path)] : [];
  });
};

const files = [
  ...markdownFiles("wiki", true),
  ...markdownFiles(".agents/skills", true),
  ...markdownFiles("tickets/open", false),
  ...markdownFiles("tickets/ongoing", false),
];

const exists = (target) => existsSync(join(root, target));
const wikiTargetExists = (target) => exists(`${target}.md`) || exists(target);

// Backtick paths are checked in the wiki and skills only: open tickets name files they are going to create.
// `.agents/state/` handover notes are optional by design (AGENTS.md) and exempt.
const checksCodePaths = (file) => file.startsWith("wiki/") || file.startsWith(".agents/skills/");

const broken = [];
const report = (file, line, kind, target) => broken.push(`${file}:${line}: ${kind} -> ${target}`);

for (const file of files) {
  const lines = readFileSync(join(root, file), "utf8").split(/\r?\n/);
  let inFence = false;
  let inFrontmatter = lines[0] === "---";
  lines.forEach((text, i) => {
    const line = i + 1;
    if (inFrontmatter) {
      if (i > 0 && text === "---") inFrontmatter = false;
      const sources = text.match(/^sources:\s*\[(.*)\]\s*$/);
      if (sources) {
        for (const entry of sources[1].split(",").map((s) => s.trim().replace(/^["']|["']$/g, "")).filter(Boolean)) {
          if (!exists(entry)) report(file, line, "sources", entry);
        }
      }
      return;
    }
    if (/^\s*(```|~~~)/.test(text)) {
      inFence = !inFence;
      return;
    }
    if (inFence) return;

    if (checksCodePaths(file)) {
      for (const [, code] of text.matchAll(/`([^`]+)`/g)) {
        const path = code.trim();
        if (/^(wiki|tickets|\.agents)\/[\w./-]+\.md$/.test(path) && !path.startsWith(".agents/state/") && !exists(path)) {
          report(file, line, "path", path);
        }
      }
    }
    const prose = text.replace(/`[^`]*`/g, "");
    for (const [, inner] of prose.matchAll(/\[\[([^\]]+)\]\]/g)) {
      const target = inner.split("|")[0].split("#")[0].trim();
      if (target && !wikiTargetExists(target)) report(file, line, "wikilink", target);
    }
  });
}

if (broken.length > 0) {
  console.error(`Broken doc links (${broken.length}) in ${files.length} files:`);
  broken.forEach((b) => console.error(`  ${b}`));
  process.exit(1);
}
console.log(`Doc links OK (${files.length} files checked).`);
