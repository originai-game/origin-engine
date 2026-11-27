import { file } from '@origingame/engine/author';

// Buttercups: five glossy yellow petals, cupped.
export default file.model({ kind: 'geometry', desc: { items: [
  { name: 'buttercup', input: { kind: 'plant', desc: {
    kind: 'daisy', seed: 12, trunk: { length: 0.28 },
    flowers: { petals: 5, size: 0.035, width: 0.8, cup: 32,
      color: [1, 0.8, 0.05], color2: [1, 0.62, 0.02], centerColor: [0.85, 0.6, 0.05] },
  } } },
] } });
