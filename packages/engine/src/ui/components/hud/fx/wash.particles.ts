import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "bursts": [
    {
      "count": 1,
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
        0.18000000715255737
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
  "duration": 0.10000000149011612,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    0.5,
    0.5
  ],
  "looping": false,
  "maxParticles": 2,
  "rate": 0,
  "shape": {
    "kind": "point"
  },
  "size": [
    1400,
    1400
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("@styles/shared/fx/glint.png")
});
