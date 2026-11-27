import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        0,
        1,
        1,
        0.5019999742507935
      ]
    ],
    [
      0.583299994468689,
      [
        0,
        1,
        1,
        0.5019999742507935
      ]
    ],
    [
      1,
      [
        0,
        1,
        1,
        0
      ]
    ]
  ],
  "duration": 0.04170000180602074,
  "lifetime": [
    0.4000000059604645,
    0.4000000059604645
  ],
  "looping": false,
  "maxParticles": 375,
  "rate": 7200,
  "shape": {
    "kind": "point"
  },
  "size": [
    0.5,
    0.5
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Particle02-b223f51c")
});
