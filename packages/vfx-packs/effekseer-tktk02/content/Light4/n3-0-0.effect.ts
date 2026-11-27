import { file } from '@origingame/engine/author';
// Effekseer node 3.0.0 "Lins"
export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.25,
        0.75
      ],
      [
        0.3125,
        0.8594
      ],
      [
        0.375,
        0.9375
      ],
      [
        0.4375,
        0.9844
      ],
      [
        0.5,
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
