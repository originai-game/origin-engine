import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "alpha": 0.75,
  "blend": "alpha",
  "doubleSided": true,
  "erosion": 0.20000000298023224,
  "noise": {
    "channel": "red",
    "uv": {
      "offset": [
        0,
        0
      ],
      "polar": true,
      "scroll": [
        0.11999999731779099,
        -0.20000000298023224
      ],
      "tile": [
        2,
        1
      ],
      "rotate": 0
    }
  },
  "erosionSoftness": 0.6000000238418579,
  "erosionEdge": {
    "width": 0,
    "color": [
      1,
      1,
      1,
      1
    ]
  },
  "softDepth": 0.05000000074505806
});
