// The development inspector's panel (crates/og-runtime/src/author/inspector):
// a TSX component mounted on the game's UI at the `system` layer while the
// inspector is open, as a game mounts its own (src/ui/inspector-host.ts). The
// runtime builds its values each frame; what its controls do goes back as
// events, done at the next frame. Only a development bundle imports this
// module (build.ts), which registers the component; a publish build has no
// inspector.

import { For, Input, Line, Scroll, Show, Text, View, type UiEvent } from "@origingame/engine/ui";
import { inspector } from "@origingame/engine/ui/rt";

interface Part { id: string; hint: string; text: string }
interface Field {
  id: string; name: string; mark: boolean; tone: "" | "flash" | "override"; doc: string;
  editor: "slider" | "check" | "pick" | "link" | "value" | "text";
  parts: Part[]; min: number; max: number; step: number; value: number; on: boolean;
  options: string[]; choice: string; text: string; swatch: string; flash: boolean;
}
interface Section { id: string; name: string; open: boolean; fields: Field[] }
interface Row { id: string; label: string; depth: number; fold: "open" | "closed" | "leaf"; selected: boolean; named: boolean; children: number }
interface UiLine { id: string; text: string; kind: "head" | "line" | "muted" }
interface Box { id: string; kind: "bound" | "margin" | "border" | "padding" | "content"; x: number; y: number; w: number; h: number; t: number; r: number; b: number; l: number }

/** What the runtime gives the panel each frame (panel/values.rs). */
export interface InspectorValues {
  key: string; keyTip: string; dirty: number; paused: boolean; picking: boolean;
  canUndo: boolean; canRedo: boolean; tab: "entities" | "resources" | "ui";
  filter: string; hint: string; tree: Row[]; note: string;
  head: { id: string; raw: string }; prefab: { what: string; none: boolean };
  overrides: { id: string; text: string }[]; sections: Section[];
  bounds: boolean; lines: UiLine[]; boxes: Box[];
  mode: "translate" | "rotate" | "scale"; space: string; clock: string; status: string;
}

/** One event back to the runtime (panel/events.rs). */
export interface InspectorEvent { kind: string; name?: string; row?: string; value?: unknown }

export interface InspectorProps {
  values: () => InspectorValues;
  send: (e: InspectorEvent) => void;
}

const TABS = [["entities", "Entities"], ["resources", "Resources"], ["ui", "UI"]] as const;

const BOX: Record<Box["kind"], string> = {
  bound: "border-danger",
  margin: "border-warn",
  border: "border-gold",
  padding: "border-success",
  content: "border-transparent bg-info opacity-45",
};

/** A toolbar button: quiet, or primary while `on`. */
function Tool(props: { label: string; on?: boolean | undefined; quiet?: boolean | undefined; disabled?: boolean | undefined; tip?: string | undefined; class?: string | undefined; onPress: () => void }) {
  return (
    <View
      role="button"
      label={props.label}
      description={props.tip}
      disabled={props.disabled}
      data-on={props.on}
      data-quiet={props.quiet}
      class={`flex-row items-center justify-center shrink-0 h-7 px-2.5 rounded-md text-sm text-fg bg-raised-lo border border-border hover:bg-raised-hover-lo active:bg-raised-pressed data-quiet:bg-transparent data-quiet:border-transparent data-quiet:hover:bg-surface-hover data-on:bg-accent data-on:border-accent data-on:text-accent-fg data-on:hover:bg-accent disabled:text-fg-subtle disabled:bg-well disabled:border-transparent ${props.class ?? ""}`}
      onPress={() => props.onPress()}
    >
      <Text class="whitespace-nowrap">{props.label}</Text>
    </View>
  );
}

/** A disclosure chevron, pointing down when open. */
function Chevron(props: { open: boolean }) {
  return (
    <Line
      class="size-4 shrink-0 stroke-fg-muted stroke-2"
      viewBox={[14, 14]}
      points={props.open ? [3.4, 5.3, 7, 9, 10.6, 5.3] : [5.3, 3.4, 9, 7, 5.3, 10.6]}
    />
  );
}

/** A check box and its label. */
function Check(props: { label: string; on: boolean; class?: string | undefined; onPress: () => void }) {
  return (
    <View role="checkbox" label={props.label} checked={props.on} class={`flex-row items-center gap-2 min-w-0 text-sm text-fg ${props.class ?? ""}`} onPress={() => props.onPress()}>
      <View data-on={props.on} class="size-4 shrink-0 rounded-sm border border-border-strong bg-well data-on:bg-accent data-on:border-transparent" />
      <Text class="truncate">{props.label}</Text>
    </View>
  );
}

