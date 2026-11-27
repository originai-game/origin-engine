import { file } from '@origingame/engine/author';

// Generated GIFs keyed, held on their feet and cut to one cell.
export default file.flipbook({
  sources: [
    { source: 'assets/hero/hero_idle.gif', clip: 'idle', scale: 0.89 },
    { source: 'assets/hero/hero_run.gif', clip: 'run', events: [[1, 'step'], [5, 'step']] },
    { source: 'assets/hero/hero_attack.gif', clip: 'attack', mode: 'once', events: [[3, 'hit']] },
  ],
  key: { tolerance: 0.1, holes: true },
  align: 'feet', scale: 0.25, filter: 'nearest',
});
