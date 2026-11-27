import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.0 "Node": spawning on a model's surface
// effekseer: node 0.0.0 "Node": ping-pong UV animation: looped
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-hato01/iwai-ume/n0-0.parent"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        -5.1,
        0.0
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-hato01/iwai-ume/n0-0-0.parent"
        }
      },
      "at": 0.0,
      "offset": [
        0.0,
        -5.1,
        0.0
      ]
    }
  ],
  "duration": 20.3,
  "events": [
    {
      "name": "impact",
      "time": 0.0
    }
  ]
});
