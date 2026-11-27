import { file } from '@origingame/engine/author';
// Effekseer node 5 "Egg Zero"
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
    1.6667,
    1.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.4612,
    0.4612
  ],
  "sizeOverLife": [
    [
      0.0,
      0.8236
    ],
    [
      0.4531,
      1.0037
    ],
    [
      1.0,
      0.766
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/eggzero"
  },
  "blend": "alpha"
});
