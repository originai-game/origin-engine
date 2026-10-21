// Two command actions over an empty world: `print_save` writes
// world.snapshot() to stdout on a line of its own, `load:<save>` restores it
// (the save's own colons rejoined from the action's arguments).
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";

export default og.game({
  commands: {
    print_save: (world: World) => {
      const save = world.snapshot();
      if (!save.ok) throw new Error(save.reason);
      console.log(save.value);
    },
    load: (world: World, args: readonly string[]) => {
      const r = world.restore(args.join(":"));
      if (!r.ok) throw new Error(r.reason);
    },
  },
});
