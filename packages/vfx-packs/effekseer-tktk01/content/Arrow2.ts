import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0.0 "矢": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.1.0 "矢": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.2.0 "矢": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.3.0 "矢": a sprite scaled unevenly: drawn square at its mean
// effekseer: node 0.4.0 "矢": a sprite scaled unevenly: drawn square at its mean
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-0.effect"
        },
        "width": [
          0.5,
          0.125
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0,
      "life": 0.1667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-0-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5811
            ],
            [
              0.4375,
              1.1859
            ],
            [
              0.6094,
              0.9882
            ],
            [
              0.75,
              0.7906
            ],
            [
              0.8594,
              0.5929
            ],
            [
              0.9375,
              0.3953
            ],
            [
              0.9844,
              0.1976
            ],
            [
              1.0,
              0.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        -180.0,
        0.0,
        -90.0
      ],
      "velocity": [
        -5.4,
        0.0,
        0.0
      ],
      "life": 0.8333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-0-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          1.0,
          0.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        -180.0,
        0.0,
        -90.0
      ],
      "velocity": [
        -2.7,
        0.0,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-0-1-0.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        0.25,
        0.0,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-0-1-0-0.emitter"
        }
      },
      "at": 0.0833,
      "offset": [
        0.25,
        0.0,
        0.0
      ],
      "rotate": [
        90.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-0-1-1.emitter"
        }
      },
      "at": 0.0833,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-0-1-1-0.emitter"
        }
      },
      "at": 0.0833,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-0-2.emitter"
        }
      },
      "at": 0.1167,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-1.effect"
        },
        "width": [
          0.5,
          0.125
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.0833,
      "life": 0.1667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-1-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5811
            ],
            [
              0.4375,
              1.1859
            ],
            [
              0.6094,
              0.9882
            ],
            [
              0.75,
              0.7906
            ],
            [
              0.8594,
              0.5929
            ],
            [
              0.9375,
              0.3953
            ],
            [
              0.9844,
              0.1976
            ],
            [
              1.0,
              0.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0833,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        -180.0,
        0.0,
        -90.0
      ],
      "velocity": [
        -6.0,
        0.0,
        0.0
      ],
      "life": 0.75
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-1-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          0.0,
          1.0,
          0.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0833,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        -180.0,
        0.0,
        -90.0
      ],
      "velocity": [
        -3.0,
        0.0,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-1-1-0.emitter"
        }
      },
      "at": 0.1667,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-1-1-0-0.emitter"
        }
      },
      "at": 0.1667,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-1-1-1.emitter"
        }
      },
      "at": 0.1667,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-1-1-1-0.emitter"
        }
      },
      "at": 0.1667,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-1-2.emitter"
        }
      },
      "at": 0.2,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-2.effect"
        },
        "width": [
          0.5,
          0.125
        ],
        "lifetime": 0.1667,
        "color": [
          0.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.1667,
      "life": 0.1667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5811
            ],
            [
              0.4375,
              1.1859
            ],
            [
              0.6094,
              0.9882
            ],
            [
              0.75,
              0.7906
            ],
            [
              0.8594,
              0.5929
            ],
            [
              0.9375,
              0.3953
            ],
            [
              0.9844,
              0.1976
            ],
            [
              1.0,
              0.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1667,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        -180.0,
        0.0,
        -90.0
      ],
      "velocity": [
        -6.75,
        0.0,
        0.0
      ],
      "life": 0.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-2-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          0.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.1667,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        -180.0,
        0.0,
        -90.0
      ],
      "velocity": [
        -3.375,
        0.0,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-2-1-0.emitter"
        }
      },
      "at": 0.25,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-2-1-0-0.emitter"
        }
      },
      "at": 0.25,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-2-1-1.emitter"
        }
      },
      "at": 0.25,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-2-1-1-0.emitter"
        }
      },
      "at": 0.25,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-2-2.emitter"
        }
      },
      "at": 0.2833,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-3.effect"
        },
        "width": [
          0.5,
          0.125
        ],
        "lifetime": 0.1667,
        "color": [
          0.0,
          0.0,
          1.0,
          1.0
        ]
      },
      "at": 0.25,
      "life": 0.1667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-3-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5811
            ],
            [
              0.4375,
              1.1859
            ],
            [
              0.6094,
              0.9882
            ],
            [
              0.75,
              0.7906
            ],
            [
              0.8594,
              0.5929
            ],
            [
              0.9375,
              0.3953
            ],
            [
              0.9844,
              0.1976
            ],
            [
              1.0,
              0.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.25,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        -180.0,
        0.0,
        -90.0
      ],
      "velocity": [
        -7.7143,
        0.0,
        0.0
      ],
      "life": 0.5833
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-3-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          0.0,
          0.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.25,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        -180.0,
        0.0,
        -90.0
      ],
      "velocity": [
        -3.8571,
        0.0,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-3-1-0.emitter"
        }
      },
      "at": 0.3333,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-3-1-0-0.emitter"
        }
      },
      "at": 0.3333,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-3-1-1.emitter"
        }
      },
      "at": 0.3333,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-3-1-1-0.emitter"
        }
      },
      "at": 0.3333,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-3-2.emitter"
        }
      },
      "at": 0.3667,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-4.effect"
        },
        "width": [
          0.5,
          0.125
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.3333,
      "life": 0.1667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-4-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.5811
            ],
            [
              0.4375,
              1.1859
            ],
            [
              0.6094,
              0.9882
            ],
            [
              0.75,
              0.7906
            ],
            [
              0.8594,
              0.5929
            ],
            [
              0.9375,
              0.3953
            ],
            [
              0.9844,
              0.1976
            ],
            [
              1.0,
              0.0
            ]
          ]
        },
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.3333,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        -180.0,
        0.0,
        -90.0
      ],
      "velocity": [
        -9.0,
        0.0,
        0.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "quad"
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-4-0-0.effect"
        },
        "scale": 2.0,
        "color": [
          1.0,
          0.0,
          0.2195,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.3333,
      "offset": [
        5.0,
        0.0,
        0.0
      ],
      "rotate": [
        -180.0,
        0.0,
        -90.0
      ],
      "velocity": [
        -4.5,
        0.0,
        0.0
      ],
      "life": 1.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-4-1-0.emitter"
        }
      },
      "at": 0.4167,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-4-1-0-0.emitter"
        }
      },
      "at": 0.4167,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-4-1-1.emitter"
        }
      },
      "at": 0.4167,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-4-1-1-0.emitter"
        }
      },
      "at": 0.4167,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Arrow2/n0-4-2.emitter"
        }
      },
      "at": 0.45,
      "rotate": [
        90.0,
        0.0,
        0.0
      ]
    }
  ],
  "duration": 2.0915,
  "events": [
    {
      "name": "impact",
      "time": 0.0833
    }
  ]
});
