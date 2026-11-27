import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 0.5666999816894531,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "params": [],
  "elements": [
    {
      "at": 0.05000000074505806,
      "life": 0.25,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "subtractive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Attack_Impact/n0.effect"),
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.399999976158142
            ],
            [
              1,
              1.7999999523162842
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                1,
                1,
                0
              ],
              [
                1,
                1,
                1,
                0.25099998712539673
              ],
              [
                1,
                1,
                1,
                0.25099998712539673
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0,
            "length": 360,
            "segments": 64,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 0.08330000191926956,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Attack_Impact/n1.effect"),
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              3.240000009536743
            ],
            [
              1,
              0
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                0.5356000065803528,
                0.5356000065803528,
                0
              ],
              [
                1,
                0.5356000065803528,
                0.5356000065803528,
                0.3765000104904175
              ],
              [
                1,
                0.5356000065803528,
                0.5356000065803528,
                0
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.6111000180244446,
            "length": 360,
            "segments": 64,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Attack_Impact/n2.emitter")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Attack_Impact/n3.emitter")
      }
    },
    {
      "at": 0,
      "life": 0.16670000553131104,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Attack_Impact/n4.effect"),
        "faceCamera": true,
        "scale": 1,
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                0.5356000065803528,
                0.5356000065803528,
                0
              ],
              [
                1,
                0.5356000065803528,
                0.5356000065803528,
                0.25099998712539673
              ],
              [
                1,
                0.5356000065803528,
                1,
                0.25099998712539673
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0,
            "length": 90,
            "segments": 64,
            "start": 225
          }
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Attack_Impact/n5.emitter")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Attack_Impact/n6.emitter")
      }
    },
    {
      "at": 0,
      "life": 0.16670000553131104,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Attack_Impact/n7.effect"),
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              1,
              2.700000047683716
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                0.5356000065803528,
                0.5356000065803528,
                0
              ],
              [
                1,
                0.5356000065803528,
                0.5356000065803528,
                0.3765000104904175
              ],
              [
                1,
                0.5356000065803528,
                0.5356000065803528,
                0
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.2777999937534332,
            "length": 360,
            "segments": 64,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "emitter",
        "seed": 0,
        "asset": ref("base/vfx/effekseer-starter/Attack_Impact/n8.emitter")
      }
    },
    {
      "at": 0,
      "life": 0.16670000553131104,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "mesh",
        "blend": "additive",
        "color": [
          1,
          1,
          1,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Attack_Impact/n9.effect"),
        "faceCamera": true,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              0
            ],
            [
              1,
              1.5
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                0.5356000065803528,
                0.5356000065803528,
                0
              ],
              [
                1,
                0.5356000065803528,
                0.5356000065803528,
                0.5019999742507935
              ],
              [
                1,
                0.5356000065803528,
                0.5356000065803528,
                1
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0,
            "length": 360,
            "segments": 64,
            "start": 0
          }
        },
        "spin": [
          0,
          0,
          0
        ]
      }
    },
    {
      "at": 0,
      "life": 0.30000001192092896,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "light",
        "color": [
          1,
          0.6909000277519226,
          0.6909000277519226
        ],
        "distance": 12,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              8
            ],
            [
              1,
              0
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        0
      ],
      "velocity": [
        0,
        0,
        0
      ],
      "acceleration": [
        0,
        0,
        0
      ],
      "node": {
        "kind": "shake",
        "trauma": 0.4000000059604645
      }
    }
  ]
});
