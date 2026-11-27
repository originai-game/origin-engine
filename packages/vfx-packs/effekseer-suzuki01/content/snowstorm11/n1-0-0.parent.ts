import { file } from '@origingame/engine/author';
// Effekseer node 1.0 "move_curve"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5,
      "cycles": 100,
      "interval": 0.0167
    }
  ],
  "duration": 1.6667,
  "looping": false,
  "lifetime": [
    2.0,
    2.0
  ],
  "speed": [
    16.1903,
    16.1903
  ],
  "speedOverLife": [
    [
      0.0,
      4.3148
    ],
    [
      0.2812,
      4.3148
    ],
    [
      0.2969,
      1.6495
    ],
    [
      0.3906,
      1.7285
    ],
    [
      0.4062,
      0.8811
    ],
    [
      0.7969,
      1.1099
    ],
    [
      0.8281,
      1.9212
    ],
    [
      1.0,
      1.9212
    ]
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-suzuki01/snowstorm11/n1-0-0.emitter"
      }
    }
  ]
});
