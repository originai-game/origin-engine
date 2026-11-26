// Customize copy in assets/strings/; give the title and main menu key art and a logo with setShellArt.
// Keep the built-in screens: they supply responsive layout, focus and input semantics.
import { og, type World, type Texture } from "@origingame/engine";
import { View, Text, Show, mount, useStore, t, formatDuration } from "@origingame/engine/ui";
import { Button } from "@origingame/engine/ui/controls/button.tsx";
import { ToastStack } from "@origingame/engine/ui/controls/toast.tsx";
import { TitleScreen } from "@origingame/engine/ui/screens/title.tsx";
import { MainMenuScreen } from "@origingame/engine/ui/screens/main-menu.tsx";
import { PauseScreen } from "@origingame/engine/ui/screens/pause.tsx";
import { SettingsScreen } from "@origingame/engine/ui/screens/settings.tsx";
import { SaveLoadScreen } from "@origingame/engine/ui/screens/save-load.tsx";
import { shellConfig } from "./shell-config.ts";
import { shellStores, shellReset, sendShell } from "./shell-state.ts";
import "./shell-locales.ts";

export function mountShell(world: World) {
  const stores = shellStores(world);
  shellReset(world);
  mount(world, () => {
    const view = useStore(stores.view), saved = useStore(stores.saved);
    const settings = useStore(stores.settings);
    const played = useStore(stores.played), slot = useStore(stores.slot);
    // Language tags as their own names (English, 中文); the setting keeps the tag.
    const names = shellConfig.languageNames as Record<string, string>;
    const tagOf = (name: string) => shellConfig.lang.find((l) => names[l] === name) ?? name;
    const send = (action: string, value = "") => sendShell(world, action, value);
    const art = useStore(stores.art);
    const keyArt = () => art().keyArt as Texture | 0;
    const identity = () => ({ keyArt: keyArt(), logo: art().logo as Texture | 0, name: shellConfig.name, version: "" });
    // Only built-in screens and controls: each brings its own ground, motion
    // and focus. The pages are dark over the game's frame or key art.
    return <View class="absolute inset-0 overflow-hidden scheme-dark" focusGroup="wrap" focusMemory label={shellConfig.name}>
      <Show when={view() === "title"}>
        <View class="absolute inset-0" modal>
          <TitleScreen {...identity()} legal="" onStart={() => send("menu")} />
        </View>
      </Show>
      <Show when={view() === "menu"}>
        <View class="absolute inset-0" modal>
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
        <View class="absolute inset-0" modal>
          <PauseScreen objective="" played={played()} canSave={og.platform.supports(world, "store.set")}
            onResume={() => send("resume")} onSettings={() => send("settings")}
            onSave={() => send("save-menu")} onLoad={() => send("load-menu")}
            onQuitToMenu={() => send("menu")} />
        </View>
      </Show>
      <Show when={view() === "settings"}>
        <View class="absolute inset-0" modal>
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
        <View class="absolute inset-0" modal>
          <SaveLoadScreen mode={view()} selected="progress"
            slots={[{ id: "progress", title: shellConfig.name, place: "", played: slot().played, date: "", auto: false, empty: !saved(), thumb: keyArt() }]}
            onBack={() => send("back")} onSelect={() => {}}
            onSave={() => send("save")} onLoad={() => send("load")} />
        </View>
      </Show>
    </View>;
  }, {}, { layer: "menu" });
  mount(world, () => {
    const notice = useStore(stores.notice);
    return <ToastStack class="scheme-dark" toasts={notice() === "" ? [] : [{ id: notice(), title: t(notice()), tone: notice() === "shell.failed" ? "danger" : undefined }]}
      onDismiss={() => sendShell(world, "dismiss")} />;
  }, {}, { layer: "toast" });
}
