import type { RustExample } from "./data";

export interface ExampleSection {
  /** Stable example key and anchor id. */
  id: string;
  /** Section heading, also the table-of-contents entry. */
  heading: string;
  /** HeroGPUI gallery snippet shown in the example card. */
  rust: RustExample;
}

function normalize(value: string): string {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim();
}

export function buildExampleSections(rustExamples: RustExample[]): ExampleSection[] {
  const anchorIds = new Set<string>();

  const anchor = (base: string): string => {
    let id = base;
    let counter = 2;
    while (anchorIds.has(id)) {
      id = `${base}-${counter}`;
      counter += 1;
    }
    anchorIds.add(id);
    return id;
  };

  return rustExamples.map((example, index) => {
    const key = normalize(example.heading);
    const base = key.replace(/\s+/g, "-") || `example-${index + 1}`;
    return {
      id: anchor(`rust-${base}`),
      heading: example.heading,
      rust: example,
    };
  });
}
