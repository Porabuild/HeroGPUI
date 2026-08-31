// Character-level Rust source scanner shared by the extraction scripts.
//
// Everything here tracks the same lexer state (string / char / raw string /
// line comment / block comment) so that bracket balancing cannot be fooled by
// quotes, commas or braces inside literals — reference_metadata.rs strings
// contain escaped quotes and components.rs strings contain braces.

const IDENT_START = /[A-Za-z_]/;
const IDENT_CHAR = /[A-Za-z0-9_]/;

function isIdentChar(c) {
  return c !== undefined && IDENT_CHAR.test(c);
}

/// Skip whitespace and comments. Returns the index of the next significant
/// character.
export function skipTrivia(src, i) {
  while (i < src.length) {
    const c = src[i];
    if (c === " " || c === "\t" || c === "\n" || c === "\r") {
      i += 1;
    } else if (c === "/" && src[i + 1] === "/") {
      i += 2;
      while (i < src.length && src[i] !== "\n") i += 1;
    } else if (c === "/" && src[i + 1] === "*") {
      i += 2;
      while (i < src.length && !(src[i] === "*" && src[i + 1] === "/")) i += 1;
      i = Math.min(i + 2, src.length);
    } else {
      break;
    }
  }
  return i;
}

/// Unescape the body of a Rust string literal (without the quotes). Handles
/// `\<newline>` continuation, which also swallows the next line's leading
/// whitespace, like the compiler does.
export function unescapeRustString(body) {
  let out = "";
  let i = 0;
  while (i < body.length) {
    const c = body[i];
    if (c !== "\\") {
      out += c;
      i += 1;
      continue;
    }
    const n = body[i + 1];
    i += 2;
    switch (n) {
      case "n":
        out += "\n";
        break;
      case "r":
        out += "\r";
        break;
      case "t":
        out += "\t";
        break;
      case "0":
        out += "\0";
        break;
      case "\\":
        out += "\\";
        break;
      case '"':
        out += '"';
        break;
      case "'":
        out += "'";
        break;
      case "x": {
        out += String.fromCharCode(parseInt(body.slice(i, i + 2), 16));
        i += 2;
        break;
      }
      case "u": {
        // \u{...}
        const close = body.indexOf("}", i);
        if (close !== -1) {
          out += String.fromCodePoint(parseInt(body.slice(i + 1, close), 16));
          i = close + 1;
        }
        break;
      }
      case "\n": {
        // Line continuation: strip the newline and the next line's leading
        // whitespace.
        while (i < body.length && (body[i] === " " || body[i] === "\t")) i += 1;
        if (body[i] === "\r") i += 1;
        break;
      }
      case "\r": {
        while (i < body.length && (body[i] === " " || body[i] === "\t")) i += 1;
        break;
      }
      default:
        out += n ?? "";
        break;
    }
  }
  return out;
}

/// Read a plain string literal whose opening `"` is at `i`. Returns
/// `{ value, end }` (end is past the closing quote) or null.
export function readStringLiteral(src, i) {
  if (src[i] !== '"') return null;
  let j = i + 1;
  let body = "";
  while (j < src.length) {
    const c = src[j];
    if (c === "\\") {
      body += c + (src[j + 1] ?? "");
      j += 2;
      // \u{...} spans more characters; the \u case only needs the braces to
      // survive unescaping, so no extra handling is required here.
      continue;
    }
    if (c === '"') {
      return { value: unescapeRustString(body), end: j + 1 };
    }
    body += c;
    j += 1;
  }
  return null;
}

/// Read a raw string literal (`r"…"`, `r#"…"#`, …) starting at `i`.
/// Returns `{ value, end }` or null.
export function readRawStringLiteral(src, i) {
  if (src[i] !== "r") return null;
  // Only a raw-string start when the preceding character cannot end an
  // identifier (`variant"…` is not `r` + string).
  if (i > 0 && (IDENT_CHAR.test(src[i - 1]) || IDENT_START.test(src[i - 1]))) {
    return null;
  }
  let j = i + 1;
  let hashes = 0;
  while (src[j] === "#") {
    hashes += 1;
    j += 1;
  }
  if (src[j] !== '"') return null;
  const terminator = '"' + "#".repeat(hashes);
  const close = src.indexOf(terminator, j + 1);
  if (close === -1) return null;
  return { value: src.slice(j + 1, close), end: close + terminator.length };
}

