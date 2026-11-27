import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 8,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    1.100000023841858,
    1.0700000524520874,
    1.0199999809265137,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0
      ]
    ],
    [
      0.07999999821186066,
      [
        1,
        1,
        1,
        0.6000000238418579
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0
      ]
    ]
  ],
  "drag": 5,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0.10000000149011612,
    0
  ],
  "lifetime": [
    0.699999988079071,
    1.2999999523162842
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 10,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 25,
    "radius": 0.029999999329447746
  },
  "sheet": {
    "blend": true,
    "columns": 16,
    "frames": 64,
    "mode": {
      "kind": "life",
      "cycles": 1
    },
    "rows": 4
  },
  "size": [
    0.11999999731779099,
    0.23999999463558197
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2.5999999046325684
    ]
  ],
  "softDistance": 0.11999999731779099,
  "speed": [
    0.15000000596046448,
    2
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
