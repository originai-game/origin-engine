import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "facing",
    "normal": [
      0,
      0,
      1
    ]
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 5,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        0.8751000165939331,
        0.9157999753952026,
        0.2468000054359436,
        0.7843000292778015
      ]
    ],
    [
      0.25,
      [
        0.9054999947547913,
        0.9363999962806702,
        0.3837999999523163,
        0.6373000144958496
      ]
    ],
    [
      0.5,
      [
        0.9363999962806702,
        0.9574000239372253,
        0.5541999936103821,
        0.490200012922287
      ]
    ],
    [
      0.75,
      [
        0.9678999781608582,
        0.978600025177002,
        0.7592999935150146,
        0.34310001134872437
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0.19609999656677246
      ]
    ]
  ],
  "duration": 0.016699999570846558,
  "lifetime": [
    0.5,
    0.5
  ],
  "looping": false,
  "maxParticles": 7,
  "rate": 0,
  "rotation": [
    -360,
    360
  ],
  "shape": {
    "kind": "point"
  },
  "size": [
    0.75,
    0.75
  ],
  "speed": [
    2.828399896621704,
    2.828399896621704
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Particle02-b223f51c")
});
