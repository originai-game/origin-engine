import { file } from "@origingame/engine/author";

export default file.grammar({
  params: { height: 12, floor: 3.2 },
  start: "Lot",
  rules: {
    Lot: [{ op: "extrude", depth: "height", faces: { front: "Facade", sides: "Rear", top: "Roof" } }],
    Facade: [{ op: "split", axis: "y", parts: [
      { size: "abs", length: 4, rule: "Ground" },
      { size: "repeat", length: "floor", rule: "Floor" },
      { size: "abs", length: 0.4, rule: "Cornice" },
    ] }],
    Rear: [{ op: "split", axis: "y", parts: [
      { size: "abs", length: 0.45, rule: "Plinth" },
      { size: "repeat", length: "floor", rule: "Floor" },
      { size: "abs", length: 0.4, rule: "Cornice" },
    ] }],
    Floor: [{ op: "split", axis: "x", parts: [
      { size: "abs", length: 0.5, rule: "Wall" },
      { size: "repeat", length: 2.8, rule: "Bay" },
      { size: "abs", length: 0.5, rule: "Wall" },
    ] }],
    Bay: [{ op: "split", axis: "x", parts: [
      { size: "fill", length: 1, rule: "Wall" },
      { size: "abs", length: 1.2, rule: "Window" },
      { size: "fill", length: 1, rule: "Wall" },
    ] }],
    Window: [{ op: "split", axis: "y", parts: [
      { size: "abs", length: 0.9, rule: "Wall" },
      { size: "abs", length: 1.6, rule: "Glass" },
      { size: "fill", length: 1, rule: "Wall" },
    ] }],
    Glass: [{ op: "recess", depth: 0.12 }, { op: "label", label: "glass" }],
    Ground: [{ op: "pick", options: [{ weight: 3, rule: "Shopfront" }, { weight: 2, rule: "Residential" }] }],
    Shopfront: [{ op: "split", axis: "x", parts: [
      { size: "abs", length: 0.4, rule: "Pier" },
      { size: "fill", length: 1, rule: "Shop" },
      { size: "abs", length: 1.35, rule: "Entrance" },
      { size: "abs", length: 0.4, rule: "Pier" },
    ] }],
    Residential: [{ op: "split", axis: "x", parts: [
      { size: "abs", length: 0.4, rule: "Pier" },
      { size: "abs", length: 1.35, rule: "Entrance" },
      { size: "fill", length: 1, rule: "HouseWindow" },
      { size: "abs", length: 0.4, rule: "Pier" },
    ] }],
    Pier: [{ op: "split", axis: "y", parts: [
      { size: "abs", length: 0.45, rule: "Plinth" },
      { size: "fill", length: 1, rule: "Wall" },
    ] }],
    HouseWindow: [{ op: "split", axis: "y", parts: [
      { size: "abs", length: 0.45, rule: "Plinth" },
      { size: "fill", length: 1, rule: "Floor" },
    ] }],
    Shop: [{ op: "split", axis: "y", parts: [
      { size: "abs", length: 0.45, rule: "Plinth" },
      { size: "fill", length: 1, rule: "ShopWindows" },
      { size: "abs", length: 0.45, rule: "Frame" },
    ] }],
    ShopWindows: [{ op: "split", axis: "x", parts: [{ size: "repeat", length: 2.2, rule: "ShopWindow" }] }],
    ShopWindow: [{ op: "inset", width: 0.1, border: "Frame", inner: "ShopGlass" }],
    ShopGlass: [{ op: "recess", depth: 0.16, jamb: "Frame" }, { op: "label", label: "shopglass" }],
    Entrance: [{ op: "split", axis: "y", parts: [
      { size: "abs", length: 0.12, rule: "Plinth" },
      { size: "abs", length: 2.6, rule: "DoorFrame" },
      { size: "fill", length: 1, rule: "Frame" },
    ] }],
    DoorFrame: [{ op: "inset", width: 0.1, border: "Frame", inner: "Door" }],
    Door: [{ op: "recess", depth: 0.2, jamb: "Frame" }, { op: "split", axis: "y", parts: [
      { size: "abs", length: 0.85, rule: "DoorPanel" },
      { size: "fill", length: 1, rule: "DoorLight" },
    ] }],
    DoorPanel: [{ op: "label", label: "door" }],
    DoorLight: [{ op: "inset", width: 0.07, border: "Frame", inner: "DoorPane" }],
    DoorPane: [{ op: "label", label: "shopglass" }],
    Frame: [{ op: "label", label: "shopfront" }],
    Plinth: [{ op: "extrude", depth: 0.06, faces: { top: "PlinthEdge", sides: "PlinthEdge" } }],
    PlinthEdge: [{ op: "label", label: "plinth" }],
    Cornice: [{ op: "label", label: "trim" }],
    Wall: [{ op: "label", label: "wall" }],
    Roof: [{ op: "roof", roof: { kind: "hip", angle: 35 }, overhang: 0.3, slope: "Slate" }],
    Slate: [{ op: "label", label: "roof" }],
  },
});
