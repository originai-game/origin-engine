import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 14,
  "looping": false,
  "events": [
    {
      "name": "impact",
      "time": 0
    }
  ],
  "params": [
    {
      "default": 1,
      "name": "mark"
    }
  ],
  "elements": [
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        1.4500000476837158,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/fireball")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        1.600000023841858,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/smoke-column")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.10000000149011612,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/ground-dust")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.20000000298023224,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/debris")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        1.600000023841858,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/heat")
      }
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.4000000059604645,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/sparks")
      }
    },
    {
      "at": 0,
      "life": 0.18000000715255737,
      "offset": [
        0,
        1,
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
        "kind": "glow",
        "color": [
          2.200000047683716,
          1.100000023841858,
          0.4000000059604645,
          0.699999988079071
        ],
        "size": {
          "kind": "curve",
          "keys": [
            [
              0,
              1.5
            ],
            [
              0.30000001192092896,
              3.200000047683716
            ],
            [
              1,
              2
            ]
          ]
        }
      }
    },
    {
      "at": 0,
      "life": 0.6000000238418579,
      "offset": [
        0,
        1.2000000476837158,
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
          0.5,
          0.18000000715255737
        ],
        "distance": 7,
        "intensity": {
          "kind": "curve",
          "keys": [
            [
              0,
              8
            ],
            [
              0.15000000596046448,
              5
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
    },
    {
      "at": 0.03999999910593033,
      "life": 13.960000038146973,
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
        "kind": "decal",
        "angleFade": 80,
        "color": [
          1,
          1,
          1,
          1
        ],
        "emissive": [
          0,
          0,
          0
        ],
        "fade": 5,
        "normal": ref("base/vfx/cinematic-realism/textures/scorch-normal"),
        "normalScale": 1,
        "roughness": 0.9599999785423279,
        "size": [
          8,
          1.399999976158142,
          8
        ],
        "texture": ref("base/vfx/cinematic-realism/textures/scorch"),
        "turn": [
          0,
          360
        ],
        "uv": [
          0,
          0,
          1,
          1
        ],
        "wet": false
      },
      "when": "mark"
    },
    {
      "at": 0.11999999731779099,
      "life": 6,
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
        "kind": "decal",
        "angleFade": 80,
        "color": [
          0.05999999865889549,
          0.05000000074505806,
          0.04500000178813934,
          1
        ],
        "emissive": [
          3,
          1.7000000476837158,
          0.8999999761581421
        ],
        "fade": 5.400000095367432,
        "normalScale": 1,
        "size": [
          3.5999999046325684,
          1.399999976158142,
          3.5999999046325684
        ],
        "texture": ref("base/vfx/cinematic-realism/textures/embers"),
        "turn": [
          0,
          360
        ],
        "uv": [
          0,
          0,
          1,
          1
        ],
        "wet": false
      },
      "when": "mark"
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
        "kind": "sound",
        "pitch": [
          0.949999988079071,
          1.0499999523162842
        ],
        "volume": 1,
        "asset": ref("base/vfx/cinematic-realism/audio/blast")
      }
    }
  ]
});
