// A style pack's core-control state sheet (docs/ui.md §6.2,
// sheets/style): four pages, each captured at desktop and phone. Kit
// extensions own their sheets.
export const BASELINE: ReadonlyArray<{ name: string; commands: string; root: string }> = [
  { name: "controls", commands: "", root: "StyleSheet" },
  { name: "fields", commands: "page:fields", root: "FieldSheet" },
  { name: "surfaces", commands: "page:surfaces", root: "SurfaceSheet" },
  { name: "overlays", commands: "page:overlays", root: "OverlaySheet" },
];
