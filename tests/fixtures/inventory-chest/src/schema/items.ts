import { defineComponent, str, u32 } from "@origingame/engine/schema";

export const ItemRow = defineComponent("ItemRow", { maxStack: u32(), name: str(32) });
