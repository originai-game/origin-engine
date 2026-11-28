// The review matrix uses the real built-in screens over each look's scene.
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { mount, Text, useStore, View } from "@origingame/engine/ui";
import { TitleScreen } from "@origingame/engine/ui/screens/title.tsx";
import { ResultsScreen } from "@origingame/engine/ui/screens/results.tsx";
import { Button } from "@origingame/engine/ui/controls/button.tsx";

const press = () => {};
const page = (world: World) => og.ui.scope(world, "look-review").writable("page", "play");

export function screen(world: World, args: readonly string[]) {
  page(world).set(args[0] ?? "play");
}

export function reviewUi(world: World) {
  mount(world, () => {
    const shown = useStore(page(world));
    return (
    <View class="absolute inset-0 scheme-dark">
      {shown() === "title" ? <TitleScreen name="Lantern Valley" version="LOOK REVIEW" legal="A journey through light and colour" keyArt={0} logo={0} onStart={press} />
        : shown() === "results" ? <ResultsScreen won title="Valley explored" grade="A" tab="summary"
          stats={[{ id: "gems", name: "Lantern shards", value: 7, best: true }, { id: "paths", name: "Paths discovered", value: 3, best: false }]}
          rewards={[]} onAgain={press} onContinue={press} />
        : <>
          <View class="absolute top-6 left-6 rounded-panel bg-glass border border-rim px-5 py-3 gap-1">
            <Text class="font-display text-xl font-bold text-fg">Lantern Valley</Text>
            <Text class="text-sm text-fg-muted">Find the path through the valley</Text>
          </View>
          <View class="absolute bottom-6 right-6"><Button onPress={press} label="Journal"><Text>Journal · 7 shards</Text></Button></View>
        </>}
    </View>
    );
  }, {}, { layer: "hud" });
}
