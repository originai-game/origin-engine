import json, sys, collections
sys.path.insert(0, ".")
import owners
from table import profile, ORDER, AB

NOTES = {
    "nodes": "the arena; split by Node's groups in batch 6",
    "state": "persisted (save_state); its fields are placed below",
    "kept": "LayoutCache, 44 fields over four stages; split in batch 1",
    "theme": "paint writes the palette for an accent (commands.rs:490-511); fixed in 1.1",
    "world": "WorldUi: declared by tree, placed by layout; split in batch 6",
    "orders": "draw order is written by paint (`update_draw_order`, `paint_reuse`)",
    "part_commits": "written by paint after rendering (commands.rs)",
    "selected": "written by paint (`mark_selection`)",
    "docs": "DocHost: documents, mounts, layer stack",
    "touched": "widget memories used this frame",
    "draws": "game draws per node (`set_draw`): authored paint input",
    "tick": "the UI clock, persisted",
    "motion": "MotionState; paint and tree write it (ghosts, drag)",
    "el": "layout writes it (settlement swaps, `apply_patch`, `line_box`, writing modes); fixed in 6.2",
    "sampled": "layout writes it (`write_values` for container-query patches, world placement)",
    "clip": "paint writes it in `commands` (8 sites); fixed in 1.2",
    "props": "layout writes it (`apply_patch`, `turn_box`)",
    "hidden": "front, docs and style write it",
    "rect": "",
}


def fmt(c):
    return " ".join(f"{AB[k]}{c[k]}" for k in ORDER if c.get(k)) or "–"


def table(name, own, title):
    prof = profile(name)
    out = [f"| {title} field | Owner | Sites | Writes by stage | Reads by stage | Note |", "|---|---|---|---|---|---|"]
    groups = collections.defaultdict(list)
    for f in prof:
        groups[own[f]].append(f)
    for o in groups:
        for f in groups[o]:
            n, w, r = prof[f]
            out.append(f"| `{f}` | {o} | {n} | {fmt(w)} | {fmt(r)} | {NOTES.get(f, '')} |")
    return "\n".join(out), {o: len(v) for o, v in groups.items()}


blocks = json.load(open("blocks.json"))


def short(c):
    return ", ".join(f"{k} {v}" for k, v in sorted(c.items(), key=lambda kv: -kv[1])) or "–"


def foreign(r):
    t = r["target"]
    own = {"Interaction": {"Interaction", "node.Interaction"}, "Tree": {"Tree", "node.Tree", "node.Authored"},
           "Style": {"Style", "node.Style"}, "Layout": {"Layout", "node.Layout", "node.Placed"},
           "Paint": {"Paint"}, "Docs": {"Docs"}}.get(t, set())
    return {k: v for k, v in r["W"].items() if k not in own and k != "Out"}


def blocks_table():
    out = ["| Block | Lines | Fns | Stage | Target | Writes outside the target (Out excluded) | Calls into other stages |", "|---|---|---|---|---|---|---|"]
    for r in blocks:
        out.append(f"| `{r['block'].removeprefix('src/')}` | {r['lines']} | {r['fns']} | {r['stage']} | {r['target']} | {short(foreign(r))} | {short(r['calls'])} |")
    return "\n".join(out)


if __name__ == "__main__":
    which = sys.argv[1]
    if which == "ui":
        t, c = table("ui", owners.UI, "Ui"); print(t); print(); print(c, file=sys.stderr)
    elif which == "state":
        t, c = table("state", owners.STATE, "State"); print(t); print(c, file=sys.stderr)
    elif which == "node":
        t, c = table("node", owners.NODE, "Node"); print(t); print(c, file=sys.stderr)
    elif which == "blocks":
        print(blocks_table())
