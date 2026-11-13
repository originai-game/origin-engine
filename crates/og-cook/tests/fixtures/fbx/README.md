# FBX and OBJ fixtures

Files for `tests/fbx.rs` and `src/cook/formats/fbx/tests.rs`. Every file here is
made for this repository from nothing but `make.py` and Blender, or
written by hand: no third-party art. They are released under CC0 1.0
(public domain dedication).

| File | Made by | Exercises |
|---|---|---|
| `board.fbx` | by hand (ASCII FBX 7.4) | Z up in centimetres, a quad without normals, a Phong material |
| `props.fbx`, `props-linked.fbx`, `crate.png` | `make.py props` | a mesh shared by two nodes, a parented emissive lamp, a point light; the texture embedded (`props.fbx`) or named beside the file (`props-linked.fbx`) |
| `arm.fbx`, `arm.glb` | `make.py arm` | a skinned tube over three bones, a shape key driven by the elbow, two actions (NLA strips); the glTF of the same scene for comparison |

To make them again (Blender 4.5.15 LTS as the `bpy` module):

```sh
uv venv -p 3.11 /tmp/bpyenv && VIRTUAL_ENV=/tmp/bpyenv uv pip install bpy==4.5.15
/tmp/bpyenv/bin/python make.py            # or: make.py -- props | arm
```

Blender writes the creation time into FBX files, so the bytes differ run
to run; the geometry, skins, clips and materials do not.
