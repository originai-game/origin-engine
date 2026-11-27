import { file } from '@origingame/engine/author';
// Effekseer node 1.0.1 "ｃｒａｓｈ"
export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.2,
        1.0
      ],
      [
        0.4,
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
