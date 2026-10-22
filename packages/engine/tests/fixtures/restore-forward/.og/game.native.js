// @bun
// packages/engine/tests/fixtures/restore-forward/src/main.ts
import { og } from "/home/user/workspace/repo/packages/engine/src/index.ts";
var main_default = og.game({
  commands: {
    print_save: (world) => {
      const save = world.snapshot();
      if (!save.ok)
        throw new Error(save.reason);
      console.log(save.value);
    },
    load: (world, args) => {
      const r = world.restore(args.join(":"));
      if (!r.ok)
        throw new Error(r.reason);
    }
  }
});
export {
  main_default as default
};
