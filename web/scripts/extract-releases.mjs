// Fetch the published GitHub Releases into web/src/data/releases.json for the
// /docs/releases page.
//
// Source: the GitHub REST API for the public Porabuild/HeroGPUI repository.
// Drafts are skipped (they are not public releases); prereleases are kept and
// flagged. Bodies are GitHub-flavoured markdown and are stored verbatim — the
// page renders them.
//
// Auth: when GITHUB_TOKEN is set it is sent as a Bearer token to raise the
// rate limit. The token is never printed.
//
// Usage:
//   node scripts/extract-releases.mjs          # fetch and write releases.json
//   node scripts/extract-releases.mjs --check  # fetch, diff against the
//                                              # checked-in file, exit 1 on
//                                              # drift. A network or rate-limit
//                                              # failure in --check is a skip
//                                              # (exit 0) so offline CI stays
//                                              # green; a 404 still exits 1,
//                                              # because a repository that no
//                                              # longer resolves is drift.

import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(scriptDir, "..");
const OUT = resolve(webRoot, "src", "data", "releases.json");

const OWNER = "Porabuild";
const REPO = "HeroGPUI";
const API = `https://api.github.com/repos/${OWNER}/${REPO}/releases?per_page=100`;

function asString(value, fallback = "") {
  return typeof value === "string" ? value : fallback;
}

/// One page of the releases API. Follows `rel="next"` links so repositories
/// with more than one page of releases stay complete.
async function fetchPage(url, headers) {
  const res = await fetch(url, { headers });
  if (res.status === 404) {
    // Not a transport failure: the configured repository is gone, renamed or
    // private, so `--check` must fail rather than skip. Skipping would let the
    // site keep serving releases from a name that no longer resolves.
    const error = new Error(
      `GitHub API returned 404 for ${OWNER}/${REPO}; is the repository name correct?`,
    );
    error.fatal = true;
    throw error;
  }
  if (res.status === 403 && res.headers.get("x-ratelimit-remaining") === "0") {
    const reset = res.headers.get("x-ratelimit-reset");
    const when = reset ? new Date(Number(reset) * 1000).toISOString() : "unknown";
    throw new Error(
      `GitHub API rate limit exceeded (resets at ${when}); set GITHUB_TOKEN and retry.`,
    );
  }
  if (!res.ok) {
    throw new Error(`GitHub API request failed: ${res.status} ${res.statusText}`);
  }
  const data = await res.json();
  if (!Array.isArray(data)) throw new Error("GitHub API returned an unexpected payload.");
  const link = res.headers.get("link") ?? "";
  const next = link.match(/<([^>]+)>;\s*rel="next"/)?.[1] ?? null;
  return { data, next };
}

async function fetchAllReleases() {
  const headers = {
    Accept: "application/vnd.github+json",
    "User-Agent": "HeroGPUI-web-extract-releases",
    "X-GitHub-Api-Version": "2022-11-28",
  };
  if (process.env.GITHUB_TOKEN) headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;

  const all = [];
  let url = API;
  while (url) {
    const { data, next } = await fetchPage(url, headers);
    all.push(...data);
    url = next;
  }
  return all;
}

function toRelease(entry) {
  return {
    tag: asString(entry.tag_name),
    name: asString(entry.name, asString(entry.tag_name)),
    publishedAt: asString(entry.published_at),
    prerelease: entry.prerelease === true,
    url: asString(entry.html_url),
    body: asString(entry.body),
  };
}

export async function run({ check = false } = {}) {
  let entries;
  try {
    entries = await fetchAllReleases();
  } catch (error) {
    if (check && !error.fatal) {
      console.log(`extract-releases --check: skipped (${error.message})`);
      return { skipped: true };
    }
    console.error(`ERROR: ${error.message}`);
    process.exitCode = 1;
    return { failed: true };
  }

  const releases = entries
    .filter((entry) => entry.draft !== true && typeof entry.tag_name === "string")
    .map(toRelease)
    // Newest first. Ties break on the tag so the file is a function of the
    // release set alone -- a comparator that never returns 0 would let two
    // releases published in the same second reorder with the API's order and
    // fail --check for no reason.
    .sort((a, b) => b.publishedAt.localeCompare(a.publishedAt) || a.tag.localeCompare(b.tag));

  const output =
    JSON.stringify(
      {
        generatedAt: new Date().toISOString(),
        repository: `${OWNER}/${REPO}`,
        releases,
      },
      null,
      2,
    ) + "\n";

  if (check) {
    let current = "";
    try {
      current = readFileSync(OUT, "utf8");
    } catch {}
    // generatedAt changes on every run; compare the payload, not the stamp.
    const same = (() => {
      try {
        const a = JSON.parse(current);
        const b = JSON.parse(output);
        return (
          JSON.stringify({ repository: a.repository, releases: a.releases }) ===
          JSON.stringify({ repository: b.repository, releases: b.releases })
        );
      } catch {
        return false;
      }
    })();
    if (!same) {
      console.error("ERROR: releases.json is stale; run `node scripts/extract-releases.mjs`");
      process.exitCode = 1;
    } else {
      console.log(`releases.json: current (${releases.length} releases)`);
    }
    return { releases: releases.length };
  }

  mkdirSync(dirname(OUT), { recursive: true });
  writeFileSync(OUT, output);
  console.log(`releases.json: ${releases.length} releases from ${OWNER}/${REPO}`);
  return { releases: releases.length };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  await run({ check: process.argv.includes("--check") });
}
