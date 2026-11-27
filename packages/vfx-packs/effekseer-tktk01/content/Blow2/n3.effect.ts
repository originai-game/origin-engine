import { file } from '@origingame/engine/author';
// Effekseer node 3 "大リング"
export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.0938,
        1.0
      ],
      [
        0.625,
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
