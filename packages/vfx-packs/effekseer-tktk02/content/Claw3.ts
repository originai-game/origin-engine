import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0.0.0 "poison": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0.0.0.0.0.0.0 "LineMain": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0.0.0.0.0.1 "LineColor": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0.0.0.0.0.1.0 "LineGlow": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0.0.1 "poison": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0.0.0.2 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0.0.0.3.0.0.0.0 "LineMain": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0.0.3.0.0.0.1 "LineColor": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.0.0.0.0.3.0.0.0.1.0 "LineGlow": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.1.0.0.0 "poison": motion out from a circle: approximated along one direction
// effekseer: node 0.0.1.0.0.0.0.0.0.0 "LineMain": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.1.0.0.0.0.0.0.1 "LineColor": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.1.0.0.0.0.0.0.1.0 "LineGlow": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.1.0.0.0.1 "poison": motion out from a circle: approximated along one direction
// effekseer: node 0.0.1.0.0.0.2 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.0.1.0.0.0.3.0.0.0.0 "LineMain": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.1.0.0.0.3.0.0.0.1 "LineColor": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0.1.0.0.0.3.0.0.0.1.0 "LineGlow": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.0.0 "LineMain": motion out from a circle: approximated along one direction
// effekseer: node 0.1.0.0.0.0.1.0 "LineGlow": motion out from a circle: approximated along one direction
// effekseer: node 0.1.0.1 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.1.0.2.0.0.0 "particleMain": motion out from a circle: approximated along one direction
// effekseer: node 0.1.0.2.0.0.1.0 "particleGlow": motion out from a circle: approximated along one direction
// effekseer: node 0.1.0.3.0.0.0.0 "LineMain": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.0.3.0.0.0.0 "LineMain": nested generation: 200 instances above its generator folded into one emitter
// effekseer: node 0.1.0.3.0.0.0.1 "LineColor": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.0.3.0.0.0.1 "LineColor": nested generation: 200 instances above its generator folded into one emitter
// effekseer: node 0.2.0.1 "Distortion": motion out from a circle: approximated along one direction
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-0-0-0-0.emitter"
        }
      },
      "at": 0.3,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-0-0-0-0-0-0-0-0.emitter"
        }
      },
      "at": 0.8,
      "offset": [
        0.0,
        0.8125,
        0.0
      ],
      "velocity": [
        0.0,
        1.125,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-0-0-0-0-0-0-0-1.emitter"
        }
      },
      "at": 0.8,
      "offset": [
        0.0,
        0.8125,
        0.0
      ],
      "velocity": [
        0.0,
        1.125,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-0-0-0-0-0-0-0-1-0.emitter"
        }
      },
      "at": 0.8,
      "offset": [
        0.0,
        0.8125,
        0.0
      ],
      "velocity": [
        0.0,
        1.125,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-0-0-0-0-1.emitter"
        }
      },
      "at": 0.7333,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-0-0-0-0-2.emitter"
        }
      },
      "at": 0.7333,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-0-0-0-0-3-0-0-0-0.emitter"
        }
      },
      "at": 0.8,
      "offset": [
        0.0,
        0.875,
        0.0
      ],
      "velocity": [
        0.0,
        1.125,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-0-0-0-0-3-0-0-0-1.emitter"
        }
      },
      "at": 0.8,
      "offset": [
        0.0,
        0.875,
        0.0
      ],
      "velocity": [
        0.0,
        1.125,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-0-0-0-0-3-0-0-0-1-0.emitter"
        }
      },
      "at": 0.8,
      "offset": [
        0.0,
        0.875,
        0.0
      ],
      "velocity": [
        0.0,
        1.125,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-1-0-0-0.emitter"
        }
      },
      "at": 0.0667,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-1-0-0-0-0-0-0-0.emitter"
        }
      },
      "at": 0.5667,
      "offset": [
        0.0,
        0.8125,
        0.0
      ],
      "velocity": [
        0.0,
        1.125,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-1-0-0-0-0-0-0-1.emitter"
        }
      },
      "at": 0.5667,
      "offset": [
        0.0,
        0.8125,
        0.0
      ],
      "velocity": [
        0.0,
        1.125,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-1-0-0-0-0-0-0-1-0.emitter"
        }
      },
      "at": 0.5667,
      "offset": [
        0.0,
        0.8125,
        0.0
      ],
      "velocity": [
        0.0,
        1.125,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-1-0-0-0-1.emitter"
        }
      },
      "at": 0.5,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-1-0-0-0-2.emitter"
        }
      },
      "at": 0.5,
      "offset": [
        0.0,
        0.5,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-1-0-0-0-3-0-0-0-0.emitter"
        }
      },
      "at": 0.5667,
      "offset": [
        0.0,
        0.875,
        0.0
      ],
      "velocity": [
        0.0,
        1.125,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-1-0-0-0-3-0-0-0-1.emitter"
        }
      },
      "at": 0.5667,
      "offset": [
        0.0,
        0.875,
        0.0
      ],
      "velocity": [
        0.0,
        1.125,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-0-1-0-0-0-3-0-0-0-1-0.emitter"
        }
      },
      "at": 0.5667,
      "offset": [
        0.0,
        0.875,
        0.0
      ],
      "velocity": [
        0.0,
        1.125,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-1-0-0-0-0-0.parent"
        }
      },
      "at": 0.0667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-1-0-0-0-0-1-0.parent"
        }
      },
      "at": 0.0667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-1-0-1.parent"
        }
      },
      "at": 0.0667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-1-0-2-0-0-0.parent"
        }
      },
      "at": 0.0667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-1-0-2-0-0-1-0.parent"
        }
      },
      "at": 0.0667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-1-0-3-0-0-0-0.effect"
        },
        "width": [
          1.0,
          1.0
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0667,
      "velocity": [
        0.0,
        -9.0,
        0.0
      ],
      "life": 0.25
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-1-0-3-0-0-0-1.effect"
        },
        "width": [
          5.0,
          5.0
        ],
        "lifetime": 0.4167,
        "color": [
          0.2195,
          0.0478,
          1.0,
          0.251
        ]
      },
      "at": 0.0667,
      "velocity": [
        0.0,
        -9.0,
        0.0
      ],
      "life": 0.4167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-2-0-0-0-0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n0-2-0-1.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-0-0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.7528,
        2.1667,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        120.0
      ],
      "velocity": [
        -18.2948,
        -10.5625,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-0-0-0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.7528,
        2.1667,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ],
      "velocity": [
        -18.2948,
        -10.5625,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-0-0-0-0-1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.7528,
        2.1667,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ],
      "velocity": [
        -18.1254,
        -10.4647,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-0-1-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.7971,
        2.4232,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        120.0
      ],
      "velocity": [
        -15.5885,
        -9.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-0-1-0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.7971,
        2.4232,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ],
      "velocity": [
        -15.5885,
        -9.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-0-1-0-0-1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.7971,
        2.4232,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ],
      "velocity": [
        -15.4441,
        -8.9167,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-0-2-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.9971,
        2.0768,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        120.0
      ],
      "velocity": [
        -15.5885,
        -9.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-0-2-0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.9971,
        2.0768,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ],
      "velocity": [
        -15.5885,
        -9.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-0-2-0-0-1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        3.9971,
        2.0768,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -60.0
      ],
      "velocity": [
        -15.4441,
        -8.9167,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-1-0-0-0-0.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        -3.7528,
        2.1667,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -120.0
      ],
      "velocity": [
        18.2948,
        -10.5625,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-1-0-0-0-1.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        -3.7528,
        2.1667,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ],
      "velocity": [
        18.2948,
        -10.5625,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-1-0-0-0-1-0.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        -3.7528,
        2.1667,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ],
      "velocity": [
        18.1254,
        -10.4647,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-1-1-0-0-0.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        -3.9971,
        2.0768,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -120.0
      ],
      "velocity": [
        15.5885,
        -9.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-1-1-0-0-1.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        -3.9971,
        2.0768,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ],
      "velocity": [
        15.5885,
        -9.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-1-1-0-0-1-0.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        -3.9971,
        2.0768,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ],
      "velocity": [
        15.4441,
        -8.9167,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-1-2-0-0-0.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        -3.7971,
        2.4232,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -120.0
      ],
      "velocity": [
        15.5885,
        -9.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-1-2-0-0-1.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        -3.7971,
        2.4232,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ],
      "velocity": [
        15.5885,
        -9.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw3/n1-1-2-0-0-1-0.emitter"
        }
      },
      "at": 0.1,
      "offset": [
        -3.7971,
        2.4232,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        60.0
      ],
      "velocity": [
        15.4441,
        -8.9167,
        0.0
      ]
    }
  ],
  "duration": 2.7333,
  "events": [
    {
      "name": "impact",
      "time": 0.0667
    }
  ]
});
