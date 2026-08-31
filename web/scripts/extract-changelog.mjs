// Extract the git history into web/src/data/changelog.json for the
// /docs/releases page.
//
// HeroGPUI has never been released — no git tags, and the workspace version
// (read from [workspace.package] in the root Cargo.toml) is still 0.1.0 — so
// this is a *development* log, not a release history. The page renders it as
// one; `released` is derived from whether any `v*` tag exists, so the JSON
// starts telling the truth the day the first tag lands.
//
// Privacy: author emails are never read, let alone emitted. Author *names* are
// kept (changelog convention) unless a name is itself an email address, in
// which case it is dropped.
//
// Commit kinds are classified from the subject's leading verb only. Subjects
// here are imperative sentences ("Complete Select's multiple collection
// contract"), not conventional commits, so the verb is the only cheap signal:
//
//   fix     — repairs a defect:              Fix, Repair, Restore, Recover, Stop
//   docs    — documentation:                 Document, a README/AGENTS.md
//                                            subject, or "doc comment" in it
//   infra   — tooling, packaging, toolchain: Test, Publish, Prepare, Require,
//                                            Harden, Exclude
//   parity  — measured-parity and contract
//           work against upstream v3:        Complete, Match, Align, Replace,
//                                            Remove, Bring, Port, Audit,
//                                            Measure, Take, Validate, Compare,
//                                            Account, Read
//   feature — everything else (the default):
//             most of this history is new
//             component behaviour
//
// The rules are deliberately simple; a chip on the page is a hint, not an
// audit, and a rare mislabel ("Build HeroGPUI with Rust 1.98" reads as a
// feature) costs nothing.

import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(scriptDir, "..");
const repoRoot = resolve(webRoot, "..");
const CARGO_TOML = join(repoRoot, "Cargo.toml");
const OUT = resolve(webRoot, "src", "data", "changelog.json");

// %x1f (unit separator) between fields: subjects containing a pipe cannot
// break the record shape. --date=short gives plain YYYY-MM-DD.
const LOG_FORMAT = "--pretty=format:%H%x1f%h%x1f%ad%x1f%an%x1f%s";

const FIX_VERBS = new Set(["Fix", "Repair", "Restore", "Recover", "Stop"]);
const INFRA_VERBS = new Set(["Test", "Publish", "Prepare", "Require", "Harden", "Exclude"]);
const PARITY_VERBS = new Set([
  "Complete",
  "Match",
  "Align",
  "Replace",
  "Remove",
  "Bring",
  "Port",
  "Audit",
  "Measure",
  "Take",
  "Validate",
  "Compare",
  "Account",
  "Read",
]);

function classify(subject) {
  const first = subject.split(" ", 1)[0];
  if (FIX_VERBS.has(first)) return "fix";
  if (INFRA_VERBS.has(first)) return "infra";
  if (PARITY_VERBS.has(first)) return "parity";
  if (
    first === "Document" ||
    subject.includes("README") ||
    first === "AGENTS.md:" ||
    subject.includes("doc comment")
  ) {
    return "docs";
  }
  return "feature";
}

// Author names that are email addresses are personal data; drop just the name.
// The commits themselves are never dropped.
function authorName(raw) {
  if (raw.includes("@")) return null;
  return raw;
}

// The single version every release artifact will share lives in
// [workspace.package]; scan that table rather than trusting the first
// `version =` anywhere in the file.
function workspaceVersion() {
  const lines = readFileSync(CARGO_TOML, "utf8").split(/\r?\n/);
  let inPackageTable = false;
  for (const line of lines) {
    if (line.startsWith("[")) inPackageTable = line === "[workspace.package]";
    if (inPackageTable) {
      const match = line.match(/^version\s*=\s*"([^"]+)"/);
      if (match) return match[1];
    }
  }
  throw new Error("no `version` under [workspace.package] in Cargo.toml");
}

function git(args) {
  // The deployed site never runs this (changelog.json is committed); it reads
  // the repository this script is checked out in.
  return execFileSync("git", args, {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
}

export function run() {
  const version = workspaceVersion();
  const released = git(["tag", "--list", "v*"]).trim().length > 0;

  const commits = git(["log", LOG_FORMAT, "--date=short"])
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => {
      const [fullSha, shortSha, date, author, subject] = line.split("\x1f");
      if (!fullSha || !shortSha || !date || author === undefined || !subject) {
        throw new Error(`malformed git log line: ${JSON.stringify(line)}`);
      }
      return { sha: shortSha, date, author: authorName(author), subject };
    });

  // Group by day, newest first; within a day git log is already newest first.
  const byDate = new Map();
  for (const commit of commits) {
    if (!byDate.has(commit.date)) byDate.set(commit.date, []);
    byDate.get(commit.date).push({
      sha: commit.sha,
      author: commit.author,
      subject: commit.subject,
      kind: classify(commit.subject),
    });
  }
  const days = [...byDate.entries()]
    .sort((a, b) => (a[0] < b[0] ? 1 : -1))
    .map(([date, dayCommits]) => ({ date, commits: dayCommits }));

  const changelog = {
    generatedAt: new Date().toISOString().slice(0, 10),
    version,
    released,
    commitCount: commits.length,
    days,
  };

  mkdirSync(dirname(OUT), { recursive: true });
  writeFileSync(OUT, JSON.stringify(changelog, null, 2) + "\n");

  const kinds = {};
  for (const day of days) {
    for (const commit of day.commits) kinds[commit.kind] = (kinds[commit.kind] ?? 0) + 1;
  }
  console.log(
    `changelog.json: ${commits.length} commits across ${days.length} days ` +
      `(${days[0].date} → ${days[days.length - 1].date}), ` +
      `version ${version}, released: ${released}`,
  );
  console.log(
    `  kinds: ${Object.entries(kinds)
      .map(([k, n]) => `${k} ${n}`)
      .join(", ")}`,
  );
  const dropped = commits.filter((c) => c.author === null).length;
  if (dropped > 0) console.log(`  dropped ${dropped} author name(s) that were email addresses`);
  return changelog;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  run();
}
