import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 300,
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
        0.10000000149011612
      ]
    ],
    [
      0.4530999958515167,
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
  "duration": 0.016699999570846558,
  "lifetime": [
    0.16670000553131104,
    0.5
  ],
  "looping": false,
  "maxParticles": 375,
  "rate": 0,
  "shape": {
    "kind": "point"
  },
  "size": [
    2,
    2
  ],
  "sizeOverLife": [
    [
      0,
      1
    ],
    [
      0.09380000084638596,
      0.8138999938964844
    ],
    [
      0.20309999585151672,
      0.6316999793052673
    ],
    [
      0.3125,
      0.48260000348091125
    ],
    [
      0.4219000041484833,
      0.3617999851703644
    ],
    [
      0.5311999917030334,
      0.2644999921321869
    ],
    [
      0.6561999917030334,
      0.17599999904632568
    ],
    [
      1,
      0
    ]
  ],
  "speed": [
    3,
    21
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Particle01-b96f716d")
});
