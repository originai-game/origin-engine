import { og } from "@origingame/engine";
import { Text, View, mount } from "@origingame/engine/ui";

export default og.game({
  tick: 60,
  seed: 1,
  ui: (world) => {
    mount(world, () => (
      <View class="absolute inset-0 items-center justify-center gap-4 bg-bg">
        <Text class="text-4xl font-display text-fg">商店</Text>
        <Text class="text-xl text-fg">金币不足，还差 1,600。</Text>
        <Text class="text-xl text-fg">Gold 3,200 → 2,500</Text>
      </View>
    ), {});
  },
});
