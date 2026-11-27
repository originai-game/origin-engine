import { file, ref } from '@origingame/engine/author';

export default file.vfx({
  "duration": 2,
  "looping": true,
  "events": [],
  "params": [
    {
      "default": 1,
      "name": "thrust"
    }
  ],
  "elements": [
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
        "asset": ref("base/vfx/cinematic-realism/emitters/burner-core")
      },
      "amount": "thrust"
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
        "asset": ref("base/vfx/cinematic-realism/emitters/burner-wake")
      },
      "amount": "thrust"
    },
    {
      "at": 0,
      "life": 0,
      "offset": [
        0,
        0.6000000238418579,
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
        "asset": ref("base/vfx/cinematic-realism/emitters/heat-jet")
      }
    },
    {
      "at": 0,
      "life": 2,
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
        "kind": "glow",
        "color": [
          0.6000000238418579,
          0.8999999761581421,
          1.7999999523162842,
          0.4000000059604645
        ],
        "size": 0.30000001192092896,
        "stretch": "thrust"
      }
    },
    {
      "at": 0,
      "life": 2,
      "offset": [
        0,
        0.699999988079071,
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
          1.5,
          0.949999988079071,
          0.4000000059604645,
          0.4000000059604645
        ],
        "size": 0.2199999988079071
      },
      "amount": "thrust"
    },
    {
      "at": 0,
      "life": 2,
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
        "kind": "glow",
        "color": [
          1.399999976158142,
          0.699999988079071,
          0.2199999988079071,
          0.3499999940395355
        ],
        "size": 0.18000000715255737
      },
      "amount": "thrust"
    },
    {
      "at": 0,
      "life": 2,
      "offset": [
        0,
        1.7000000476837158,
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
          1.100000023841858,
          0.44999998807907104,
          0.12999999523162842,
          0.30000001192092896
        ],
        "size": 0.12999999523162842
      },
      "amount": "thrust"
    },
    {
      "at": 0,
      "life": 2,
      "offset": [
        0,
        0.6000000238418579,
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
          0.6499999761581421,
          0.3499999940395355
        ],
        "distance": 6,
        "intensity": 2
      },
      "amount": "thrust"
    }
  ]
});
