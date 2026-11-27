import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "move_curve": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0 "move_curve": a track whose points move on their own: drawn along its generator's path
// effekseer: node 0.0 "move_curve": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0 "move_curve": a ribbon or track's sideways motion: only its rise and turn are kept
// effekseer: node 0.0.0 "random": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 0.0 "move_curve": location over an f-curve: straightened
// effekseer: node 0.0.1 "random_stop": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 1.0 "move_curve": a moving ancestor: its descendants are placed where it starts
// effekseer: node 1.0 "move_curve": a track whose points move on their own: drawn along its generator's path
// effekseer: node 1.0 "move_curve": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 1.0 "move_curve": a ribbon or track's sideways motion: only its rise and turn are kept
// effekseer: node 1.0.0 "random": nested generation: 5 instances above its generator folded into one emitter
// effekseer: node 1.0 "move_curve": location over an f-curve: straightened
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-suzuki01/snowstorm11/n0-0.effect"
        },
        "width": [
          0.0,
          0.55
        ],
        "lifetime": 1.6667,
        "color": [
          1.0,
          1.0,
          1.0,
          0.0
        ],
        "rise": 11.5581
      },
      "at": 1.0083,
      "offset": [
        -39.6217,
        -18.9738,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -15.0
      ],
      "life": 1.65
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/snowstorm11/n0-0-0.parent"
        }
      },
      "at": 1.0083,
      "offset": [
        -39.6217,
        -18.9738,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -63.969
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/snowstorm11/n0-0-1.parent"
        }
      },
      "at": 1.0083,
      "offset": [
        -39.6217,
        -18.9738,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -63.969
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-suzuki01/snowstorm11/n1-0.effect"
        },
        "width": [
          0.0,
          0.85
        ],
        "lifetime": 2.0,
        "color": [
          1.0,
          1.0,
          1.0,
          0.0
        ],
        "rise": 10.3047
      },
      "at": 1.3417,
      "offset": [
        -33.9018,
        -8.7864,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -20.0
      ],
      "life": 1.65
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/snowstorm11/n1-0-0.parent"
        }
      },
      "at": 1.3417,
      "offset": [
        -33.9018,
        -8.7864,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        -70.4706
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/snowstorm11/n2.emitter"
        }
      },
      "at": 0.0,
      "rotate": [
        92.2137,
        -43.2684,
        -41.7721
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-suzuki01/snowstorm11/n3-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        -100.0,
        0.0,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        180.0
      ]
    }
  ],
  "duration": 8.9167,
  "events": [
    {
      "name": "impact",
      "time": 1.35
    }
  ]
});
