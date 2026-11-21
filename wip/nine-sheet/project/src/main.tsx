import { og, type World } from "@origingame/engine";
import { Image, View, mount, packClass } from "@origingame/engine/ui";
let shown: (() => void) | null = null;
function sheet(world: World): void {
  shown?.();
  const chip = og.load.texture(world, "@styles/scifi/art/chip.png");
  const tex = chip.ok ? chip.value : 0;
  shown = mount(world, () => (
    <View class="absolute inset-0 bg-[#3a4656]">
      <Image src={tex} class="absolute left-[800px] top-[200px] w-[469px] h-[96px]" />
      <View class={`absolute left-[24px] top-[24px] w-[96px] h-[36px] ${packClass("Button", "plate")}`} />
      <View class={`absolute left-[140px] top-[24px] w-[160px] h-[44px] ${packClass("Button", "plate")}`} />
      <View class={`absolute left-[320px] top-[24px] w-[260px] h-[56px] ${packClass("Button", "plate")}`} />
      <View class={`absolute left-[600px] top-[24px] w-[96px] h-[36px] ${packClass("Button", "plate", "primary")}`} />
      <View class={`absolute left-[716px] top-[24px] w-[160px] h-[44px] ${packClass("Button", "plate", "primary")}`} />
      <View class={`absolute left-[896px] top-[24px] w-[260px] h-[56px] ${packClass("Button", "plate", "primary")}`} />
      <View class={`absolute left-[24px] top-[110px] w-[36px] h-[20px] ${packClass("Chip", "frame")}`} />
      <View class={`absolute left-[76px] top-[110px] w-[56px] h-[24px] ${packClass("Chip", "frame")}`} />
      <View class={`absolute left-[148px] top-[110px] w-[80px] h-[28px] ${packClass("Chip", "frame")}`} />
      <View class={`absolute left-[244px] top-[110px] w-[140px] h-[28px] ${packClass("Chip", "frame")}`} />
      <View class={`absolute left-[404px] top-[104px] w-[120px] h-[40px] ${packClass("Plate", "frame")}`} />
      <View class={`absolute left-[544px] top-[104px] w-[220px] h-[64px] ${packClass("Plate", "frame")}`} />
      <View class={`absolute left-[784px] top-[104px] w-[360px] h-[48px] ${packClass("Plate", "frame")}`} />
      <View class={`absolute left-[24px] top-[200px] w-[220px] h-[140px] ${packClass("Panel", "frame")}`} />
      <View class={`absolute left-[264px] top-[200px] w-[120px] h-[300px] ${packClass("Panel", "frame")}`} />
      <View class={`absolute left-[404px] top-[200px] w-[360px] h-[220px] ${packClass("Panel", "frame")}`} />
      <View class={`absolute left-[24px] top-[540px] w-[1180px] h-[120px] ${packClass("Panel", "frame")}`} />
    </View>
  ), {}, { layer: "hud" });
}
function pack(world: World, name: string): void {
  const style = og.load.style(world, `@styles/${name}`);
  if (!style.ok) throw new Error(`@styles/${name}: ${style.reason}`);
  og.ui.configure(world, { style: style.value });
}
export default og.game({
  tick: 60, seed: 1, ui: () => {},
  commands: {
    sheet: (world) => sheet(world),
    pack: (world, args) => pack(world, args[0] ?? "cartoon"),
    unit: (world, args) => og.ui.configure(world, { unitScale: Number(args[0] ?? 1) }),
  },
});
