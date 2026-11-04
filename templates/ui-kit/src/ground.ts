// Pack-ground review mode, from u8-wip-results-baseline's ground.ts.
let pack = false;
export function packGround(): boolean { return pack; }
export function setGround(value: string): void { pack = value === "pack"; }
