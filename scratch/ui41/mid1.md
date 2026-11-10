
### `State` (61 fields, persisted by `save_state`)

`State` is the interaction state the host snapshots. Most of it moves with **Interaction**. The fields that belong to other stages move to those states, but `save_state` and `restore_state` keep reading and writing them **in the same byte order**, so `STATE_VERSION` (23) and the snapshot bytes do not change.

Owners: Interaction 49 · Style 4 · Front 3 · Out 3 · Env 1 · Docs 1.