/** The inspector's panel over the game: its tools, the hierarchy, the detail and the UI view's boxes. */
export function Inspector(props: InspectorProps) {
  const v = props.values;
  const send = props.send;
  const click = (name: string, row = "") => send({ kind: "click", name, row });
  const changed = (name: string, value: unknown, row = "") => send({ kind: "changed", name, row, value });
  // A text box: its text as typed, committed on Enter or when it loses the keyboard.
  const typing = (row: string) => ({
    onFocus: () => send({ kind: "focus" }),
    onBlur: () => { send({ kind: "commit", row }); send({ kind: "blur" }); },
    onKey: (e: UiEvent) => {
      if (e.key === "Enter") send({ kind: "commit", row });
      else if (e.key === "Escape") send({ kind: "cancel" });
    },
  });
  return (
    <View class="absolute inset-0 flex-row justify-end p-3 pointer-events-none">
      <View class="absolute inset-0">
        <For each={v().boxes} key={(b) => b.id}>
          {(b) => (
            <View
              class={`absolute ${BOX[b().kind]}`}
              style={{
                left: `${b().x}px`, top: `${b().y}px`, width: `${b().w}px`, height: `${b().h}px`,
                "border-top-width": `${b().t}px`, "border-right-width": `${b().r}px`, "border-bottom-width": `${b().b}px`, "border-left-width": `${b().l}px`,
              }}
            />
          )}
        </For>
      </View>
      <View
        ref={(node) => send({ kind: "panel", value: node.id })}
        role="dialog"
        label="Inspector"
        class="relative w-90 h-full flex-col gap-2 p-3 rounded-lg bg-surface border border-border shadow-panel text-fg pointer-events-auto"
      >
        <View class="flex-row items-center gap-1">
          <Text class="text-base font-semibold whitespace-nowrap">Inspector</Text>
          <Tool label={v().key} quiet tip={v().keyTip} onPress={() => click("Key")} />
          <View class="grow" />
          <Tool
            label={v().dirty > 0 ? `Save (${v().dirty})` : "Save"}
            on={v().dirty > 0}
            disabled={v().dirty === 0}
            tip="Write the edited fields into the scenes' text (save_scene)"
            onPress={() => click("Save")}
          />
          <Tool label="Close" quiet onPress={() => click("Close")} />
        </View>
        <View class="flex-row items-center gap-1">
          <Tool label={v().paused ? "Play" : "Pause"} onPress={() => click("Clock")} />
          <Tool label="Step" disabled={!v().paused} tip="Advance one tick while paused" onPress={() => click("Step")} />
          <Tool label={v().picking ? "Picking…" : "Pick"} on={v().picking} tip="Select what is under the next click in the game" onPress={() => click("Pick")} />
          <View class="grow" />
          <Tool label="Undo" disabled={!v().canUndo} tip="Undo the last edit (Ctrl+Z)" onPress={() => click("Undo")} />
          <Tool label="Redo" disabled={!v().canRedo} tip="Redo (Ctrl+Shift+Z)" onPress={() => click("Redo")} />
        </View>
        <View role="tablist" label="Inspector view" class="flex-row p-0.5 rounded-md bg-well">
          <For each={TABS}>
            {(tab) => (
              <View
                role="tab"
                label={tab()[1]}
                selected={v().tab === tab()[0]}
                data-on={v().tab === tab()[0]}
                class="grow basis-0 flex-row items-center justify-center h-7 rounded-sm text-sm text-fg-muted hover:text-fg data-on:bg-raised-lo data-on:text-fg"
                onPress={() => changed("current", tab()[0])}
              >
                <Text>{tab()[1]}</Text>
              </View>
            )}
          </For>
        </View>
        <Input
          class="py-1.5 px-2 rounded-md bg-well border border-border text-sm text-fg"
          label={v().hint}
          placeholder={v().hint}
          value={v().filter}
          onChange={(e) => changed("input", e.value)}
        />
        <Show when={v().tab === "entities"}>
          <Scroll role="tree" label="Entities" class="h-60 shrink-0 flex-col">
            <For each={v().tree} key={(r) => r.id}>
              {(r) => (
                <View class="flex-row items-center gap-0.5 shrink-0" style={{ "padding-left": `${r().depth * 12}px` }}>
                  <Show when={r().fold !== "leaf"} fallback={<View class="size-6 shrink-0" />}>
                    <View role="button" label={r().label} class="size-6 shrink-0 items-center justify-center rounded-sm hover:bg-surface-hover" onPress={() => click("Fold", r().id)}>
                      <Chevron open={r().fold === "open"} />
                    </View>
                  </Show>
                  <View
                    role="treeitem"
                    label={r().label}
                    selected={r().selected}
                    data-on={r().selected}
                    data-unnamed={!r().named}
                    class="grow min-w-0 flex-row items-center h-7 px-2 rounded-sm text-sm text-fg hover:bg-surface-hover data-unnamed:text-fg-muted data-on:bg-surface-hover data-on:border-l-3 data-on:border-hud-accent"
                    onPress={() => click("Label", r().id)}
                  >
                    <Text class="truncate">{r().label}</Text>
                  </View>
                  <Show when={r().children > 0}>
                    <Text class="shrink-0 px-1 text-xs text-fg-subtle">{r().children}</Text>
                  </Show>
                </View>
              )}
            </For>
          </Scroll>
          <View class="h-px shrink-0 bg-border" />
        </Show>
        <Show when={v().tab === "ui"}>
          <Check label="Layout bounds" on={v().bounds} onPress={() => changed("checked", !v().bounds)} />
        </Show>
        <Scroll class="grow shrink basis-0 min-h-0 flex-col gap-1 pr-3">
          <Show when={v().note !== ""}>
            <Text class="text-sm text-fg-muted">{v().note}</Text>
          </Show>
          <Show when={v().tab === "entities"}>
            <Show when={v().head.id !== ""}>
              <View class="flex-row items-center gap-1">
                <Text class="grow min-w-0 truncate text-sm">{v().head.id}</Text>
                <Text class="text-xs text-fg-subtle">{v().head.raw}</Text>
              </View>
            </Show>
            <Show when={v().prefab.what !== ""}>
              <View class="flex-row items-center gap-1">
                <Text class="grow min-w-0 truncate text-xs text-info">{v().prefab.what}</Text>
                <Tool label="Apply all" disabled={v().prefab.none} tip="Write every override into the prefab's file: every instance takes it" onPress={() => click("ApplyAll")} />
                <Tool label="Revert all" quiet disabled={v().prefab.none} tip="Take every override back to the prefab's value" onPress={() => click("RevertAll")} />
              </View>
            </Show>
            <For each={v().overrides} key={(o) => o.id}>
              {(o) => (
                <View class="flex-row items-center gap-1 pl-3">
                  <Text class="grow min-w-0 truncate text-sm text-fg-muted">{o().text}</Text>
                  <Tool label="Apply" quiet tip="Write this value into the prefab" onPress={() => click("Apply", o().id)} />
                  <Tool label="Revert" quiet tip="Back to the prefab's value" onPress={() => click("Revert", o().id)} />
                </View>
              )}
            </For>
          </Show>
          <For each={v().sections} key={(s) => s.id}>
            {(s) => (
              <View class="flex-col gap-1 shrink-0">
                <View role="button" label={s().name} expanded={s().open} class="flex-row items-center gap-1 h-7 px-1 rounded-sm bg-well text-sm hover:bg-surface-hover" onPress={() => click("Section", s().id)}>
                  <Chevron open={s().open} />
                  <Text class="grow min-w-0 truncate">{s().name}</Text>
                </View>
                <For each={s().fields} key={(f) => f.id}>
                  {(f) => (
                    <View class="flex-col shrink-0">
                      <View class="flex-row items-center gap-1 min-h-7">
                        <View data-on={f().mark} class="size-2 shrink-0 rounded-capsule data-on:bg-fg" />
                        <Text
                          data-tone={f().tone}
                          class="w-24 shrink-0 truncate text-xs text-fg-muted data-[tone=override]:text-info data-[tone=flash]:text-warn"
                          description={f().doc}
                        >
                          {f().name}
                        </Text>
                        <Show when={f().editor === "slider"}>
                          <View
                            role="slider"
                            label={f().name}
                            valueMin={f().min}
                            valueMax={f().max}
                            valueNow={f().value}
                            class="grow min-w-0 h-6 flex-row items-center"
                            onPointerDown={(e) => slide(e, f(), (x) => changed("value", x, f().id))}
                            onPointerMove={(e) => slide(e, f(), (x) => changed("value", x, f().id))}
                          >
                            <View class="relative grow h-1 rounded-capsule bg-track">
                              <View class="absolute left-0 top-0 h-full rounded-capsule bg-accent" style={{ width: `${share(f()) * 100}%` }} />
                              <View class="absolute top-0 size-2.5 -translate-y-0.75 rounded-capsule bg-accent" style={{ left: `calc(${share(f()) * 100}% - 5px)` }} />
                            </View>
                          </View>
                        </Show>
                        <Show when={f().editor === "check"}>
                          <Check class="grow" label={f().name} on={f().on} onPress={() => changed("checked", !f().on, f().id)} />
                        </Show>
                        <Show when={f().editor === "pick"}>
                          <View
                            role="combobox"
                            label={f().name}
                            valueText={f().choice}
                            class="grow min-w-0 flex-row items-center justify-between h-7 px-2 rounded-md bg-well border border-border text-sm hover:bg-surface-hover"
                            onPress={() => {
                              const o = f().options;
                              changed("current", o[(o.indexOf(f().choice) + 1) % o.length], f().id);
                            }}
                          >
                            <Text class="truncate">{f().choice}</Text>
                            <Chevron open />
                          </View>
                        </Show>
                        <Show when={f().editor === "link"}>
                          <View role="link" label={f().text} description="Select this entity" class="grow min-w-0 h-7 flex-row items-center px-2 rounded-sm text-sm text-info hover:bg-surface-hover" onPress={() => click("Link", f().id)}>
                            <Text class="truncate">{f().text}</Text>
                          </View>
                        </Show>
                        <Show when={f().editor === "value"}>
                          <Text class="grow min-w-0 truncate text-xs text-fg-subtle">{f().text}</Text>
                        </Show>
                        <Show when={f().swatch !== ""}>
                          <View class="size-6 shrink-0 rounded-sm border border-border" style={{ background: f().swatch }} />
                        </Show>
                        <For each={f().parts} key={(p) => p.id}>
                          {(p) => (
                            <Input
                              data-narrow={f().editor === "slider"}
                              class="grow basis-0 min-w-0 py-1 px-1.5 rounded-sm bg-well border border-border text-sm text-fg data-narrow:grow-0 data-narrow:basis-auto data-narrow:w-16"
                              label={`${f().name} ${p().hint}`}
                              placeholder={p().hint}
                              value={p().text}
                              onChange={(e) => changed("input", e.value, p().id)}
                              {...typing(p().id)}
                            />
                          )}
                        </For>
                      </View>
                      <Show when={f().flash}>
                        <Text class="pl-28 text-xs text-warn">changed by the game: a system writes it every tick</Text>
                      </Show>
                    </View>
                  )}
                </For>
              </View>
            )}
          </For>
          <For each={v().lines} key={(l) => l.id}>
            {(l) => (
              <Text data-kind={l().kind} class="shrink-0 text-xs text-fg data-[kind=head]:text-sm data-[kind=muted]:text-fg-muted">{l().text}</Text>
            )}
          </For>
        </Scroll>
        <Show when={v().tab === "entities"}>
          <View class="flex-row items-center gap-1">
            <Tool label="Move" on={v().mode === "translate"} tip="Move along an axis or a plane (W); Ctrl snaps to 0.5" onPress={() => click("Move")} />
            <Tool label="Rotate" on={v().mode === "rotate"} tip="Turn about an axis or the view (E); Ctrl snaps to 15°" onPress={() => click("Rotate")} />
            <Tool label="Scale" on={v().mode === "scale"} tip="Scale along the entity's own axes, or all from the centre (R); Ctrl snaps to 0.1" onPress={() => click("Scale")} />
            <View class="grow" />
            <Tool label={v().space} quiet tip="Move and rotate along the world's axes or the entity's own" onPress={() => click("Space")} />
          </View>
        </Show>
        <View class="flex-row items-center gap-2">
          <Text class="shrink-0 text-xs text-fg-subtle">{v().clock}</Text>
          <Text class="grow min-w-0 truncate text-sm text-fg-muted">{v().status}</Text>
        </View>
      </View>
    </View>
  );
}

/** A slider field's value as a share of its range. */
function share(f: Field): number {
  return f.max > f.min ? Math.min(1, Math.max(0, (f.value - f.min) / (f.max - f.min))) : 0;
}

/** The value a press or drag at `e.x` on a slider's box gives, on its step. */
function slide(e: UiEvent, f: Field, set: (value: number) => void): void {
  const [bx = 0, , bw = 1] = e.value.split(" ").map(Number);
  const raw = f.min + Math.min(1, Math.max(0, (e.x - bx) / (bw || 1))) * (f.max - f.min);
  set(f.step > 0 ? Math.min(f.max, Math.max(f.min, f.min + Math.round((raw - f.min) / f.step) * f.step)) : raw);
}

inspector.panel = Inspector;
