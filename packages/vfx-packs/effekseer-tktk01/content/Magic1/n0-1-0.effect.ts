import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0 "リングの残像"
export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.0781,
        0.4602
      ],
      [
        0.125,
        0.6342
      ],
      [
        0.1719,
        0.7482
      ],
      [
        0.25,
        0.8415
      ],
      [
        0.4219,
        0.9
      ],
      [
        0.5,
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
