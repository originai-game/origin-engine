import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.4 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.1.0.0.0.0.0.0 "LineMain": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.0.0.1.0 "Line_color": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.0.0.2.0 "Line_Glow": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.0.0.3.0.0.0.0 "particleMain": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.0.0.3.0.0.0.1 "particleColor": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.0.0.3.0.0.0.2 "particleGlow": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.1.0.0 "LineMain": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.1.0.1.0 "Line_color": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.1.0.2.0 "Line_Glow": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.1.0.3.0.0.0.0 "particleMain": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.1.0.3.0.0.0.1 "particleColor": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.1.0.3.0.0.0.2 "particleGlow": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.2.0.0 "LineMain": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.2.0.1.0 "Line_color": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.2.0.2.0 "Line_Glow": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.2.0.3.0.0.0.0 "particleMain": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.2.0.3.0.0.0.1 "particleColor": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.0.0.0.2.0.3.0.0.0.2 "particleGlow": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.0.0.0 "LineMain": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.0.0.1.0 "Line_color": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.0.0.2.0 "Line_Glow": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.0.0.3.0.0.0.0 "particleMain": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.0.0.3.0.0.0.1 "particleColor": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.0.0.3.0.0.0.2 "particleGlow": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.1.0.0 "LineMain": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.1.0.1.0 "Line_color": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.1.0.2.0 "Line_Glow": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.1.0.3.0.0.0.0 "particleMain": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.1.0.3.0.0.0.1 "particleColor": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.1.0.3.0.0.0.2 "particleGlow": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.2.0.0 "LineMain": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.2.0.1.0 "Line_color": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.2.0.2.0 "Line_Glow": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.2.0.3.0.0.0.0 "particleMain": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.2.0.3.0.0.0.1 "particleColor": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.1.0.0.2.0.3.0.0.0.2 "particleGlow": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 1.0.0.0.4 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 1.0.1.0.0.0.0.0.0 "LineMain": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.0.0.1.0 "Line_color": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.0.0.2.0 "Line_Glow": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.0.0.3.0.0.0.0 "particleMain": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.0.0.3.0.0.0.1 "particleColor": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.0.0.3.0.0.0.2 "particleGlow": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.1.0.0 "LineMain": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.1.0.1.0 "Line_color": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.1.0.2.0 "Line_Glow": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.1.0.3.0.0.0.0 "particleMain": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.1.0.3.0.0.0.1 "particleColor": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.1.0.3.0.0.0.2 "particleGlow": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.2.0.0 "LineMain": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.2.0.1.0 "Line_color": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.2.0.2.0 "Line_Glow": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.2.0.3.0.0.0.0 "particleMain": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.2.0.3.0.0.0.1 "particleColor": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 1.0.1.0.0.0.2.0.3.0.0.0.2 "particleGlow": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 2.0.0.0.3 "Glow": motion out from a circle: approximated along one direction
// effekseer: node 2.0.0.0.4 "Distortion": motion out from a circle: approximated along one direction
// effekseer: node 2.0.1.0.0.0.0.0.0 "LineMain": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.0.0.1.0 "Line_color": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.0.0.2.0 "Line_Glow": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.0.0.3.0.0.0.0 "particleMain": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.0.0.3.0.0.0.1 "particleColor": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.0.0.3.0.0.0.2 "particleGlow": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.1.0.0 "LineMain": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.1.0.1.0 "Line_color": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.1.0.2.0 "Line_Glow": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.1.0.3.0.0.0.0 "particleMain": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.1.0.3.0.0.0.1 "particleColor": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.1.0.3.0.0.0.2 "particleGlow": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.2.0.0 "LineMain": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.2.0.1.0 "Line_color": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.2.0.2.0 "Line_Glow": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.2.0.3.0.0.0.0 "particleMain": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.2.0.3.0.0.0.1 "particleColor": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.0.1.0.0.0.2.0.3.0.0.0.2 "particleGlow": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 2.1 "Distortion": motion out from a circle: approximated along one direction
// effekseer: more than 64 elements: the last 43 left out
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-0-0-0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-0-0-0-0-1.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-0-0-0-0-1-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-0-1-0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-0-1-0-0-1.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-0-1-0-0-1-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-0-2-0-0-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-0-2-0-0-1.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-0-2-0-0-1-0.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-0-3.emitter"
        }
      },
      "at": 0.0017
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-0-4.parent"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-0-0-0.effect"
        },
        "width": [
          1.9444,
          1.9444
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.3333
        ]
      },
      "at": 0.0,
      "offset": [
        -1.2794,
        -0.6062,
        -0.4119
      ],
      "rotate": [
        -34.3235,
        77.6576,
        -142.278
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-0-0-1-0.effect"
        },
        "width": [
          3.8889,
          3.8889
        ],
        "lifetime": 0.4167,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.3333
        ]
      },
      "at": 0.0,
      "offset": [
        -1.2794,
        -0.6062,
        -0.4119
      ],
      "rotate": [
        -34.3235,
        77.6576,
        -142.278
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-0-0-2-0.effect"
        },
        "width": [
          15.5556,
          15.5556
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.3333
        ]
      },
      "at": 0.0,
      "offset": [
        -1.2794,
        -0.6062,
        -0.4119
      ],
      "rotate": [
        -34.3235,
        77.6576,
        -142.278
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-0-0-3-0-0-0-0.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        7.5916,
        0.4172,
        2.6695
      ],
      "rotate": [
        -34.3235,
        77.6576,
        127.7219
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-0-0-3-0-0-0-1.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        7.5916,
        0.4172,
        2.6695
      ],
      "rotate": [
        -34.3235,
        77.6576,
        127.7219
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-0-0-3-0-0-0-2.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        7.5916,
        0.4172,
        2.6695
      ],
      "rotate": [
        -34.3235,
        77.6576,
        127.7219
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-1-0-0.effect"
        },
        "width": [
          1.9444,
          1.9444
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0,
      "offset": [
        -1.1243,
        -1.8001,
        0.342
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-1-0-1-0.effect"
        },
        "width": [
          3.8889,
          3.8889
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0,
      "offset": [
        -1.1243,
        -1.8001,
        0.342
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-1-0-2-0.effect"
        },
        "width": [
          15.5556,
          15.5556
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0,
      "offset": [
        -1.1243,
        -1.8001,
        0.342
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-1-0-3-0-0-0-0.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        8.2387,
        -0.5561,
        0.1872
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-1-0-3-0-0-0-1.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        8.2387,
        -0.5561,
        0.1872
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-1-0-3-0-0-0-2.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        8.2387,
        -0.5561,
        0.1872
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-2-0-0.effect"
        },
        "width": [
          1.9444,
          1.9444
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0,
      "offset": [
        -1.4294,
        0.5105,
        0.4531
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-2-0-1-0.effect"
        },
        "width": [
          3.8889,
          3.8889
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0,
      "offset": [
        -1.4294,
        0.5105,
        0.4531
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-2-0-2-0.effect"
        },
        "width": [
          15.5556,
          15.5556
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0,
      "offset": [
        -1.4294,
        0.5105,
        0.4531
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-2-0-3-0-0-0-0.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        7.9336,
        1.7545,
        0.2983
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-2-0-3-0-0-0-1.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        7.9336,
        1.7545,
        0.2983
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-0-0-0-2-0-3-0-0-0-2.emitter"
        }
      },
      "at": 0.0167,
      "offset": [
        7.9336,
        1.7545,
        0.2983
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-0-0-0.effect"
        },
        "width": [
          1.9444,
          1.9444
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.3333
        ]
      },
      "at": 0.0667,
      "offset": [
        -1.2794,
        -0.6062,
        -0.4119
      ],
      "rotate": [
        -34.3235,
        77.6576,
        -142.278
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-0-0-1-0.effect"
        },
        "width": [
          3.8889,
          3.8889
        ],
        "lifetime": 0.4167,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.3333
        ]
      },
      "at": 0.0667,
      "offset": [
        -1.2794,
        -0.6062,
        -0.4119
      ],
      "rotate": [
        -34.3235,
        77.6576,
        -142.278
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-0-0-2-0.effect"
        },
        "width": [
          15.5556,
          15.5556
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.3333
        ]
      },
      "at": 0.0667,
      "offset": [
        -1.2794,
        -0.6062,
        -0.4119
      ],
      "rotate": [
        -34.3235,
        77.6576,
        -142.278
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-0-0-3-0-0-0-0.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        7.5916,
        0.4172,
        2.6695
      ],
      "rotate": [
        -34.3235,
        77.6576,
        127.7219
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-0-0-3-0-0-0-1.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        7.5916,
        0.4172,
        2.6695
      ],
      "rotate": [
        -34.3235,
        77.6576,
        127.7219
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-0-0-3-0-0-0-2.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        7.5916,
        0.4172,
        2.6695
      ],
      "rotate": [
        -34.3235,
        77.6576,
        127.7219
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-1-0-0.effect"
        },
        "width": [
          1.9444,
          1.9444
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0667,
      "offset": [
        -1.1243,
        -1.8001,
        0.342
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-1-0-1-0.effect"
        },
        "width": [
          3.8889,
          3.8889
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0667,
      "offset": [
        -1.1243,
        -1.8001,
        0.342
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-1-0-2-0.effect"
        },
        "width": [
          15.5556,
          15.5556
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0667,
      "offset": [
        -1.1243,
        -1.8001,
        0.342
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-1-0-3-0-0-0-0.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        8.2387,
        -0.5561,
        0.1872
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-1-0-3-0-0-0-1.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        8.2387,
        -0.5561,
        0.1872
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-1-0-3-0-0-0-2.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        8.2387,
        -0.5561,
        0.1872
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-2-0-0.effect"
        },
        "width": [
          1.9444,
          1.9444
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0667,
      "offset": [
        -1.4294,
        0.5105,
        0.4531
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-2-0-1-0.effect"
        },
        "width": [
          3.8889,
          3.8889
        ],
        "lifetime": 0.25,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0667,
      "offset": [
        -1.4294,
        0.5105,
        0.4531
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-2-0-2-0.effect"
        },
        "width": [
          15.5556,
          15.5556
        ],
        "lifetime": 0.3333,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.0556
        ]
      },
      "at": 0.0667,
      "offset": [
        -1.4294,
        0.5105,
        0.4531
      ],
      "rotate": [
        -141.1841,
        77.3697,
        -36.7342
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-2-0-3-0-0-0-0.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        7.9336,
        1.7545,
        0.2983
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-2-0-3-0-0-0-1.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        7.9336,
        1.7545,
        0.2983
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n0-1-1-0-0-2-0-3-0-0-0-2.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        7.9336,
        1.7545,
        0.2983
      ],
      "rotate": [
        -141.1841,
        77.3698,
        -126.7342
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-0-0-0-0-0-0.parent"
        }
      },
      "at": 1.1
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-0-0-0-0-0-1.parent"
        }
      },
      "at": 1.1
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-0-0-0-0-0-1-0.parent"
        }
      },
      "at": 1.1
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-0-0-1-0-0-0.parent"
        }
      },
      "at": 1.1
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-0-0-1-0-0-1.parent"
        }
      },
      "at": 1.1
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-0-0-1-0-0-1-0.parent"
        }
      },
      "at": 1.1
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-0-0-2-0-0-0.parent"
        }
      },
      "at": 1.1
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-0-0-2-0-0-1.parent"
        }
      },
      "at": 1.1
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-0-0-2-0-0-1-0.parent"
        }
      },
      "at": 1.1
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-0-0-3.emitter"
        }
      },
      "at": 1.2333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-0-0-4.parent"
        }
      },
      "at": 1.1
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-1-0-0-0-0-0-0.effect"
        },
        "width": [
          1.9444,
          1.9444
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.3333
        ]
      },
      "at": 1.1333,
      "offset": [
        -1.2794,
        -0.6062,
        -0.4119
      ],
      "rotate": [
        -34.3235,
        77.6576,
        -142.278
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-1-0-0-0-0-0-1-0.effect"
        },
        "width": [
          3.8889,
          3.8889
        ],
        "lifetime": 0.4167,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.3333
        ]
      },
      "at": 1.1333,
      "offset": [
        -1.2794,
        -0.6062,
        -0.4119
      ],
      "rotate": [
        -34.3235,
        77.6576,
        -142.278
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-1-0-0-0-0-0-2-0.effect"
        },
        "width": [
          15.5556,
          15.5556
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          9.3333,
          -1.3333
        ]
      },
      "at": 1.1333,
      "offset": [
        -1.2794,
        -0.6062,
        -0.4119
      ],
      "rotate": [
        -34.3235,
        77.6576,
        -142.278
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-1-0-0-0-0-0-3-0-0-0-0.emitter"
        }
      },
      "at": 1.15,
      "offset": [
        7.5916,
        0.4172,
        2.6695
      ],
      "rotate": [
        -34.3235,
        77.6576,
        127.7219
      ],
      "until": 1.5667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-1-0-0-0-0-0-3-0-0-0-1.emitter"
        }
      },
      "at": 1.15,
      "offset": [
        7.5916,
        0.4172,
        2.6695
      ],
      "rotate": [
        -34.3235,
        77.6576,
        127.7219
      ],
      "until": 1.5667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw4/n1-0-1-0-0-0-0-0-3-0-0-0-2.emitter"
        }
      },
      "at": 1.15,
      "offset": [
        7.5916,
        0.4172,
        2.6695
      ],
      "rotate": [
        -34.3235,
        77.6576,
        127.7219
      ],
      "until": 1.5667
    }
  ],
  "duration": 3.5333,
  "looping": true
});
