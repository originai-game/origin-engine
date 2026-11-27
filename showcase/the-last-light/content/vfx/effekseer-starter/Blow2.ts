import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 0.53329998254776,
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
      "at": 0,
      "life": 0.53329998254776,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        63.434898376464844,
        37.761199951171875,
        -50.76850128173828
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
          3,
          3,
          3,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Blow2/n0.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              2.5
            ],
            [
              0.14059999585151672,
              5.474800109863281
            ],
            [
              0.23440000414848328,
              7.253600120544434
            ],
            [
              0.3280999958515167,
              8.67389965057373
            ],
            [
              0.4219000041484833,
              9.689399719238281
            ],
            [
              0.5156000256538391,
              10.325400352478027
            ],
            [
              0.6093999743461609,
              10.65310001373291
            ],
            [
              1,
              11
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.75,
            "colors": [
              [
                1,
                0,
                0,
                0
              ],
              [
                1,
                0.03669999912381172,
                0.03669999912381172,
                0.304500013589859
              ],
              [
                1,
                0,
                0,
                0
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.9746999740600586,
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
      "life": 0.5,
      "offset": [
        0.5,
        0,
        -0.8659999966621399
      ],
      "rotate": [
        63.434898376464844,
        37.761199951171875,
        -50.76850128173828
      ],
      "velocity": [
        4,
        0,
        -6.928199768066406
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
          0.04699999839067459,
          0.04699999839067459,
          0.04699999839067459,
          0.7843000292778015
        ],
        "effect": ref("base/vfx/effekseer-starter/Blow2/n1-0.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              4
            ],
            [
              0.07810000330209732,
              3.383500099182129
            ],
            [
              0.15620000660419464,
              2.865799903869629
            ],
            [
              0.23440000414848328,
              2.433500051498413
            ],
            [
              0.3280999958515167,
              2.009399890899658
            ],
            [
              0.46880000829696655,
              1.5194000005722046
            ],
            [
              0.8281000256538391,
              0.5755000114440918
            ],
            [
              1,
              0
            ]
          ]
        },
        "shape": {
          "kind": "quad"
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
        1,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        -90
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
        "asset": ref("base/vfx/effekseer-starter/Blow2/n2-0.emitter")
      }
    },
    {
      "at": 0,
      "life": 0.53329998254776,
      "offset": [
        0,
        0,
        0
      ],
      "rotate": [
        63.434898376464844,
        37.761199951171875,
        -50.76850128173828
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
          2,
          2,
          2,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/Blow2/n3.effect"),
        "faceCamera": false,
        "scale": {
          "kind": "curve",
          "keys": [
            [
              0,
              2.5
            ],
            [
              0.14059999585151672,
              5.474800109863281
            ],
            [
              0.23440000414848328,
              7.253600120544434
            ],
            [
              0.3280999958515167,
              8.67389965057373
            ],
            [
              0.4219000041484833,
              9.689399719238281
            ],
            [
              0.5156000256538391,
              10.325400352478027
            ],
            [
              0.6093999743461609,
              10.65310001373291
            ],
            [
              1,
              11
            ]
          ]
        },
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.75,
            "colors": [
              [
                1,
                0,
                0,
                0
              ],
              [
                1,
                0.03669999912381172,
                0.03669999912381172,
                0.304500013589859
              ],
              [
                1,
                0,
                0,
                0
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.7875000238418579,
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
        2,
        0,
        0
      ],
      "rotate": [
        0,
        0,
        -90
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
        "asset": ref("base/vfx/effekseer-starter/Blow2/n4-0-0.emitter")
      }
    },
    {
      "at": 0,
      "life": 0.30000001192092896,
      "offset": [
        0.6241000294685364,
        0,
        -0.0005000000237487257
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
          0.6657000184059143,
          0.6607999801635742
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
