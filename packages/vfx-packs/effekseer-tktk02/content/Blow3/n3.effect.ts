import { file } from '@origingame/engine/author';
// Effekseer node 3 "Ring"
export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.4,
        1.0
      ],
      [
        0.6,
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
