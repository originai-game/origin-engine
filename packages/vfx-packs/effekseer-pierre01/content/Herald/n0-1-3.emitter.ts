import { file } from '@origingame/engine/author';
// Effekseer node 0.1.3 "SplashVariation"
export default file.particles({
  "maxParticles": 25,
  "shape": {
    "kind": "point"
  },
  "spread": 50.7685,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 20,
      "interval": 0.0167
    }
  ],
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.3333,
    1.3333
  ],
  "speed": [
    4.2426,
    11.225
  ],
  "size": [
    0.5687,
    1.7062
  ],
  "sizeOverLife": [
    [
      0.0,
      0.8791
    ],
    [
      0.25,
      0.9739
    ],
    [
      0.5,
      1.0
    ],
    [
      0.75,
      0.9574
    ],
    [
      1.0,
      0.8462
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.7519,
        0.8751,
        1.0,
        0.0
      ]
    ],
    [
      0.2,
      [
        0.7519,
        0.8751,
        1.0,
        0.251
      ]
    ],
    [
      0.3,
      [
        0.7519,
        0.8751,
        1.0,
        0.3294
      ]
    ],
    [
      1.0,
      [
        0.7519,
        0.8751,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -60.0,
    60.0
  ],
  "gravity": [
    0.0,
    -10.8,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Splash"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/Herald/n0-1-3.effect"
  }
});
