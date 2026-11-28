// The UI gallery's control overview: one frame of the core controls as a front end and a settings
// panel put them together (the menu with its current entry, buttons, a titled panel with tabs,
// sliders, switches and a checkbox, a dialog, a toast, a tooltip and the HUD bars), so a pack is
// judged on one picture. Not a component: the gallery copies it into its project.
import { Text, View, packClass } from "@origingame/engine/ui";
import { Button } from "@origingame/engine/ui/controls/button";
import { Checkbox } from "@origingame/engine/ui/controls/checkbox";
import { MenuEntry, MenuList } from "@origingame/engine/ui/controls/menu-list";
import { Panel } from "@origingame/engine/ui/controls/panel";
import { Slider } from "@origingame/engine/ui/controls/slider";
import { Switch } from "@origingame/engine/ui/controls/switch";
import { Tab, Tabs } from "@origingame/engine/ui/controls/tabs";
import { HudBar } from "@origingame/engine/ui/components/hud/bar";

export interface OverviewSheetProps {
  /** The frame's shape: a wide screen's columns, or a phone's single column. */
  layout: "landscape" | "portrait";
}

const press = () => {};
const change = () => {};

function Menu() {
  return (
    <MenuList label="Main menu" class={`w-full p-3 ${packClass("Panel", "frame")}`}>
      <MenuEntry id="continue" label="Continue" line="Chapter 3 · 4h 12m" onPress={press} autofocus />
      <MenuEntry id="newGame" label="New game" onPress={press} />
      <MenuEntry id="settings" label="Settings" onPress={press} />
      <MenuEntry id="credits" label="Credits" minor onPress={press} />
      <MenuEntry id="quit" label="Quit" minor danger onPress={press} />
    </MenuList>
  );
}

function Settings() {
  return (
    <Panel title="Settings" class="w-full">
      <Tabs label="Sections" class="w-full">
        <Tab selected onPress={press}><Text>Audio</Text></Tab>
        <Tab onPress={press}><Text>Video</Text></Tab>
        <Tab onPress={press}><Text>Controls</Text></Tab>
      </Tabs>
      <View class="flex-col gap-1 w-full">
        <View class="flex-row items-center gap-4"><Text class="w-24">Music</Text><Slider class="grow" value={70} label="Music" onChange={change} /></View>
        <View class="flex-row items-center gap-4"><Text class="w-24">Effects</Text><Slider class="grow" value={35} label="Effects" onChange={change} /></View>
        <Switch checked onChange={change}><Text>Subtitles</Text></Switch>
        <Switch onChange={change}><Text>Screen shake</Text></Switch>
        <Checkbox checked onChange={change}><Text>Show damage numbers</Text></Checkbox>
      </View>
      <View class="flex-row justify-end gap-3 w-full">
        <Button onPress={press}><Text>Back</Text></Button>
        <Button variant="danger" onPress={press}><Text>Reset</Text></Button>
        <Button variant="primary" onPress={press}><Text>Apply</Text></Button>
      </View>
    </Panel>
  );
}

function Feedback() {
  return (
    <View class="flex-col gap-4 w-full">
      <View class={`flex-col gap-3 ${packClass("Dialog", "frame")} w-full`}>
        <Text class={packClass("Dialog", "title")}>Leave the match?</Text>
        <Text class={packClass("Dialog", "body")}>Your team plays on without you, and the match counts as a loss.</Text>
        <View class={packClass("Dialog", "actions")}>
          <Button variant="ghost-muted" onPress={press}><Text>Stay</Text></Button>
          <Button variant="danger" onPress={press}><Text>Leave</Text></Button>
        </View>
      </View>
      <View class="flex-row flex-wrap items-center gap-4">
        <View class={packClass("Toast", "frame")}>
          <View class="flex-col grow">
            <Text class="font-semibold">Game saved</Text>
            <Text class="text-sm text-fg-muted">Chapter 3 · the lighthouse</Text>
          </View>
        </View>
        <View class="flex-row items-center gap-3">
          <Button size="sm" onPress={press}><Text>Hover me</Text></Button>
          <View class={packClass("Tooltip", "frame")}><Text>Opens the map</Text></View>
        </View>
      </View>
    </View>
  );
}

function Hud() {
  return (
    <View class={`flex-col gap-2 w-full p-3 ${packClass("Plate", "frame")}`}>
      <HudBar value={72} max={100} kind="health" showValue />
      <HudBar value={40} max={100} kind="mana" />
      <HudBar value={18} max={100} kind="health" low />
    </View>
  );
}

export function OverviewSheet(props: OverviewSheetProps) {
  return props.layout === "portrait" ? (
    <View class="absolute inset-0 flex-col gap-3 p-4 overflow-hidden">
      <Text class={`self-center text-center ${packClass("Title", "wordmark")} text-display-xl`}>Lantern Coast</Text>
      <Menu />
      <Settings />
      <Feedback />
      <Hud />
    </View>
  ) : (
    <View class="absolute inset-0 flex-row gap-8 p-8 overflow-hidden">
      <View class="flex-col gap-6 w-80 shrink-0">
        <Text class={`${packClass("Title", "wordmark")} text-display-xl`}>Lantern Coast</Text>
        <Menu />
        <Hud />
      </View>
      <View class="flex-col w-120 shrink-0"><Settings /></View>
      <View class="flex-col grow"><Feedback /></View>
    </View>
  );
}

export const fixtures: Record<string, OverviewSheetProps> = {
  landscape: { layout: "landscape" },
  portrait: { layout: "portrait" },
};
