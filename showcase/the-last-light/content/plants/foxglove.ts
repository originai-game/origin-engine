import { file } from '@origingame/engine/author';

// A foxglove: a tall spire hung with purple bells.
export default file.model({ kind: 'geometry', desc: { items: [
  { name: 'foxglove', input: { kind: 'plant', desc: {
    kind: 'tulip', seed: 4, trunk: { length: 1.05, radius: 0.009, sections: 8 },
    leaves: { count: 9, end: 0.35, size: 0.2, width: 0.34 },
    flowers: { share: 1, count: 14, petals: 5, size: 0.045, width: 0.7, cup: 74,
      color: [0.62, 0.2, 0.6], color2: [0.85, 0.45, 0.78], centerColor: [0.95, 0.85, 0.9] },
  } } },
] } });
