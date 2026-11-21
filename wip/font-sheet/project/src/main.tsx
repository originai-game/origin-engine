import { og, type World } from "@origingame/engine";
import { Text, View, mount } from "@origingame/engine/ui";

const SIZES = [12, 14, 16, 20, 28, 48];
const LINES = ["Settings · Quit to menu · Hamburgefonstiv", "开始游戏 设置 第 3 关 · Level 3", "0123456789 +1,240 XP 06:54 ×320"];
let shown: (() => void) | null = null;
function sheet(world: World): void {
  shown?.();
  shown = mount(world, () => (
    <View class="absolute inset-0 bg-[#141a22] px-3 py-2 flex flex-col">
      {LINES.map((l) => (
        <View class="flex flex-col">
          {SIZES.map((s) => (
            <View class="flex flex-row items-center" style={{ height: `${Math.round(s * 1.3)}px` }}>
              <Text class="text-[10px] text-white/50 w-7">{`${s}`}</Text>
              <Text class="text-[#e8ecf2] whitespace-nowrap" style={{ fontSize: `${s}px` }}>{l}</Text>
            </View>
          ))}
        </View>
      ))}
    </View>
  ), {}, { layer: "hud" });
}
function pack(world: World, name: string): void {
  const sc = og.load.font(world, "fonts/NotoSansSC-Regular.ttf");
  if (!sc.ok) throw new Error(`NotoSansSC: ${sc.reason}`);
  og.ui.configure(world, { fallbackFonts: [sc.value] });
  if (name === "default") og.ui.configure(world, { style: null });
  else {
    const style = og.load.style(world, `@styles/${name}`);
    if (!style.ok) throw new Error(`@styles/${name}: ${style.reason}`);
    og.ui.configure(world, { style: style.value });
  }
}
export default og.game({
  tick: 60,
  seed: 1,
  ui: () => {},
  commands: {
    sheet: (world) => sheet(world),
    pack: (world, args) => pack(world, args[0] ?? "default"),
  },
});
