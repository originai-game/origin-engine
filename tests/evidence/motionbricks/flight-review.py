"""Build a disposable og run project from flight_plan --review JSON; no GPU."""
import argparse
import json
from pathlib import Path
import shutil

parser = argparse.ArgumentParser(__doc__)
parser.add_argument('recording', type=Path)
parser.add_argument('destination', type=Path)
args = parser.parse_args()
root = Path(__file__).resolve().parents[3]
rows = json.loads(args.recording.read_text())['results']
assert len(rows) == 20 and all('review' in row for row in rows)
dest = args.destination
dest.mkdir(parents=True, exist_ok=False)
(dest/'src').mkdir()
(dest/'assets/fonts').mkdir(parents=True)
for name in ('ui.ttf', 'OFL.txt'):
    shutil.copyfile(root/'examples/ui-motion/assets/fonts'/name, dest/'assets/fonts'/name)
template = Path(__file__).with_suffix('.ts').read_text()
drawn = [dict(group=r['group'], rig=r['rig'], frames=r['frames'],
              review={key:r['review'][key] for key in ('nodes','raw','aligned','corrected','contacts')}) for r in rows]
(dest/'src/main.ts').write_text(template.replace('/* EXPORTED_CASES */ []', json.dumps(drawn, separators=(',', ':'))))
(dest/'project.ogt').write_text('type project\nversion 1\n---\nname "Flight operator review"\nid "flight-review"\nengine "0.7.7"\nentry "src/main.ts"\nprofile "3d"\ntick 30\nseed 1\n')
shutil.copyfile(root/'examples/ui-motion/tsconfig.json', dest/'tsconfig.json')
index = []
for i,row in enumerate(rows):
    r = row['review']
    samples = sorted({r['takeoff'], r['apex'], r['recovery'],
                      *range(max(0,r['landing']-2), min(row['frames'],r['landing']+6))})
    samples = [s for s in samples if s < row['frames']]
    commands = [f'0.1:case:{i}', '0.2:review_view:all']
    time = .4
    for sample in samples:
        commands += [f'{time:.2f}:sample:{sample}', f'{time+.1:.2f}:screenshot:case-{i:02d}-s{sample:02d}.png']
        time += .3
    commands += [f'{time:.2f}:review_view:corrected', f'{time+.1:.2f}:sample:{r["apex"]}',
                 f'{time+.2:.2f}:screenshot:case-{i:02d}-apex-detail.png',f'{time+.3:.2f}:quit']
    (dest/f'case-{i:02d}.commands').write_text(';'.join(commands)+'\n')
    index.append(dict(index=i, group=row['group'], rig=row['rig'], samples=samples))
(dest/'index.json').write_text(json.dumps(index,indent=2)+'\n')
print(dest)
