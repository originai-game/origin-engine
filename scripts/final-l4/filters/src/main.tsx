// The UI filter scenes the final L4 pass shoots and times (scripts/final-l4-pass.sh filters):
// glass (backdrop blur over busy ground), shadows (blur, drop shadow and float shadow on cards,
// text and a turned box), spotlight (TutorialScreen's shade with its round hole) and minimap
// (MinimapView's round clip). `scene:<name>` mounts one; `scene:all` mounts the four in quarters
// of the screen; `scene:hud` is the filter plan's §6 HUD, which the budget is measured on. A second argument `plain` draws the same nodes with no filter
// classes, the off side of `og cost`.
import { og, type World } from "@origingame/engine";
import { Text, View, mount } from "@origingame/engine/ui";
import { MinimapView } from "@origingame/engine/ui/components/world/minimap-view.tsx";
import { TutorialScreen } from "@origingame/engine/ui/screens/tutorial.tsx";

type Fx = { on: boolean };
const fx = (p: Fx, cls: string) => (p.on ? cls : "");

function Stripes() {
  return (
    <View class="absolute inset-0 flex flex-row">
      {["bg-danger", "bg-warn", "bg-success", "bg-info", "bg-accent", "bg-mana", "bg-gold", "bg-health"].map((c) => (
        <View class={`flex-1 ${c}`} />
      ))}
    </View>
  );
}

function Glass(p: Fx) {
  return (
    <View class="relative size-full overflow-hidden">
      <Stripes />
      <View class="absolute inset-0 flex flex-row items-center justify-center gap-6">
        <View class={`w-64 h-40 rounded-card bg-material-thin border border-rim p-4 ${fx(p, "backdrop-blur-regular")}`}>
          <Text class="text-fg">Glass</Text>
        </View>
        <View class={`size-40 rounded-full bg-material-regular border border-rim ${fx(p, "backdrop-blur-thin")}`} />
        <View class={`w-56 h-40 rounded-sheet bg-material-thick border border-rim p-4 ${fx(p, "backdrop-blur-thick")}`}>
          <View class={`rounded-capsule bg-card px-3 py-1 ${fx(p, "shadow-float")}`}>
            <Text class="text-fg">Chip</Text>
          </View>
        </View>
      </View>
    </View>
  );
}

function Shadows(p: Fx) {
  return (
    <View class="relative size-full bg-bg flex flex-row items-center justify-center gap-8">
      <View class={`w-40 h-28 rounded-card bg-card p-3 ${fx(p, "shadow-float")}`}>
        <Text class="text-fg">Float</Text>
      </View>
      <View class={`w-40 h-28 rounded-card bg-card p-3 ${fx(p, "blur-thin")}`}>
        <Text class="text-fg">Blurred</Text>
      </View>
      <View class={`size-28 rotate-12 rounded-block bg-accent ${fx(p, "shadow-glow-lg")}`} />
    </View>
  );
}

function Spotlight(p: Fx) {
  return (
    <View class="relative size-full bg-bg-subtle">
      <View name="dodge" class="absolute left-1/4 top-1/5 size-24 rounded-full border-2 border-rim-strong bg-card shadow-float" />
      <TutorialScreen step={3} steps={5} target={p.on ? "dodge" : ""} title="Your dodge" text="This is your dodge. Press it just before an attack lands." waits texture={0} x={64} y={120} />
    </View>
  );
}

function Minimap(p: Fx) {
  return (
    <View class="relative size-full bg-bg-subtle flex items-center justify-center">
      <MinimapView texture={0} centerX={0.5} centerY={0.5} zoom={0.3} heading={30} rotate={false} square={!p.on} zone="Ashen Vale"
        markers={[{ id: "ally", kind: "ally", x: 0.45, y: 0.42, label: "Mira" }, { id: "foe", kind: "enemy", x: 0.58, y: 0.55, label: "Enemy" }, { id: "quest", kind: "objective", x: 0.6, y: 0.38, label: "Quest" }]} />
    </View>
  );
}

// The filter plan's §6 HUD at 1080p, the scene the 0.5 ms budget is measured on: 4 blurred
// cards (σ 8, up to 500×300), 8 drop-shadowed icons and labels (σ 4, up to 128×128) and 2
// colour-function backdrops (σ 12 with brightness and saturation, a quarter screen each).
function Hud(p: Fx) {
  return (
    <View class="relative size-full overflow-hidden">
      <Stripes />
      <View class={`absolute left-0 top-0 w-1/2 h-1/2 bg-material-thin ${fx(p, "backdrop-blur-thin backdrop-brightness-75 backdrop-saturate-50")}`} />
      <View class={`absolute right-0 bottom-0 w-1/2 h-1/2 bg-material-thin ${fx(p, "backdrop-blur-thin backdrop-brightness-75 backdrop-saturate-50")}`} />
      <View class="absolute inset-0 p-8 flex flex-row flex-wrap content-start gap-8">
        {[0, 1, 2, 3].map(() => (
          <View class={`w-125 h-75 rounded-card bg-card p-4 ${fx(p, "blur-lens")}`}>
            <Text class="text-fg">Card</Text>
          </View>
        ))}
      </View>
      <View class="absolute left-8 bottom-8 flex flex-row gap-4">
        {[0, 1, 2, 3, 4, 5, 6, 7].map((i) => (
          <View class={`size-32 rounded-block bg-card flex items-center justify-center ${fx(p, "drop-shadow-md")}`}>
            <Text class="text-fg">{`${i + 1}`}</Text>
          </View>
        ))}
      </View>
    </View>
  );
}

const SCENES: Record<string, (p: Fx) => unknown> = { glass: Glass, shadows: Shadows, spotlight: Spotlight, minimap: Minimap };

function scene(world: World, name: string, on: boolean): void {
  const one = SCENES[name];
  if (one) return void mount(world, () => one({ on }) as never, {}, { layer: "hud" });
  if (name === "hud") return void mount(world, () => Hud({ on }) as never, {}, { layer: "hud" });
  if (name !== "all") throw new Error(`scene ${name}: one of all, hud, ${Object.keys(SCENES).join(", ")}`);
  mount(world, () => (
    <View class="absolute inset-0 grid grid-cols-2 grid-rows-2">
      {Object.values(SCENES).map((s) => <View class="relative overflow-hidden">{s({ on }) as never}</View>)}
    </View>
  ), {}, { layer: "hud" });
}

function pack(world: World, name: string): void {
  if (name === "default") return void og.ui.configure(world, { style: null });
  const style = og.load.style(world, `@styles/${name}`);
  if (!style.ok) throw new Error(`@styles/${name}: ${style.reason}`);
  og.ui.configure(world, { style: style.value });
}

export default og.game({
  tick: 60,
  seed: 1,
  ui: () => {},
  commands: {
    scene: (world, args) => scene(world, args[0] ?? "all", args[1] !== "plain"),
    pack: (world, args) => pack(world, args[0] ?? "default"),
    scheme: (world, args) => og.ui.configure(world, { scheme: args[0] === "light" ? "light" : "dark" }),
  },
});
