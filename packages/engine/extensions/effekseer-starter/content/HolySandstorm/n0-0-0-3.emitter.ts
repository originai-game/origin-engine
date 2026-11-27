import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -1800,
    1800
  ],
  "blend": "additive",
  "bursts": [
    {
      "count": 2,
      "cycles": 240,
      "interval": 0.016699999570846558,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        0.5356000065803528,
        0.4422999918460846,
        0.21950000524520874,
        1
      ]
    ],
    [
      0.428600013256073,
      [
        0.5356000065803528,
        0.4422999918460846,
        0.21950000524520874,
        1
      ]
    ],
    [
      1,
      [
        0.5356000065803528,
        0.4422999918460846,
        0.21950000524520874,
        0
      ]
    ]
  ],
  "duration": 4,
  "gravity": [
    13.976200103759766,
    0,
    52.15999984741211
  ],
  "lifetime": [
    1,
    1.333299994468689
  ],
  "looping": false,
  "maxParticles": 178,
  "rate": 0,
  "rotation": [
    -360,
    360
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      4.199999809265137,
      0.20000000298023224,
      0.20000000298023224
    ]
  },
  "size": [
    0.09000000357627869,
    0.3100000023841858
  ],
  "speed": [
    27.00670051574707,
    27.059900283813477
  ],
  "spread": 88.72699737548828,
  "texture": ref("base/vfx/effekseer-starter/textures/Particle_Hard")
});
