import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0,
        1
      ],
      [
        0.25,
        1
      ],
      [
        1,
        0
      ]
    ]
  },
  "blend": "additive",
  "doubleSided": true
});
