import { file } from '@origingame/engine/author';

// Oxeye daisies: white rays round a yellow eye.
export default file.model({ kind: 'geometry', desc: { items: [
  { name: 'oxeye', input: { kind: 'plant', desc: {
    kind: 'daisy', seed: 7, trunk: { length: 0.34 },
    flowers: { petals: 16, size: 0.05, color: [1, 1, 0.96], centerColor: [0.95, 0.7, 0.08] },
  } } },
] } });
