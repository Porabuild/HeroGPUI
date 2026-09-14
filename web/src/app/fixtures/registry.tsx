"use client";

import { Button, Label, ListBox, ListBoxItem, Modal, Select } from "@heroui/react";
import type { ReactNode } from "react";

/**
 * Dev-only matched fixtures for upstream/port parity captures
 * (docs/parity/fixtures.md, plan section 5.1). Every entry mirrors one keyed
 * specimen from the native gallery so a capture of the pinned HeroUI packages
 * and a capture of the port show the same content, in the same stage frame the
 * gallery's `preview_wrapper` uses: the composition centered on the background
 * token, nothing else.
 *
 * The fixtures are deliberately static and uncontrolled wherever the gallery
 * specimen is: no timers, no randomness, no network, no moving dates. An
 * interaction capture drives real input against these components; a resting
 * capture must be reproducible byte-for-byte.
 */

/**
 * One registry entry. `stageWidth`/`stageHeight` override the default stage
 * size when a composition needs more room than 640x360 (the open Modal).
 */
export interface FixtureEntry {
  id: string;
  label: string;
  stageWidth?: number;
  stageHeight?: number;
  element: ReactNode;
}

/**
 * The fixed collection behind `language_items()` (gallery/src/pages/components/
 * mod.rs): the slug is the stable key, the display name the label. Mirrored
 * verbatim so Select captures line up option-for-option.
 */
const LANGUAGES = [
  { id: "rust", label: "Rust" },
  { id: "typescript", label: "TypeScript" },
  { id: "python", label: "Python" },
  { id: "go", label: "Go" },
  { id: "swift", label: "Swift" },
  { id: "kotlin", label: "Kotlin" },
] as const;

/**
 * Mirrors `btn-usage` (gallery/src/pages/components/buttons.rs): one default
 * (Primary) Button labelled "Click me" in a centered specimen row. The port
 * example sets `radius(px(8.))` and squares the top-left corner through `sx`;
 * the React Button has no radius prop (only variant/size/fullWidth/isIconOnly,
 * per the installed button.d.ts), so the same geometry enters through inline
 * style. Left square, three round: that asymmetry is the specimen's point.
 */
function ButtonUsageFixture() {
  return (
    <div className="flex flex-wrap items-center justify-center gap-3">
      <Button style={{ borderRadius: "8px", borderTopLeftRadius: 0 }}>Click me</Button>
    </div>
  );
}

/**
 * Mirrors `sel-main`, the Select Usage specimen (gallery/src/pages/components/
 * pickers.rs), at rest: label "Language", placeholder "Choose one", the
 * language collection, closed. The gallery seeds `select_lang: None`, so the
 * resting composition shows the placeholder, not a selection — the fixture
 * stays uncontrolled for the same reason, and an interaction capture supplies
 * the selection with real input. The 256px column is the gallery's
 * `field_col`/`DEMO_FIELD_W` frame (v3's own `w-64` demo width).
 */
function SelectUsageFixture() {
  return (
    <div className="flex w-64 flex-col gap-3">
      <Select placeholder="Choose one">
        <Label>Language</Label>
        <Select.Trigger>
          <Select.Value />
          <Select.Indicator />
        </Select.Trigger>
        <Select.Popover>
          <ListBox>
            {LANGUAGES.map((language) => (
              <ListBoxItem key={language.id} id={language.id}>
                {language.label}
              </ListBoxItem>
            ))}
          </ListBox>
        </Select.Popover>
      </Select>
    </div>
  );
}

/**
 * Mirrors the Modal gallery's `md-size-md` specimen (gallery/src/pages/
 * components/overlays.rs, "Sizes" section) in its opened state — the state the
 * port reaches through `HEROGPUI_OPEN_OVERLAYS`. Same title ("Size: Md"), the
 * close trigger, and the same one-line panel body, rendered open on load with
 * no trigger button because the comparison target is the open panel itself.
 *
 * The overlay is viewport-fixed and portals to document.body, so it is not
 * clipped by the stage; docs/parity/fixtures.md documents the matching capture
 * frame. `isDismissable` is spelled out even though it defaults to true —
 * the fixture's contract should be readable without checking the package.
 */
function ModalOpenFixture() {
  return (
    <Modal isOpen>
      <Modal.Backdrop isDismissable>
        <Modal.Container size="md">
          <Modal.Dialog>
            <Modal.Header>
              <Modal.Heading>Size: Md</Modal.Heading>
              <Modal.CloseTrigger />
            </Modal.Header>
            <Modal.Body>Every size shares one panel style.</Modal.Body>
          </Modal.Dialog>
        </Modal.Container>
      </Modal.Backdrop>
    </Modal>
  );
}

/**
 * The fixture registry. Ids and stage overrides are mirrored in `FIXTURES` in
 * page.tsx (a server component cannot read across the client boundary); keep
 * the two lists in step when adding a fixture.
 */
export const fixtures: FixtureEntry[] = [
  { id: "button-usage", label: "Button Usage", element: <ButtonUsageFixture /> },
  { id: "select-usage", label: "Select Usage", element: <SelectUsageFixture /> },
  {
    id: "modal-open",
    label: "Modal Sizes — Md, open",
    stageWidth: 720,
    stageHeight: 480,
    element: <ModalOpenFixture />,
  },
];

/**
 * Renders one registry entry by id. page.tsx validates the id before rendering,
 * so an unknown id renders nothing rather than inventing content.
 */
export function FixtureStage({ id }: { id: string }) {
  const fixture = fixtures.find((entry) => entry.id === id);
  if (!fixture) {
    return null;
  }
  return fixture.element;
}
