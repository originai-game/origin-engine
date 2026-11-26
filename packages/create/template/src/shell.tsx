// Customize copy in assets/strings/; give the menus key art, a logo or the live scene with setShellArt.
// Keep the built-in screens: they supply responsive layout, focus and input semantics.
import { og, type World, type Texture } from "@origingame/engine";
import { View, Text, Show, mount, useStore, t, formatDuration } from "@origingame/engine/ui";
import { Button } from "@origingame/engine/ui/controls/button.tsx";
import { Art } from "@origingame/engine/ui/controls/art.tsx";
import { TitleScreen } from "@origingame/engine/ui/screens/title.tsx";
import { MainMenuScreen } from "@origingame/engine/ui/screens/main-menu.tsx";
import { PauseScreen } from "@origingame/engine/ui/screens/pause.tsx";
import { SettingsScreen } from "@origingame/engine/ui/screens/settings.tsx";
import { SaveLoadScreen } from "@origingame/engine/ui/screens/save-load.tsx";
import { shellConfig } from "./shell-config.ts";
import { shellStores, shellReset, sendShell } from "./shell-state.ts";
import "./shell-locales.ts";

/** The dark ground menus stand on when the game gives no key art and no live scene: night falling into the pack's accent glow at the horizon. */
function Backdrop() {
  return <View class="absolute inset-0 scheme-dark bg-bg pointer-events-none">
    <View class="absolute inset-0 bg-linear-to-b from-surface-raised via-surface via-45% to-bg" />
    <View class="absolute -inset-x-1/4 -bottom-1/3 h-2/3 bg-radial from-accent via-accent-soft via-25% to-transparent to-70% opacity-dim" />
    <View class="absolute -inset-x-1/4 -top-1/3 h-2/3 bg-radial from-surface-hover to-transparent to-70%" />
  </View>;
}

export function mountShell(world: World) {
  const stores = shellStores(world);
  shellReset(world);
  mount(world, () => {
    const view = useStore(stores.view), saved = useStore(stores.saved);
    const settings = useStore(stores.settings), notice = useStore(stores.notice);
    const played = useStore(stores.played), slot = useStore(stores.slot);
    // Language tags as their own names (English, 中文); the setting keeps the tag.
    const names = shellConfig.languageNames as Record<string, string>;
    const tagOf = (name: string) => shellConfig.lang.find((l) => names[l] === name) ?? name;
    const send = (action: string, value = "") => sendShell(world, action, value);
    const art = useStore(stores.art), back = useStore(stores.back);
    const keyArt = () => art().keyArt as Texture | 0;
    // Before play the menus stand on key art, the live scene or the dark
    // ground; once a session started they stand on its frozen frame.
    const beforePlay = () => view() === "title" || view() === "menu" || (view() === "settings" && back() === "menu");
    const identity = () => ({ keyArt: 0 as const, logo: art().logo as Texture | 0, name: shellConfig.name, version: "" });
    // Every page is dark, the built-in components in the theme's dark
    // tokens over the game, and arrives the same way: it fades up and settles.
    const page = "absolute inset-0 transition duration-slow ease-out starting:opacity-0 starting:translate-y-2";
    return <View class="absolute inset-0 overflow-hidden scheme-dark" focusGroup="wrap" focusMemory label={shellConfig.name}>
      <Show when={beforePlay() && view() !== "title" && keyArt() !== 0}>
        <Art class="absolute object-cover inset-0 size-full pointer-events-none" kind="keyArt" src={keyArt()} />
      </Show>
      <Show when={beforePlay() && view() !== "title" && keyArt() === 0 && !art().liveScene}><Backdrop /></Show>
      <Show when={!beforePlay() && view() !== "playing"}>
        <View class={`absolute inset-0 pointer-events-none bg-scrim transition-opacity duration-slow ease-out starting:opacity-0 ${view() === "save" || view() === "load" ? "" : "backdrop-blur-regular"}`} />
      </Show>
      <Show when={view() === "title"}>
        <View class="absolute inset-0" modal>
          <TitleScreen {...identity()} keyArt={keyArt()} legal="" onStart={() => send("menu")}
            background={() => <Show when={keyArt() === 0 && !art().liveScene}><Backdrop /></Show>} />
        </View>
      </Show>
      <Show when={view() === "menu"}>
        <View class={page} modal>
          <MainMenuScreen {...identity()} canContinue={saved()} saveLine={t("screen.pause.Played", { played: formatDuration(slot().played) })} news=""
            onNewGame={() => send("new")} onContinue={() => send("load")}
            onSettings={() => send("settings")} />
        </View>
      </Show>
      <Show when={view() === "playing"}>
        <View class="absolute top-4 right-4 mt-[env(hud-inset-top)] mr-[env(hud-inset-right)]">
          <Button onPress={() => send("pause")} label={t("shell.pause")}><Text>{t("shell.pause")}</Text></Button>
        </View>
      </Show>
      <Show when={view() === "paused"}>
        <View class={page} modal>
          <PauseScreen objective="" played={played()} canSave={og.platform.supports(world, "store.set")}
            onResume={() => send("resume")} onSettings={() => send("settings")}
            onSave={() => send("save-menu")} onLoad={() => send("load-menu")}
            onQuitToMenu={() => send("menu")} />
        </View>
      </Show>
      <Show when={view() === "settings"}>
        <View class={page} modal>
          <SettingsScreen note="" pages={[{ id: "general", name: t("shell.general") }]} page="general" pending={false}
            entries={[
              { id: "volume", page: "general", name: t("shell.volume"), description: t("shell.volume.about"), kind: "range", value: settings().volume, min: 0, max: 1, step: 0.05 },
              { id: "textScale", page: "general", name: t("shell.textScale"), description: t("shell.textScale.about"), kind: "range", value: settings().textScale, min: 1, max: 1.5, step: 0.1 },
              { id: "reducedMotion", page: "general", name: t("shell.reducedMotion"), description: t("shell.reducedMotion.about"), kind: "toggle", value: settings().reducedMotion },
              { id: "language", page: "general", name: t("shell.language"), description: t("shell.language.about"), kind: "select", value: names[settings().language] ?? settings().language, options: shellConfig.lang.map((l) => names[l] ?? l) },
            ]}
            onChange={(_, id, value) => send("setting", JSON.stringify({ id, value: id === "language" ? tagOf(String(value)) : value }))}
            onDefaults={() => send("defaults")} onBack={() => send("back")} />
        </View>
      </Show>
      <Show when={view() === "save" || view() === "load"}>
        <View class={page} modal>
          <SaveLoadScreen mode={view()} selected="progress"
            slots={[{ id: "progress", title: shellConfig.name, place: "", played: slot().played, date: "", auto: false, empty: !saved(), thumb: keyArt() }]}
            onBack={() => send("back")} onSelect={() => {}}
            onSave={() => send("save")} onLoad={() => send("load")} />
        </View>
      </Show>
      <Show when={notice() !== ""}>
        <View class="absolute bottom-4 inset-x-4 items-center pointer-events-none"><Text class="bg-bg text-fg px-4 py-2 rounded-panel">{t(notice())}</Text></View>
      </Show>
    </View>;
  }, {}, { layer: "menu" });
}
