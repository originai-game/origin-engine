import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0.0.0.0 "Light": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0.0.0.0 "Light": nested generation: 2 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0.0.1 "Tracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0.0.0.1 "Tracker": nested generation: 2 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0.0.2 "Particle": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0.0.0.2 "Particle": nested generation: 2 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0.0.2.0 "Light": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0.0.0.0.2.0 "Light": nested generation: 2 instances above its generator folded into one emitter
// effekseer: node 0.0.1.0.0.0.0 "Tracker": nested generation: 9 instances above its generator folded into one emitter
// effekseer: node 0.0.1.0.0.0.0.0 "Light": nested generation: 9 instances above its generator folded into one emitter
// effekseer: node 0.0.1.0.0.0.1 "Particle": nested generation: 9 instances above its generator folded into one emitter
// effekseer: node 0.0.1.0.0.0.1.0 "Light": nested generation: 9 instances above its generator folded into one emitter
// effekseer: node 0.0.2.0.0.0.0 "Tracker": nested generation: 30 instances above its generator folded into one emitter
// effekseer: node 0.0.2.0.0.0.0.0 "Light": nested generation: 30 instances above its generator folded into one emitter
// effekseer: node 0.0.3.0 "Debris_Variation1": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.3.0 "Debris_Variation1": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.3.1 "Debris_Variation2": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.3.1 "Debris_Variation2": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.3.2 "Debris_Variation3": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.3.2 "Debris_Variation3": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.3.3 "Debris_Variation4": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.3.3 "Debris_Variation4": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.3.5 "Burst1": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.3.5 "Burst1": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.3.6 "Burst2": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.3.6 "Burst2": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.3.7 "Burst3": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.3.7 "Burst3": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.3.8.0 "Smoke_Variation1": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.3.8.1 "Smoke_Variation2": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.3.8.2 "Smoke_Variation3": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.3.8.3 "Smoke_Variation4": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.3.9.0 "SparkTracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.3.9.0 "SparkTracker": nested generation: 40 instances above its generator folded into one emitter
// effekseer: node 0.0.3.9.0.0 "SparkLight": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.3.10 "Fire_Variation1": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.3.10 "Fire_Variation1": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.3.11 "Fire_Variation2": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.3.11 "Fire_Variation2": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.3.12 "Fire_Variation3": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.3.12 "Fire_Variation3": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.0.3.13 "Fire_Variation4": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.3.13 "Fire_Variation4": a fixed sprite turned at random: its mean plane drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-0-0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0488,
        145.9998,
        1.0
      ],
      "rotate": [
        0.0,
        0.0,
        -0.466
      ],
      "velocity": [
        0.0,
        -480.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-0-0-0-0-1.effect"
        },
        "width": [
          4.0,
          1.0
        ],
        "lifetime": 0.6667,
        "color": [
          0.945,
          0.8613,
          0.995,
          0.9977
        ],
        "orbit": [
          0.0,
          10.0
        ]
      },
      "at": 0.0,
      "offset": [
        0.0488,
        145.9998,
        1.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.466
      ],
      "velocity": [
        0.0,
        -480.0,
        0.0
      ],
      "life": 0.2658
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-0-0-0-0-2.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0488,
        145.9998,
        1.0
      ],
      "rotate": [
        0.0,
        0.0,
        -0.466
      ],
      "velocity": [
        0.0,
        -480.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-0-0-0-0-2-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0488,
        145.9998,
        1.0
      ],
      "rotate": [
        0.0,
        0.0,
        -0.466
      ],
      "velocity": [
        0.0,
        -480.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-1-0-0-0-0.effect"
        },
        "width": [
          2.0,
          0.5
        ],
        "lifetime": 0.4,
        "color": [
          0.9364,
          0.8516,
          0.9915,
          0.9961
        ],
        "orbit": [
          1.0,
          40.0
        ]
      },
      "at": 0.2,
      "offset": [
        0.0,
        -8.0,
        1.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.466
      ],
      "velocity": [
        0.0,
        240.0,
        0.0
      ],
      "life": 0.295
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-1-0-0-0-0-0.emitter"
        }
      },
      "at": 0.2,
      "offset": [
        0.0,
        -8.0,
        2.0
      ],
      "rotate": [
        0.0,
        0.0,
        -0.466
      ],
      "velocity": [
        0.0,
        240.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-1-0-0-0-1.emitter"
        }
      },
      "at": 0.2,
      "offset": [
        0.0,
        -8.0,
        1.0
      ],
      "rotate": [
        0.0,
        0.0,
        -0.466
      ],
      "velocity": [
        0.0,
        240.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-1-0-0-0-1-0.emitter"
        }
      },
      "at": 0.2,
      "offset": [
        0.0,
        -8.0,
        1.0
      ],
      "rotate": [
        0.0,
        0.0,
        -0.466
      ],
      "velocity": [
        0.0,
        240.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-2-0-0-0-0.effect"
        },
        "width": [
          1.0,
          0.25
        ],
        "lifetime": 0.075,
        "color": [
          0.825,
          0.7277,
          0.9453,
          0.9748
        ],
        "orbit": [
          1.0,
          40.0
        ]
      },
      "at": 0.5,
      "offset": [
        0.0,
        -8.0,
        1.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.466
      ],
      "velocity": [
        0.0,
        120.0,
        0.0
      ],
      "life": 0.175
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-2-0-0-0-0-0.emitter"
        }
      },
      "at": 0.5,
      "offset": [
        0.0,
        -8.0,
        2.0
      ],
      "rotate": [
        0.0,
        0.0,
        -0.466
      ],
      "velocity": [
        0.0,
        120.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-0.emitter"
        }
      },
      "at": 0.3167,
      "offset": [
        0.0,
        1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-1.emitter"
        }
      },
      "at": 0.3167,
      "offset": [
        0.0,
        1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-2.emitter"
        }
      },
      "at": 0.3167,
      "offset": [
        0.0,
        1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-3.emitter"
        }
      },
      "at": 0.3167,
      "offset": [
        0.0,
        1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-4.emitter"
        }
      },
      "at": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-5.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        0.0,
        16.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-6.emitter"
        }
      },
      "at": 0.3167,
      "offset": [
        0.0,
        10.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-7.emitter"
        }
      },
      "at": 0.35,
      "offset": [
        0.0,
        4.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-8-0.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        -88.5679,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-8-1.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        -88.5679,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-8-2.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        -65.0102,
        28.0604,
        -42.827
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-8-3.emitter"
        }
      },
      "at": 0.4,
      "offset": [
        0.0,
        0.5,
        0.0
      ],
      "rotate": [
        -65.0102,
        28.0604,
        -42.827
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-9-0.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 0.0583,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.65,
      "life": 0.9833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-9-0-0.parent"
        }
      },
      "at": 0.65
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-10.emitter"
        }
      },
      "at": 0.3167,
      "offset": [
        0.0,
        16.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-11.emitter"
        }
      },
      "at": 0.3167,
      "offset": [
        0.0,
        16.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-12.emitter"
        }
      },
      "at": 0.3167,
      "offset": [
        0.0,
        16.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-13.emitter"
        }
      },
      "at": 0.3167,
      "offset": [
        0.0,
        16.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-14.parent"
        }
      },
      "at": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-15.emitter"
        }
      },
      "at": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-4.emitter"
        }
      },
      "at": 0.3167,
      "offset": [
        0.0,
        1.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "light",
        "color": [
          0.9501,
          0.6893,
          1.0
        ],
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              8.0
            ],
            [
              1.0,
              0.0
            ]
          ]
        },
        "distance": 12.0
      },
      "at": 0.3167,
      "life": 0.3,
      "offset": [
        0.0,
        2.7787,
        0.0
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.3167
    }
  ],
  "duration": 2.7833,
  "events": [
    {
      "name": "impact",
      "time": 0.3167
    }
  ]
});
