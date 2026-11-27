import { file } from '@origingame/engine/author';
// Effekseer node 4 "Hole"
export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        1.0
      ],
      [
        0.8361,
        1.0
      ],
      [
        0.8906,
        0.5497
      ],
      [
        0.9219,
        0.3301
      ],
      [
        0.9531,
        0.1542
      ],
      [
        0.9688,
        0.0866
      ],
      [
        1.0,
        0.0
      ]
    ]
  },
  "blend": "alpha",
  "doubleSided": true
});
