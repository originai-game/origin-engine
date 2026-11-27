import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "bursts": [
    {
      "count": 22,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
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
      0.30000001192092896,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        1
      ]
    ]
  ],
  "duration": 0.10000000149011612,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    0.23999999463558197,
    0.23999999463558197
  ],
  "looping": false,
  "maxParticles": 32,
  "rate": 0,
  "shape": {
    "kind": "sphere",
    "radius": 150,
    "surface": true
  },
  "size": [
    8,
    14
  ],
  "sizeOverLife": [
    [
      0,
      0.4000000059604645
    ],
    [
      1,
      1
    ]
  ],
  "speed": [
    -640,
    -560
  ],
  "texture": ref("@styles/shared/fx/glint.png")
});
