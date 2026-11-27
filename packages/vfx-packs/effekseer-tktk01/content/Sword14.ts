import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "ナイフ": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0 "ナイフ軌跡": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.0 "ナイフ軌跡": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.0.1 "*衝撃生業用": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.1 "*衝撃生業用": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.0.2.0 "衝撃": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.2.0 "衝撃": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.0.2.1 "衝撃線": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.0.2 "*衝撃生業用": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.0.2.1 "衝撃線": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 0.0.3 "衝撃グロー": a moving ancestor: its descendants are placed where it starts
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword14/n0-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        4.2096,
        4.8515,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        166.0
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword14/n0-0-0.effect"
        },
        "width": [
          2.0,
          0.5
        ],
        "lifetime": 0.1667,
        "color": [
          1.0,
          1.0,
          1.0,
          0.6667
        ]
      },
      "at": 0.0,
      "offset": [
        4.2096,
        4.8515,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        166.0
      ],
      "life": 0.1667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword14/n0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        4.4515,
        5.8218,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        166.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword14/n0-0-2-0.emitter"
        }
      },
      "at": 0.1333,
      "offset": [
        4.0886,
        4.3663,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        76.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword14/n0-0-2-1.parent"
        }
      },
      "at": 0.1333,
      "offset": [
        4.0886,
        4.3663,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        166.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-tktk01/Sword14/n0-0-3.emitter"
        }
      },
      "at": 0.1333,
      "offset": [
        4.2096,
        4.8515,
        0.0
      ],
      "rotate": [
        0.0,
        0.0,
        166.0
      ]
    }
  ],
  "duration": 2.3017,
  "events": [
    {
      "name": "impact",
      "time": 0.1333
    }
  ]
});
