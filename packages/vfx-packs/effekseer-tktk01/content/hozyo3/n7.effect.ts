import { file } from '@origingame/engine/author';
// Effekseer node 7 "ヘックスグロー外側"
export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        1.0
      ],
      [
        0.25,
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
