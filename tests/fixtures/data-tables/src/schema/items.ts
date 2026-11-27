import { TableRow, defineComponent, f32, list, str, u32 } from "@origingame/engine/schema";

export const Item = defineComponent("Item", {
  damage: u32(), weight: f32(), rarity: str(16), icon: str(64), name: str(32),
  ammo: TableRow(), salvage: list(TableRow(), 4),
});
export const Material = defineComponent("Material", { hardness: u32(), makes: TableRow() });
