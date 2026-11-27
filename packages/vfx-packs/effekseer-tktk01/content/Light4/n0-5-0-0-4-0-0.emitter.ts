import { file } from '@origingame/engine/author';
// Effekseer node 0.5.0.0.4.0.0 "Node"
export default file.particles({
  "maxParticles": 300,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 720.0,
  "duration": 0.3333,
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
    0.5,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.0,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        1.0,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
