import { file } from '@origingame/engine/author';

// Generated GIF keyed and brought to its pixel grid.
export default file.flipbook({
  sources: [{ source: 'assets/farmer/farmer_walk.gif', clip: 'walk', events: [[1, 'step'], [5, 'step']] }],
  key: { tolerance: 0.1, holes: true },
  align: 'none', scale: 0.0909, filter: 'nearest',
});
