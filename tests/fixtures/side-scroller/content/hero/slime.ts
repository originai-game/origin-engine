import { file } from '@origingame/engine/author';

export default file.flipbook({
  sources: [{ source: 'assets/hero/slime.gif', clip: 'hop', align: 'none' }],
  key: { tolerance: 0.1, holes: true },
  align: 'feet', scale: 0.2, filter: 'nearest',
});
