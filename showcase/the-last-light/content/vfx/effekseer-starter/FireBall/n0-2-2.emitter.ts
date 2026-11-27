import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 9,
      "cycles": 1,
      "interval": 0,
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
      0.06669999659061432,
      [
        1,
        0.21950000524520874,
        0.04780000075697899,
        0.12549999356269836
      ]
    ],
    [
      0.5,
      [
        1,
        0.21950000524520874,
        0.04780000075697899,
        0.12549999356269836
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
  "duration": 0.016699999570846558,
  "lifetime": [
    0.5,
    0.5
  ],
  "looping": false,
  "maxParticles": 12,
  "rate": 0,
  "rotation": [
    -360,
    360
  ],
  "shape": {
    "kind": "point"
  },
  "size": [
    36,
    36
  ],
  "sizeOverLife": [
    [
      0,
      0.16670000553131104
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
