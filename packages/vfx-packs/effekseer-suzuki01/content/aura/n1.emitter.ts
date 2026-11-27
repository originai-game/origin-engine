import { file } from '@origingame/engine/author';
// Effekseer node 1 "Glow"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.1667,
    0.1667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    3.5,
    3.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0546,
        0.0658,
        0.0582,
        0.7843
      ]
    ],
    [
      1.0,
      [
        0.0546,
        0.0658,
        0.0582,
        0.7843
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.4313,
        0.4591,
        0.673,
        0.7843
      ]
    ],
    [
      1.0,
      [
        0.4313,
        0.4591,
        0.673,
        0.7843
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-suzuki01/textures/Particle01"
  },
  "blend": "additive"
});
