import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 1,
      "cycles": 3,
      "interval": 0.016699999570846558,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        0.21950000524520874,
        0.04780000075697899,
        0
      ]
    ],
    [
      0.16670000553131104,
      [
        1,
        0.21950000524520874,
        0.04780000075697899,
        0.5019999742507935
      ]
    ],
    [
      1,
      [
        1,
        0.21950000524520874,
        0.04780000075697899,
        0
      ]
    ]
  ],
  "duration": 0.05000000074505806,
  "lifetime": [
    0.30000001192092896,
    0.30000001192092896
  ],
  "looping": false,
  "maxParticles": 4,
  "rate": 0,
  "shape": {
    "kind": "point"
  },
  "size": [
    8,
    8
  ],
  "sizeOverLife": [
    [
      0,
      0.25
    ],
    [
      1,
      1
    ]
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Particle_Soft")
});
