import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "alpha": 0.5,
  "blend": "additive",
  "doubleSided": true,
  "erosion": 0.25,
  "noise": {
    "channel": "red",
    "uv": {
      "offset": [
        0,
        0
      ],
      "polar": false,
      "scroll": [
        0.029999999329447746,
        0.07999999821186066
      ],
      "tile": [
        3,
        2
      ],
      "rotate": 0
    }
  },
  "erosionSoftness": 0.5,
  "erosionEdge": {
    "width": 0,
    "color": [
      1,
      1,
      1,
      1
    ]
  },
  "fresnel": {
    "invert": false,
    "power": 3,
    "strength": 1.600000023841858
  },
  "softDepth": 0.07999999821186066
});
