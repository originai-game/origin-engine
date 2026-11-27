import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 2.0 "リング白": ring 10 instances: the first 4 drawn
// effekseer: node 2.1 "リング外枠": ring 10 instances: the first 4 drawn
// effekseer: node 2.1 "リング外枠": a ring whose thickness changes over its life: its middle thickness kept
// effekseer: node 2.3 "リング赤": ring 5 instances: the first 4 drawn
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow9/n0-0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow9/n0-0-0.emitter"
        }
      },
      "at": 0.0
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow9/n0-1-0.emitter"
        }
      },
      "at": 0.0333,
      "offset": [
        1.125,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow9/n0-1-0-0.emitter"
        }
      },
      "at": 0.0333,
      "offset": [
        1.125,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow9/n0-1-0-1.emitter"
        }
      },
      "at": 0.0333,
      "offset": [
        1.125,
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
          "$ogRef": "base/vfx/effekseer-tktk01/Blow9/n1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        2.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow9/n1-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        2.0,
        0.0
      ]
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow9/n2-0.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              2.0546
            ],
            [
              0.1719,
              3.0472
            ],
            [
              0.2656,
              3.7797
            ],
            [
              0.375,
              4.3582
            ],
            [
              0.4688,
              4.6625
            ],
            [
              0.5781,
              4.8498
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          10.0,
          10.0,
          10.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.796,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow9/n2-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              2.0546
            ],
            [
              0.1719,
              3.0472
            ],
            [
              0.2656,
              3.7797
            ],
            [
              0.375,
              4.3582
            ],
            [
              0.4688,
              4.6625
            ],
            [
              0.5781,
              4.8498
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        90.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.796,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow9/n2-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              2.0546
            ],
            [
              0.1719,
              3.0472
            ],
            [
              0.2656,
              3.7797
            ],
            [
              0.375,
              4.3582
            ],
            [
              0.4688,
              4.6625
            ],
            [
              0.5781,
              4.8498
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0017,
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.796,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow9/n2-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              2.0546
            ],
            [
              0.1719,
              3.0472
            ],
            [
              0.2656,
              3.7797
            ],
            [
              0.375,
              4.3582
            ],
            [
              0.4688,
              4.6625
            ],
            [
              0.5781,
              4.8498
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0033,
      "rotate": [
        0.0,
        0.0,
        90.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.796,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow9/n2-1.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              2.0546
            ],
            [
              0.1719,
              3.0472
            ],
            [
              0.2656,
              3.7797
            ],
            [
              0.375,
              4.3582
            ],
            [
              0.4688,
              4.6625
            ],
            [
              0.5781,
              4.8498
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          2.5,
          2.5,
          2.5,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.005,
      "rotate": [
        0.0,
        0.0,
        -90.0
      ],
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow9/n2-2.emitter"
        }
      },
      "at": 0.0333
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "ring",
          "ring": {
            "inner": 0.0,
            "band": 1.0,
            "colors": [
              [
                1.0,
                1.0,
                1.0,
                0.0
              ],
              [
                1.0,
                0.0478,
                0.0104,
                0.0
              ],
              [
                1.0,
                1.0,
                1.0,
                0.0
              ]
            ]
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow9/n2-3.effect"
        },
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0.0,
              1.0
            ],
            [
              0.0781,
              2.0546
            ],
            [
              0.1719,
              3.0472
            ],
            [
              0.2656,
              3.7797
            ],
            [
              0.375,
              4.3582
            ],
            [
              0.4688,
              4.6625
            ],
            [
              0.5781,
              4.8498
            ],
            [
              1.0,
              5.0
            ]
          ]
        },
        "color": [
          5.0,
          5.0,
          5.0,
          1.0
        ],
        "blend": "additive"
      },
      "at": 0.0,
      "life": 0.5
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Blow9/n3-0.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        0.0,
        0.0,
        -90.0
      ]
    },
    {
      "node": {
        "kind": "light",
        "color": [
          1.0,
          0.9806,
          0.9799
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
      "at": 0.0,
      "life": 0.3,
      "offset": [
        0.0,
        0.0344,
        0.0
      ]
    },
    {
      "node": {
        "kind": "shake",
        "trauma": 0.4
      },
      "at": 0.0
    }
  ],
  "duration": 1.6667,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
