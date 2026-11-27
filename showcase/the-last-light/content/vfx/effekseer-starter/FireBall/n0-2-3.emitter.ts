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
      0.11110000312328339,
      [
        1,
        0.21950000524520874,
        0.04780000075697899,
        0.09560000151395798
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
    0.30000001192092896,
    0.30000001192092896
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
      1
    ],
    [
      1,
      0.08330000191926956
    ]
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Particle_Soft")
});
