# inventory-chest

og.systems.inventory with an item table (`assets/items.ogt`, stack limits
in `maxStack`, read as the game's `ItemRow`): a 4-slot bag carried by a
kinematic walker (an og.Collector), a 3-slot chest holding a sword and 4
ore, and three pickups on the walker's path (60 coins, 12 ore, 8 ore)
(`src/main.ts`).

What it checks (`tests/chest.og.ts`, `og test tests/fixtures/inventory-chest`):

- the walker crosses the pickups' sensors and takes all 80 items into the
  bag, which fills its four slots (coins 50 + 10, ore 10 + 10);
- `overfill` moves the chest's sword into the full bag: refused, neither
  side changes; `deposit`, `split` (3 off, merged back) and `stash` move
  ore between them; `smelt` crafts an ingot from 2 ore: refused while the
  bag has no slot for it, done once one is free; `roll` draws 5 loot lines
  on og.Rng;
- every tick, nothing is made or lost but by craft, no stack passes its
  row's limit and every slot is its container's child; two runs end on one
  hash.

A refused move or craft leaving the world hash as it was, split and merge
counts, a restore with its slots and parents, the same loot from one seed
and a lowered slot count are `packages/engine/tests/inventory.test.ts`.

Budget: under 10 s warm. No frames: nothing here is drawn.
