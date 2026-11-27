import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0.0.0 "particleMain": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0.1 "particleColor": motion out from a circle: approximated along one direction
// effekseer: node 0.0.0.0.1.0 "particleGlow": motion out from a circle: approximated along one direction
// effekseer: node 0.1 "particle_Distortion": motion out from a circle: approximated along one direction
// effekseer: node 0.3 "Glow_Distortion": motion out from a circle: approximated along one direction
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n0-0-0-0-0.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        1.5,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n0-0-0-0-1.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        1.5,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n0-0-0-0-1-0.emitter"
        }
      },
      "at": 0.1667,
      "offset": [
        1.5,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n0-1.emitter"
        }
      },
      "at": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n0-2.emitter"
        }
      },
      "at": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n0-3.emitter"
        }
      },
      "at": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n0-4.emitter"
        }
      },
      "at": 0.1667
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-0-0-0-0.effect"
        },
        "width": [
          1.6667,
          1.6667
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          8.0,
          -1.4444
        ]
      },
      "at": 0.0,
      "offset": [
        -1.2649,
        -0.6981,
        -1.9526
      ],
      "rotate": [
        -5.7943,
        -11.101,
        -116.9144
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-0-0-0-1.effect"
        },
        "width": [
          3.3333,
          3.3333
        ],
        "lifetime": 0.6667,
        "color": [
          1.0,
          0.2195,
          0.0478,
          1.0
        ],
        "orbit": [
          8.0,
          -1.4444
        ]
      },
      "at": 0.0,
      "offset": [
        -1.2649,
        -0.6981,
        -1.9526
      ],
      "rotate": [
        -5.7943,
        -11.101,
        -116.9144
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-0-0-0-2.effect"
        },
        "width": [
          13.3333,
          13.3333
        ],
        "lifetime": 0.8333,
        "color": [
          1.0,
          0.0478,
          0.0104,
          0.502
        ],
        "orbit": [
          8.0,
          -1.4444
        ]
      },
      "at": 0.0,
      "offset": [
        -1.2649,
        -0.6981,
        -1.9526
      ],
      "rotate": [
        -5.7943,
        -11.101,
        -116.9144
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-0-0-0-3-0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        -3.5456,
        -1.3988,
        5.8632
      ],
      "rotate": [
        -5.7943,
        -11.101,
        153.0856
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-0-0-0-3-0-0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        -3.5456,
        -1.3988,
        5.8632
      ],
      "rotate": [
        -5.7943,
        -11.101,
        153.0856
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-0-0-0-3-0-0-0-2.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        -3.5456,
        -1.3988,
        5.8632
      ],
      "rotate": [
        -5.7943,
        -11.101,
        153.0856
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-1-0-0-0.effect"
        },
        "width": [
          0.6111,
          0.6111
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          3.6667,
          -1.3333
        ]
      },
      "at": 0.0,
      "offset": [
        -1.273,
        -0.6064,
        2.2637
      ],
      "rotate": [
        3.4509,
        -15.4708,
        -114.7838
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-1-0-0-1.effect"
        },
        "width": [
          1.2222,
          1.2222
        ],
        "lifetime": 0.6667,
        "color": [
          1.0,
          0.2195,
          0.0478,
          1.0
        ],
        "orbit": [
          3.6667,
          -1.3333
        ]
      },
      "at": 0.0,
      "offset": [
        -1.273,
        -0.6064,
        2.2637
      ],
      "rotate": [
        3.4509,
        -15.4708,
        -114.7838
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-1-0-0-2.effect"
        },
        "width": [
          4.5833,
          4.5833
        ],
        "lifetime": 0.8333,
        "color": [
          1.0,
          0.0478,
          0.0104,
          0.502
        ],
        "orbit": [
          3.6667,
          -1.3333
        ]
      },
      "at": 0.0,
      "offset": [
        -1.273,
        -0.6064,
        2.2637
      ],
      "rotate": [
        3.4509,
        -15.4708,
        -114.7838
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-1-0-0-3-0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        -2.5597,
        -1.5062,
        5.6641
      ],
      "rotate": [
        3.4509,
        -15.4708,
        155.2162
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-1-0-0-3-0-0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        -2.5597,
        -1.5062,
        5.6641
      ],
      "rotate": [
        3.4509,
        -15.4708,
        155.2162
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-1-0-0-3-0-0-0-2.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        -2.5597,
        -1.5062,
        5.6641
      ],
      "rotate": [
        3.4509,
        -15.4708,
        155.2162
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-2-0-0-0.effect"
        },
        "width": [
          0.6111,
          0.6111
        ],
        "lifetime": 0.5,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "orbit": [
          3.6667,
          -1.3333
        ]
      },
      "at": 0.0,
      "offset": [
        -3.0051,
        -0.6064,
        1.2637
      ],
      "rotate": [
        3.4509,
        -15.4708,
        -114.7838
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-2-0-0-1.effect"
        },
        "width": [
          1.2222,
          1.2222
        ],
        "lifetime": 0.6667,
        "color": [
          1.0,
          0.2195,
          0.0478,
          1.0
        ],
        "orbit": [
          3.6667,
          -1.3333
        ]
      },
      "at": 0.0,
      "offset": [
        -3.0051,
        -0.6064,
        1.2637
      ],
      "rotate": [
        3.4509,
        -15.4708,
        -114.7838
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-2-0-0-2.effect"
        },
        "width": [
          4.5833,
          4.5833
        ],
        "lifetime": 0.8333,
        "color": [
          1.0,
          0.0478,
          0.0104,
          0.502
        ],
        "orbit": [
          3.6667,
          -1.3333
        ]
      },
      "at": 0.0,
      "offset": [
        -3.0051,
        -0.6064,
        1.2637
      ],
      "rotate": [
        3.4509,
        -15.4708,
        -114.7838
      ],
      "life": 0.3167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-2-0-0-3-0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        -4.2917,
        -1.5062,
        4.6641
      ],
      "rotate": [
        3.4509,
        -15.4708,
        155.2162
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-2-0-0-3-0-0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        -4.2917,
        -1.5062,
        4.6641
      ],
      "rotate": [
        3.4509,
        -15.4708,
        155.2162
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-2-0-0-3-0-0-0-2.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        -4.2917,
        -1.5062,
        4.6641
      ],
      "rotate": [
        3.4509,
        -15.4708,
        155.2162
      ],
      "until": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk02/Claw1/n1-0-3-0-0-0-0-0-0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        -3.1754,
        -0.6522,
        5.8604
      ],
      "rotate": [
        -5.7943,
        -11.101,
        153.0856
      ],
      "until": 0.5
    }
  ],
  "duration": 1.15,
  "looping": true
});
