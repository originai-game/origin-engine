import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 28,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        4,
        1.600000023841858,
        0.30000001192092896,
        1
      ]
    ],
    [
      1,
      [
        1.5,
        0.30000001192092896,
        0.05000000074505806,
        0
      ]
    ]
  ],
  "drag": 2,
  "duration": 0.10000000149011612,
  "gravity": [
    0,
    0.800000011920929,
    0
  ],
  "lifetime": [
    0.800000011920929,
    1.5
  ],
  "looping": false,
  "maxParticles": 64,
  "rate": 0,
  "shape": {
    "kind": "sphere",
    "radius": 0.3499999940395355,
    "surface": false
  },
  "size": [
    0.029999999329447746,
    0.07000000029802322
  ],
  "speed": [
    0.4000000059604645,
    1.7999999523162842
  ],
  "spread": 180,
  "texture": ref("base/textures/spark")
});
