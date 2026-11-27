import { file } from '@origingame/engine/author';
// Effekseer node 1 "Glow"
export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.0469,
        0.3444
      ],
      [
        0.0781,
        0.5174
      ],
      [
        0.1094,
        0.6534
      ],
      [
        0.1719,
        0.8432
      ],
      [
        0.25,
        1.0
      ],
      [
        0.3333,
        1.0
      ],
      [
        1.0,
        0.0
      ]
    ]
  },
  "blend": "additive",
  "doubleSided": true
});
