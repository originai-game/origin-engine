// Two command actions over an empty world: `print_save` writes
// world.snapshot() to stdout on a line of its own, `load:<save>` restores it
// (the save's own colons rejoined from the action's arguments).
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";

// The native host's console: a game's lib is ES2022 alone (og check reads
// templates/game/tsconfig.json for a project without one), so this fixture,
// which prints the save for og-run.test.ts to read, declares what it uses.
declare const console: { log(line: string): void };

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
