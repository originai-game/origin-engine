import { file } from '@origingame/engine/author';
// Effekseer node 1.2 "衝撃グロー収縮"
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
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    7.0,
    7.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.3438,
      0.824
    ],
    [
      0.4688,
      0.7355
    ],
    [
      0.5781,
      0.6382
    ],
    [
      0.6875,
      0.5174
    ],
    [
      0.7969,
      0.3683
    ],
    [
      0.9062,
      0.1861
    ],
    [
      1.0,
      0.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        1.0,
        0.2195,
        0.3922
      ]
    ],
    [
      0.5,
      [
        0.1166,
        1.0,
        0.1166,
        0.1961
      ]
    ],
    [
      1.0,
      [
        0.0478,
        1.0,
        0.0478,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
