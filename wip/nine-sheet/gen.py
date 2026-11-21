# Writes project/src/main.tsx and ref/<pack>.html from one table of boxes.
import json, os, shutil, sys
ENGINE = sys.argv[1]  # engine checkout
BOXES = [  # slot, x, y, w, h
  ("Button.plate", 24, 24, 96, 36), ("Button.plate", 140, 24, 160, 44), ("Button.plate", 320, 24, 260, 56),
  ("Button.plate.primary", 600, 24, 96, 36), ("Button.plate.primary", 716, 24, 160, 44), ("Button.plate.primary", 896, 24, 260, 56),
  ("Chip.frame", 24, 110, 36, 20), ("Chip.frame", 76, 110, 56, 24), ("Chip.frame", 148, 110, 80, 28), ("Chip.frame", 244, 110, 140, 28),
  ("Plate.frame", 404, 104, 120, 40), ("Plate.frame", 544, 104, 220, 64), ("Plate.frame", 784, 104, 360, 48),
  ("Panel.frame", 24, 200, 220, 140), ("Panel.frame", 264, 200, 120, 300), ("Panel.frame", 404, 200, 360, 220),
  ("Panel.frame", 24, 540, 1180, 120),
]
os.makedirs("project/src", exist_ok=True); os.makedirs("ref", exist_ok=True)
def cls(slot):
    c, s, *v = slot.split(".")
    return f'packClass("{c}", "{s}"' + (f', "{v[0]}")' if v else ")")
items = "\n".join(
  f'      <View class={{`absolute left-[{x}px] top-[{y}px] w-[{w}px] h-[{h}px] ${{{cls(s)}}}`}} />' for s, x, y, w, h in BOXES)
open("project/src/main.tsx", "w").write(f'''import {{ og, type World }} from "@origingame/engine";
import {{ View, mount, packClass }} from "@origingame/engine/ui";
let shown: (() => void) | null = null;
function sheet(world: World): void {{
  shown?.();
  shown = mount(world, () => (
    <View class="absolute inset-0 bg-[#3a4656]">
{items}
    </View>
  ), {{}}, {{ layer: "hud" }});
}}
function pack(world: World, name: string): void {{
  const style = og.load.style(world, `@styles/${{name}}`);
  if (!style.ok) throw new Error(`@styles/${{name}}: ${{style.reason}}`);
  og.ui.configure(world, {{ style: style.value }});
}}
export default og.game({{
  tick: 60, seed: 1, ui: () => {{}},
  commands: {{
    sheet: (world) => sheet(world),
    pack: (world, args) => pack(world, args[0] ?? "cartoon"),
    unit: (world, args) => og.ui.configure(world, {{ unitScale: Number(args[0] ?? 1) }}),
  }},
}});
''')
def four(v):
    v = v if isinstance(v, list) else [v] * 4
    return v
for pack in ["cartoon", "fantasy", "scifi"]:
    pj = json.load(open(f"{ENGINE}/templates/styles/{pack}/pack.json"))
    images = pj["images"]
    out = []
    for s, x, y, w, h in BOXES:
        v = pj["variants"].get(s, "")
        name = next((t[5:] for t in v.split() if t.startswith("nine-")), None)
        if not name:
            continue
        im = images[name]
        if "src" not in im: im = im.get("dark", im)
        os.makedirs(f"ref/art/{pack}", exist_ok=True)
        shutil.copy(f"{ENGINE}/templates/styles/{pack}/{im['src']}", f"ref/art/{pack}/")
        src = f"art/{pack}/{os.path.basename(im['src'])}"
        sl = four(im["slice"]); wd = four(im.get("width", im["slice"]))
        fill = " fill" if im.get("fill", True) else ""
        rep = "round" if im.get("repeat") == "tile" else "stretch"
        out.append(f'<div style="position:absolute;left:{x}px;top:{y}px;width:{w}px;height:{h}px;box-sizing:border-box;'
                   f'border-style:solid;border-width:{" ".join(f"{k}px" for k in wd)};'
                   f'border-image:url({src}) {" ".join(str(k) for k in sl)}{fill} / {" ".join(f"{k}px" for k in wd)} {rep}"></div>')
    open(f"ref/{pack}.html", "w").write('<!doctype html><style>html,body{margin:0;width:1280px;height:720px;background:#3a4656;overflow:hidden}</style>' + "\n".join(out))
