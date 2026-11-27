import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 66.66670227050781,
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
      "life": 66.66670227050781,
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
          10,
          10,
          10,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/smallTexturesSquareWave_orbital/n0-0-0-0.effect"),
        "faceCamera": false,
        "scale": 2,
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                0,
                1,
                0.03920000046491623
              ],
              [
                1,
                0,
                1,
                0.03920000046491623
              ],
              [
                1,
                0,
                0,
                0.03920000046491623
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.800000011920929,
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
      "life": 66.66670227050781,
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
          10,
          10,
          10,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/smallTexturesSquareWave_orbital/n0-0-0-1.effect"),
        "faceCamera": false,
        "scale": 1.399999976158142,
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                0.21950000524520874,
                0.04780000075697899,
                0.25099998712539673
              ],
              [
                1,
                0.21950000524520874,
                0.04780000075697899,
                0.25099998712539673
              ],
              [
                0.9657999873161316,
                0.6187000274658203,
                0.2508000135421753,
                0.25099998712539673
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.7142999768257141,
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
      "life": 66.66670227050781,
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
          10,
          10,
          10,
          1
        ],
        "effect": ref("base/vfx/effekseer-starter/smallTexturesSquareWave_orbital/n0-0-0-2.effect"),
        "faceCamera": false,
        "scale": 0.800000011920929,
        "shape": {
          "kind": "ring",
          "ring": {
            "band": 0.5,
            "colors": [
              [
                1,
                0,
                0,
                1
              ],
              [
                1,
                0.21950000524520874,
                0.04780000075697899,
                1
              ],
              [
                0.9657999873161316,
                0.6187000274658203,
                0.2508000135421753,
                1
              ]
            ],
            "height": [
              0,
              0
            ],
            "inner": 0.125,
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
    }
  ]
});
