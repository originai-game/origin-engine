import { file } from '@origingame/engine/author';
// Effekseer node 0 "リング"
export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.1,
        1.0
      ],
      [
        1.0,
        1.0
      ]
    ]
  },
  "blend": "additive",
  "doubleSided": true
});
