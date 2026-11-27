import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -30,
    30
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 10,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        0.20000000298023224,
        0.17000000178813934,
        0.15000000596046448,
        0
      ]
    ],
    [
      0.15000000596046448,
      [
        0.20000000298023224,
        0.17000000178813934,
        0.15000000596046448,
        0.550000011920929
      ]
    ],
    [
      1,
      [
        0.25,
        0.23000000417232513,
        0.2199999988079071,
        0
      ]
    ]
  ],
  "drag": 2.5,
  "duration": 0.10000000149011612,
  "gravity": [
    0,
    0.4000000059604645,
    0
  ],
  "lifetime": [
    0.8999999761581421,
    1.399999976158142
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 32,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "sphere",
    "radius": 0.30000001192092896,
    "surface": false
  },
  "size": [
    0.6000000238418579,
    0.8999999761581421
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2.200000047683716
    ]
  ],
  "softDistance": 0.4000000059604645,
  "speed": [
    0.30000001192092896,
    1.100000023841858
  ],
  "spread": 70,
  "texture": ref("base/textures/puff")
});