/// Read a char literal starting at `'`. Returns `{ value, end }`, or null if
/// the quote opens something else (a lifetime such as `'static`), in which
/// case the caller should treat `i` as a single insignificant character.
export function readCharLiteral(src, i) {
  if (src[i] !== "'") return null;
  if (src[i + 1] === "\\") {
    // Escape: \n, \', \\, \x41, \u{1F600}, …
    let j = i + 2;
    const e = src[j];
    if (e === "x") {
      j += 3; // x + 2 hex digits
    } else if (e === "u") {
      const close = src.indexOf("}", j);
      if (close === -1) return null;
      j = close + 1;
    } else {
      j += 1;
    }
    if (src[j] !== "'") return null;
    return { value: unescapeRustString(src.slice(i + 1, j)), end: j + 1 };
  }
  if (src[i + 1] !== undefined && src[i + 1] !== "'" && src[i + 2] === "'") {
    return { value: src[i + 1], end: i + 3 };
  }
  return null;
}

/// Advance past one character that may open a literal or comment. Used inside
/// balanced scans. Returns the next index, or null when the character is
/// ordinary (the caller must still handle brackets itself).
export function stepOver(src, i) {
  const c = src[i];
  if (c === '"') {
    const s = readStringLiteral(src, i);
    if (s) return s.end;
    return i + 1;
  }
  if (c === "r") {
    const r = readRawStringLiteral(src, i);
    if (r) return r.end;
    return i + 1;
  }
  if (c === "'") {
    const ch = readCharLiteral(src, i);
    if (ch) return ch.end;
    // Lifetime or label: the apostrophe itself is insignificant.
    return i + 1;
  }
  if (c === "/" && src[i + 1] === "/") {
    while (i < src.length && src[i] !== "\n") i += 1;
    return i;
  }
  if (c === "/" && src[i + 1] === "*") {
    i += 2;
    while (i < src.length && !(src[i] === "*" && src[i + 1] === "/")) i += 1;
    return Math.min(i + 2, src.length);
  }
  return null;
}

/// Scan a balanced, delimited region whose opening delimiter is at `i`.
/// Returns `{ start, end, depthOneRanges }` where `end` is past the matching
/// close, or null if the region never closes. `depthOneRanges` collects
/// `{ start, end }` spans of the top-level `(...)`, `[...]` and `{...}`
/// groups directly inside the region — used to split tuple/array elements.
export function scanDelimited(src, i) {
  const open = src[i];
  const close = { "(": ")", "[": "]", "{": "}" }[open];
  if (!close) return null;
  const depthOneRanges = [];
  let depth = 1;
  let j = i + 1;
  const groupStarts = [];
  while (j < src.length) {
    const stepped = stepOver(src, j);
    if (stepped !== null) {
      j = stepped;
      continue;
    }
    const c = src[j];
    if (c === "(" || c === "[" || c === "{") {
      if (depth === 1) groupStarts.push(j);
      depth += 1;
      j += 1;
      continue;
    }
    if (c === ")" || c === "]" || c === "}") {
      depth -= 1;
      if (depth === 0) {
        return { start: i, end: j + 1, depthOneRanges };
      }
      if (depth === 1 && groupStarts.length) {
        depthOneRanges.push({ start: groupStarts.pop(), end: j + 1 });
      }
      j += 1;
      continue;
    }
    j += 1;
  }
  return null;
}

/// Read a Rust identifier at `i`. Returns `{ name, end }` or null.
export function readIdent(src, i) {
  if (!IDENT_START.test(src[i] ?? "")) return null;
  let j = i + 1;
  while (j < src.length && IDENT_CHAR.test(src[j])) j += 1;
  return { name: src.slice(i, j), end: j };
}

/// Split the region between `start` and `end` on top-level commas (depth 0
/// relative to the region), returning the element ranges.
export function splitTopLevelCommas(src, start, end) {
  const parts = [];
  let j = skipTrivia(src, start);
  let elemStart = j;
  while (j < end) {
    const stepped = stepOver(src, j);
    if (stepped !== null) {
      j = stepped;
      continue;
    }
    if (src[j] === "(" || src[j] === "[" || src[j] === "{") {
      const d = scanDelimited(src, j);
      if (d) {
        j = d.end;
        continue;
      }
    }
    if (src[j] === ",") {
      parts.push({ start: elemStart, end: j });
      j = skipTrivia(src, j + 1);
      elemStart = j;
      continue;
    }
    j += 1;
  }
  if (elemStart < end && src.slice(elemStart, end).trim() !== "") {
    parts.push({ start: elemStart, end });
  }
  return parts;
}

/// kebab-case slug used for every page/component key in the generated data.
/// "Button Group" -> "button-group", "InputOTP" -> "input-otp",
/// "Label & Messages" -> "label-messages".
export function slugify(title) {
  return title
    .replace(/&/g, " ")
    .replace(/([a-z0-9])([A-Z])/g, "$1-$2")
    .replace(/([A-Z]+)([A-Z][a-z])/g, "$1-$2")
    .replace(/[^A-Za-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .toLowerCase();
}
