import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { runInNewContext } from "node:vm";

// Run each real bootstrap's event registration without starting WASM. The
// canvas listener represents GPUI's wheel input boundary: a field steps once
// per event, while a scroller consumes the event's distance and hit-test point.
function wheelHarness(htmlPath) {
  const html = readFileSync(new URL(htmlPath, import.meta.url), "utf8");
  const modules = [...html.matchAll(/<script type="module">([\s\S]*?)<\/script>/g)];
  assert.equal(modules.length, 1);
  const source = modules[0][1];
  assert.match(source, /\n\s*boot\(\);\s*$/);
  const delivered = [];
  const captures = [];
  const frames = new Map();
  let frameId = 0;

  class WheelEvent {
    static DOM_DELTA_PIXEL = 0;
    static DOM_DELTA_LINE = 1;

    constructor(type, options) {
      Object.assign(this, { type, isTrusted: false, ...options });
    }

    preventDefault() {
      this.defaultPrevented = true;
    }

    stopImmediatePropagation() {
      this.propagationStopped = true;
    }
  }

  const canvas = {
    dispatchEvent(event) {
      event.target = canvas;
      for (const capture of captures) {
        capture(event);
        if (event.propagationStopped) return;
      }
      delivered.push(event);
    },
  };

  runInNewContext(source.replace(/\n\s*boot\(\);\s*$/, ""), {
    WheelEvent,
    document: {
      querySelector: (selector) => (selector === "canvas" ? canvas : null),
      addEventListener(type, callback) {
        if (type === "wheel") captures.push(callback);
      },
    },
    window: { addEventListener() {} },
    requestAnimationFrame(callback) {
      frames.set(++frameId, callback);
      return frameId;
    },
    cancelAnimationFrame: (id) => frames.delete(id),
  });

  return (options) => {
    const event = new WheelEvent("wheel", { isTrusted: true, ...options });
    canvas.dispatchEvent(event);
    let frame = 0;
    while (frames.size) {
      assert.ok(++frame < 100, "wheel animation must settle");
      const [id, callback] = frames.entries().next().value;
      frames.delete(id);
      callback(frame * 16);
    }
    assert.equal(delivered.length, 1, "one wheel action must remain one field step");
    assert.equal(delivered[0], event, "GPUI must receive the original trusted wheel event");
    for (const [key, value] of Object.entries(options)) {
      assert.equal(delivered[0][key], value, `${key} must reach GPUI unchanged`);
    }
  };
}

for (const html of ["../public/gallery/index.html", "../../crates/herogpui-web/index.html"]) {
  test(`${html} preserves wheel count, coordinates, axes, units and modifiers`, () => {
    for (const input of [
      { deltaX: 0, deltaY: 120, deltaMode: 0 },
      { deltaX: 0, deltaY: -3, deltaMode: 1 },
      { deltaX: 73, deltaY: -120, deltaMode: 0 },
      { deltaX: 2, deltaY: -3, deltaMode: 0 },
      { deltaX: 0, deltaY: 120, deltaMode: 0, shiftKey: true },
      { deltaX: 0, deltaY: 120, deltaMode: 0, ctrlKey: true, altKey: true, metaKey: true },
    ]) {
      wheelHarness(html)({
        clientX: 740,
        clientY: 510,
        screenX: 880,
        screenY: 620,
        shiftKey: false,
        ctrlKey: false,
        altKey: false,
        metaKey: false,
        ...input,
      });
    }
  });

  test(`${html} carries the artifact version to the glue and WASM URLs`, () => {
    const source = readFileSync(new URL(html, import.meta.url), "utf8");
    const assetFunction = source.match(/function assetUrl\(name\) \{[\s\S]*?\n      \}/)?.[0];
    assert.ok(assetFunction, "the bootstrap must expose its asset URL helper");

    const assetUrl = (href, name) =>
      runInNewContext(`${assetFunction}\nassetUrl(name);`, {
        URL,
        URLSearchParams,
        name,
        window: { location: { href, search: new URL(href).search } },
      });

    assert.equal(
      assetUrl("http://127.0.0.1:8765/gallery/index.html?v=33d123", "herogpui_web.js"),
      "http://127.0.0.1:8765/gallery/herogpui_web.js?v=33d123",
    );
    assert.equal(
      assetUrl("http://127.0.0.1:8765/gallery/index.html?v=33d123", "herogpui_web_bg.wasm"),
      "http://127.0.0.1:8765/gallery/herogpui_web_bg.wasm?v=33d123",
    );
    assert.equal(
      assetUrl("http://127.0.0.1:8765/gallery/index.html", "herogpui_web.js"),
      "http://127.0.0.1:8765/gallery/herogpui_web.js",
    );
    assert.match(source, /await init\(\{ module_or_path: wasmUrl \}\)/);
  });
}
