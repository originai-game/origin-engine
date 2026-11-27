import {file, ref} from '@origingame/engine/author';

export default file.import({
  "kind": "model",
  "source": "assets/models/strider.glb",
  "animation": {
    "rig": ref("base/rigs/strider"),
    "libraries": [
      ref("base/anims/humanoid-clips")
    ]
  }
});
