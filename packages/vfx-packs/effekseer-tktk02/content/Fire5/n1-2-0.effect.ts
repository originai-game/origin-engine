import { file } from '@origingame/engine/author';
// Effekseer node 1.2.0 "Rast"
export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.1667,
        1.0
      ],
      [
        0.6667,
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
