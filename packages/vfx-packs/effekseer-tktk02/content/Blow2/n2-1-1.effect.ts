import { file } from '@origingame/engine/author';
// Effekseer node 2.1.1 "Ring"
export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.1111,
        1.0
      ],
      [
        0.7778,
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
