"""Makes the FBX fixtures of og-cook's tests/fbx.rs with Blender's own
exporters, from nothing but this script (README.md says how to run it).

props.fbx / props-linked.fbx: a crate (a box, 2 x 1 x 0.5 m, with a
checker texture) holding a glowing lamp (an emissive cylinder) on its top,
a second crate sharing the first's mesh, and a point light; exported
Z up in centimetres. props.fbx embeds the texture, props-linked.fbx names
crate.png beside it.

arm.fbx / arm.glb: a skinned arm (a 12-ring tube, 2 m long, along +Z in
Blender) over three bones (root, upper, lower), with a shape key (bulge)
and two actions: wave (lower turns 90 degrees about X over a second, the
bulge, driven by the bend, rising to 1) and lift (upper turns 45 degrees over half a second).
The same scene exported as glTF, so a test compares the two cooks.
"""

import math
import os
import sys

import bpy

OUT = os.path.dirname(os.path.abspath(__file__))


def reset():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    scene.render.fps = 30
    return scene


def checker(path):
    """crate.png: 16 x 16, a 4 x 4 checker of brown and tan, red in the top-left cell."""
    img = bpy.data.images.new("crate", 16, 16, alpha=True)
    px = []
    for y in range(16):
        for x in range(16):
            # Blender's rows run bottom up: row 15 is the image's top.
            cell = ((x // 4) + ((15 - y) // 4)) % 2
            if x < 4 and y >= 12:
                px += [0.8, 0.05, 0.05, 1.0]
            elif cell:
                px += [0.45, 0.25, 0.1, 1.0]
            else:
                px += [0.8, 0.65, 0.4, 1.0]
    img.pixels = px
    img.filepath_raw = path
    img.file_format = "PNG"
    img.save()
    return img


def material(name, color=None, image=None, emission=None, strength=1.0, roughness=0.8):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    bsdf = m.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Roughness"].default_value = roughness
    if color:
        bsdf.inputs["Base Color"].default_value = (*color, 1.0)
    if image:
        tex = m.node_tree.nodes.new("ShaderNodeTexImage")
        tex.image = image
        m.node_tree.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    if emission:
        bsdf.inputs["Emission Color"].default_value = (*emission, 1.0)
        bsdf.inputs["Emission Strength"].default_value = strength
    return m


def props():
    scene = reset()
    scene.unit_settings.system = "METRIC"
    img = checker(os.path.join(OUT, "crate.png"))
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(0, 0, 0.25))
    crate = bpy.context.object
    crate.name = "crate"
    crate.scale = (2.0, 1.0, 0.5)
    bpy.ops.object.transform_apply(scale=True)
    crate.data.name = "crate-mesh"
    crate.data.materials.append(material("wood", image=img))
    bpy.ops.mesh.primitive_cylinder_add(vertices=8, radius=0.1, depth=0.3, location=(0.5, 0, 0.65))
    lamp = bpy.context.object
    lamp.name = "lamp"
    lamp.data.materials.append(material("glow", color=(1.0, 0.3, 0.1), emission=(1.0, 0.3, 0.1), strength=3.0))
    lamp.parent = crate
    lamp.matrix_parent_inverse = crate.matrix_world.inverted()
    twin = bpy.data.objects.new("crate-twin", crate.data)
    scene.collection.objects.link(twin)
    twin.location = (3.0, 0, 0.25)
    twin.rotation_euler = (0, 0, math.radians(90))
    light = bpy.data.lights.new("bulb", "POINT")
    light.color = (1.0, 0.9, 0.7)
    light.energy = 100.0
    lo = bpy.data.objects.new("bulb", light)
    scene.collection.objects.link(lo)
    lo.location = (0, -2, 2)
    common = dict(
        axis_up="Z",
        axis_forward="Y",
        global_scale=1.0,
        apply_unit_scale=False,
        apply_scale_options="FBX_SCALE_UNITS",
        object_types={"MESH", "LIGHT", "EMPTY"},
        bake_anim=False,
    )
    # Centimetres: the file's unit scale says 1 unit is 0.01 m.
    scene.unit_settings.scale_length = 0.01
    bpy.ops.export_scene.fbx(filepath=os.path.join(OUT, "props.fbx"), path_mode="COPY", embed_textures=True, **common)
    bpy.ops.export_scene.fbx(filepath=os.path.join(OUT, "props-linked.fbx"), path_mode="RELATIVE", **common)


def arm():
    scene = reset()
    scene.frame_start = 0
    scene.frame_end = 30
    arm_data = bpy.data.armatures.new("skeleton")
    rig = bpy.data.objects.new("rig", arm_data)
    scene.collection.objects.link(rig)
    bpy.context.view_layer.objects.active = rig
    bpy.ops.object.mode_set(mode="EDIT")
    bones = {}
    for name, head, tail, parent in [
        ("root", (0, 0, 0), (0, 0, 0.2), None),
        ("upper", (0, 0, 0.2), (0, 0, 1.1), "root"),
        ("lower", (0, 0, 1.1), (0, 0, 2.0), "upper"),
    ]:
        b = arm_data.edit_bones.new(name)
        b.head, b.tail = head, tail
        if parent:
            b.parent = bones[parent]
            b.use_connect = True
        bones[name] = b
    bpy.ops.object.mode_set(mode="OBJECT")
    # The tube: 12 rings of 8 vertices from z = 0.2 to 2.0, capped.
    rings, sides, r = 12, 8, 0.15
    verts, faces = [], []
    for i in range(rings):
        z = 0.2 + 1.8 * i / (rings - 1)
        for j in range(sides):
            a = 2 * math.pi * j / sides
            verts.append((r * math.cos(a), r * math.sin(a), z))
    for i in range(rings - 1):
        for j in range(sides):
            a, b = i * sides + j, i * sides + (j + 1) % sides
            faces.append((a, b, b + sides, a + sides))
    faces.append(tuple(reversed(range(sides))))
    faces.append(tuple((rings - 1) * sides + j for j in range(sides)))
    mesh = bpy.data.meshes.new("tube")
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    mesh.materials.append(material("skin", color=(0.2, 0.5, 0.8), roughness=0.5))
    tube = bpy.data.objects.new("tube", mesh)
    scene.collection.objects.link(tube)
    tube.parent = rig
    mod = tube.modifiers.new("skin", "ARMATURE")
    mod.object = rig
    # Weights: upper below the elbow, lower above it, blended over 0.3 m.
    up, low = tube.vertex_groups.new(name="upper"), tube.vertex_groups.new(name="lower")
    for v in mesh.vertices:
        t = min(max((v.co.z - 0.95) / 0.3, 0.0), 1.0)
        if t < 1.0:
            up.add([v.index], 1.0 - t, "REPLACE")
        if t > 0.0:
            low.add([v.index], t, "REPLACE")
    # The bulge: rings 4 to 7 pushed out by 0.1 m.
    tube.shape_key_add(name="Basis")
    bulge = tube.shape_key_add(name="bulge")
    for i, v in enumerate(mesh.vertices):
        ring = i // sides
        if 4 <= ring <= 7 and i < rings * sides:
            co = v.co
            d = math.hypot(co.x, co.y)
            bulge.data[i].co = (co.x * (d + 0.1) / d, co.y * (d + 0.1) / d, co.z)
    # Actions.
    def act(name, frames):
        a = bpy.data.actions.new(name)
        rig.animation_data_create()
        rig.animation_data.action = a
        for frame, bone, angle in frames:
            pb = rig.pose.bones[bone]
            pb.rotation_mode = "XYZ"
            pb.rotation_euler = (math.radians(angle), 0, 0)
            pb.keyframe_insert("rotation_euler", frame=frame)
        for fc in a.fcurves:
            for k in fc.keyframe_points:
                k.interpolation = "LINEAR"
        a.use_fake_user = True
        return a
    wave = act("wave", [(0, "lower", 0), (30, "lower", 90)])
    lift = act("lift", [(0, "upper", 0), (15, "upper", 45)])
    # The bulge rises with the elbow's bend (a driver: 0 straight, 1 at 90
    # degrees), so both exporters bake it into whichever clip bends it.
    fc = bulge.driver_add("value")
    fc.driver.type = "SCRIPTED"
    var = fc.driver.variables.new()
    var.type = "TRANSFORMS"
    var.name = "bend"
    var.targets[0].id = rig
    var.targets[0].bone_target = "lower"
    var.targets[0].transform_type = "ROT_X"
    var.targets[0].transform_space = "LOCAL_SPACE"
    fc.driver.expression = "bend / (pi / 2)"
    # NLA strips so the glTF exporter writes each action as its clip.
    for a in (wave, lift):
        tr = rig.animation_data.nla_tracks.new()
        tr.name = a.name
        tr.strips.new(a.name, 0, a)
    rig.animation_data.action = None
    # The rest pose is frame 0's, every key at its first value.
    scene.frame_set(0)
    bpy.ops.export_scene.fbx(
        filepath=os.path.join(OUT, "arm.fbx"),
        axis_up="Y",
        axis_forward="-Z",
        object_types={"ARMATURE", "MESH"},
        add_leaf_bones=False,
        bake_anim=True,
        bake_anim_use_all_actions=False,
        bake_anim_use_nla_strips=True,
        bake_anim_force_startend_keying=True,
        bake_anim_simplify_factor=0.0,
    )
    bpy.ops.export_scene.gltf(
        filepath=os.path.join(OUT, "arm.glb"),
        export_format="GLB",
        export_animation_mode="NLA_TRACKS",
        export_force_sampling=True,
        export_morph=True,
        export_skins=True,
    )


if __name__ == "__main__":
    which = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else ["props", "arm"]
    for w in which:
        globals()[w]()
