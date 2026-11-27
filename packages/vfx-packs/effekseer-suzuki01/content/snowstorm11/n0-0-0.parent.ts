import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "move_curve"
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
    1.6667,
    1.6667
  ],
  "speed": [
    17.6065,
    17.6065
  ],
  "speedOverLife": [
    [
      0.0,
      4.8016
    ],
    [
      0.2031,
      4.5738
    ],
    [
      0.2188,
      2.8266
    ],
    [
      0.2344,
      2.4205
    ],
    [
      0.25,
      0.7794
    ],
    [
      0.5,
      0.3224
    ],
    [
      0.875,
      0.8034
    ],
    [
      1.0,
      0.2973
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
        "$ogRef": "base/vfx/effekseer-suzuki01/snowstorm11/n0-0-0.emitter"
      }
    }
  ]
});
