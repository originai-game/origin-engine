import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0.19609999656677246
      ]
    ],
    [
      0.5,
      [
        1,
        1,
        1,
        0.7843000292778015
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
  "duration": 0.41670000553131104,
  "lifetime": [
    0.33329999446868896,
    0.33329999446868896
  ],
  "looping": false,
  "maxParticles": 102,
  "rate": 240,
  "rotation": [
    45,
    45
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.75,
      1,
      0.0010000000474974513
    ]
  },
  "size": [
    0.375,
    0.375
  ],
  "speed": [
    1.5,
    3
  ],
  "speedOverLife": [
    [
      0,
      0.8568000197410583
    ],
    [
      1,
      1.18149995803833
    ]
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/star")
});
