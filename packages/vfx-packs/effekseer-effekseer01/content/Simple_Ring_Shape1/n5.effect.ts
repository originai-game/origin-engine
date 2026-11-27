import { file } from '@origingame/engine/author';
// Effekseer node 5 "Triangle"
export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.1429,
        1.0
      ],
      [
        0.8571,
        1.0
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
