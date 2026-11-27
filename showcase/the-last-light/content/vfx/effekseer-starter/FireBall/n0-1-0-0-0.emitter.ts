import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 24,
      "cycles": 2,
      "interval": 0.26669999957084656,
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
        0.12549999356269836
      ]
    ],
    [
      0.33329999446868896,
      [
        1,
        0.21950000524520874,
        0.04780000075697899,
        0.20080000162124634
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
  "duration": 0.2833000123500824,
  "lifetime": [
    0.516700029373169,
    0.6833000183105469
  ],
  "looping": false,
  "maxParticles": 60,
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
      0.5
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
