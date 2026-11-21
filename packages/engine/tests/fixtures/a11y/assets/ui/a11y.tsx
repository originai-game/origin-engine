import { For, Show, Text, View, type UiEvent } from "@origingame/engine/ui";

export interface A11yProps {
  /** The settings dialog is open. */
  settings: boolean;
  /** The pause dialog is open. */
  pause: boolean;
  /** The settings section shown: audio or graphics. */
  tab: "audio" | "graphics";
  /** The master and music volumes, 0 to 1. */
  master: number;
  music: number;
  /** Mute everything, subtitles and high contrast. */
  muted: boolean;
  subtitles: boolean;
  contrast: boolean;
  /** What the polite status says ("Settings saved"); empty says nothing. */
  saved: string;
  onTab: (e: UiEvent, tab: "audio" | "graphics") => void;
  onVolume: (e: UiEvent, which: "master" | "music", value: number) => void;
  onSwitch: (e: UiEvent, which: "muted" | "subtitles" | "contrast") => void;
  onDone: (e: UiEvent) => void;
  onResume: (e: UiEvent) => void;
}

const TABS = [{ id: "audio", label: "Audio" }, { id: "graphics", label: "Graphics" }] as const;
const STEP = 0.05;

/** A labelled switch row: the row is the switch, named by its words. */
function Switch(props: { label: string; on: boolean; onToggle: (e: UiEvent) => void }) {
  return (
    <View class="flex-row items-center justify-between w-full min-h-11 gap-3" role="switch" tabIndex={0} checked={props.on} onPress={props.onToggle}>
      <Text>{props.label}</Text>
      <View class="w-10 h-6 p-0.5 rounded-capsule bg-tile data-on:bg-accent data-on:items-end" data-on={props.on}>
        <View class="size-5 rounded-full bg-fg" />
      </View>
    </View>
  );
}

/** A volume row: its name and a slider the arrow keys and a screen reader's adjust move. */
function Volume(props: { label: string; value: number; onSet: (e: UiEvent, value: number) => void }) {
  const set = (e: UiEvent, v: number) => props.onSet(e, Math.min(1, Math.max(0, Math.round(v / STEP) * STEP)));
  return (
    <View class="flex-row items-center w-full gap-4">
      <Text class="w-35">{props.label}</Text>
      <View
        class="grow h-11 justify-center"
        role="slider"
        label={props.label}
        tabIndex={0}
        valueMin={0}
        valueMax={1}
        valueNow={props.value}
        valueText={`${Math.round(props.value * 100)}%`}
        onChange={(e) => set(e, Number(e.value))}
        onKey={(e) => (e.key === "ArrowLeft" || e.key === "ArrowDown" ? set(e, props.value - STEP) : e.key === "ArrowRight" || e.key === "ArrowUp" ? set(e, props.value + STEP) : undefined)}
      >
        <View class="h-1.5 w-full rounded-capsule bg-tile">
          <View class="h-full rounded-capsule bg-accent" style={{ width: `${props.value * 100}%` }} />
        </View>
      </View>
    </View>
  );
}

/** The accessibility mirror's fixture: a settings dialog and a pause dialog. */
export function A11y(props: A11yProps) {
  return (
    <View class="absolute inset-0 items-center justify-center">
      <Show when={props.settings}>
        <View class="w-150 max-w-full gap-3 p-6 rounded-panel bg-surface shadow-overlay" role="dialog" modal={true} label="Settings">
          <Text class="text-xl font-bold" role="heading" level={2}>Settings</Text>
          <View class="flex-row w-full gap-2" role="radiogroup" label="Section">
            <For each={TABS}>
              {(tab) => (
                <View
                  class="min-h-11 px-4 justify-center rounded-control bg-tile data-on:bg-accent data-on:text-bg"
                  role="radio"
                  tabIndex={0}
                  checked={props.tab === tab().id}
                  data-on={props.tab === tab().id}
                  onPress={(e) => props.onTab(e, tab().id)}
                >
                  <Text>{tab().label}</Text>
                </View>
              )}
            </For>
          </View>
          <Show when={props.tab === "audio"} fallback={<Switch label="High contrast" on={props.contrast} onToggle={(e) => props.onSwitch(e, "contrast")} />}>
            <Volume label="Master" value={props.master} onSet={(e, v) => props.onVolume(e, "master", v)} />
            <Volume label="Music" value={props.music} onSet={(e, v) => props.onVolume(e, "music", v)} />
            <Switch label="Mute everything" on={props.muted} onToggle={(e) => props.onSwitch(e, "muted")} />
            <Switch label="Subtitles" on={props.subtitles} onToggle={(e) => props.onSwitch(e, "subtitles")} />
          </Show>
          <View class="flex-row justify-end w-full">
            <View class="min-h-11 px-6 justify-center rounded-button bg-accent text-bg font-semibold" role="button" tabIndex={0} onPress={props.onDone}>
              <Text>Done</Text>
            </View>
          </View>
        </View>
      </Show>
      <Show when={props.pause}>
        <View class="gap-3 p-6 rounded-panel bg-surface shadow-overlay transition duration-200 starting:opacity-0 starting:scale-90 motion-reduce:starting:scale-100" role="dialog" modal={true} label="Paused">
          <Text class="text-xl font-bold" role="heading" level={2}>Paused</Text>
          <View class="min-h-11 px-6 justify-center rounded-button bg-accent text-bg font-semibold" role="button" tabIndex={0} onPress={props.onResume}>
            <Text>Resume</Text>
          </View>
        </View>
      </Show>
      <View class="absolute bottom-6 inset-x-0 items-center" role="status" live="polite">
        <Show when={props.saved}>
          <Text class="px-4 py-2 rounded-capsule bg-surface-raised">{props.saved}</Text>
        </Show>
      </View>
    </View>
  );
}
