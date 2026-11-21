// The accessibility mirror's input (src/web/a11y.ts A11yUpdate) from a tree
// written with each node's parent: a whole update, as the Worker sends the
// first one after the tree was made.
import type { A11yNode, A11yTree, A11yUpdate } from "../../src/web/a11y.ts";

export function whole(tree: Pick<A11yTree, "nodes" | "focused"> & Partial<A11yTree>): A11yUpdate {
  const under = (path: string) => tree.nodes.filter((n) => n.parent === path).map((n) => n.path);
  const root = { path: "0", role: "", name: "", description: "", value: "", states: [], x: 0, y: 0, width: 0, height: 0, children: under("") };
  return {
    full: true,
    focus: tree.focused,
    nodes: [root, ...tree.nodes.map(({ parent: _, ...n }: A11yNode) => ({ ...n, children: under(n.path) }))],
  };
}
