// The template's playtest, run by `og test` through `og run --headless
// --commands`: a tap on the menu's Start button (found by its UI id) drops
// the crate, spins the dropship up as it lifts off and hands the trickster
// to seat 0; W then walks it for a second, in whichever view this scaffold
// draws (src/creation.ts), and the frame is checked for that view's
// framing: third person, the camera behind and above the trickster looking
// at it; first person, the camera at its eye looking along its heading,
// its own body not drawn; isometric, an orthographic camera at the look's
// ArtCamera angle (35.264° down, turned 45°) with the trickster in the
// middle, walked towards the top of the screen. The frame is compared with
// references/<tier>/smoke-<view>.png (tests/frames.ts; `OG_FX_CAPTURE=1 og
// test .` records it), and the description, the dumps and the UI tree are
// written beside it.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { angle, component, FIRST_PERSON, forward, ISO, ISOMETRIC, LOCAL, position, tap, value, VIEW } from "./lib.ts";
import type { V3 } from "./lib.ts";

const FRAME = `smoke-${VIEW}.png`;

interface Camera {
  position: V3;
  quaternion: number[];
  orthographic: boolean;
  height: number;
  layers: number;
}

export default playtest({
  name: `A tap on Start hands the trickster over and W walks it, framed for ${VIEW}`,
  size: "1280x720",
  commands: [
    tap(0.5, "Start"),
    // The tap was one press; the run started; the HUD replaced the menu.
    "0.8:expect:event:og.PointerDown:1",
    "0.8:expect:state:started==true",
    "0.9:expect:ui:text:=Cells 0/5",
    "0.95:dump_entity:trickster:before.json",
    "1.0:key_down:KeyW",
    "1.6:dump_entity:trickster:walking.json",
    "2.0:key_up:KeyW",
    // The dropship spins as it lifts; the crate fell from 4 m onto the ground.
    "2.0:expect:state:spin>0",
    "2.0:expect:event:og.CollisionStarted",
    // The trickster on screen, about the middle (first person draws none of it: its own eye).
    ...(LOCAL || FIRST_PERSON ? [] : ["2.4:expect:entity:trickster:within:340,120,600,560"]),
    "2.4:screenshot:" + FRAME,
    "2.4:get:og.Camera:camera.json",
    "2.4:dump_entity:trickster:after.json",
    ...(LOCAL ? ["2.4:dump_entity:view-0:view.json"] : []),
    "2.4:get:dropship.og.Transform:ship.json",
    "2.45:dump_entity:crate:crate.json",
    "2.5:describe:describe.json",
    "2.5:dump_world:world.json",
    "2.55:dump_ui:ui.json",
    "2.6:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const before = position(run, "before.json");
    const after = position(run, "after.json");
    const walked: V3 = [after[0] - before[0], 0, after[2] - before[2]];
    const metres = Math.hypot(walked[0], walked[2]);
    t.ok(metres > 2 && metres < 3.2, `W walked the trickster ${metres.toFixed(2)} m in a second`);
    t.ok(Math.abs(after[1]) < 0.1, `on the ground (y ${after[1].toFixed(3)})`);
    // W walks forward: along the view in first and third person (-Z, the heading at Start), up the screen isometric.
    const yaw = ((ISOMETRIC ? ISO.yaw : 0) * Math.PI) / 180;
    const ahead: V3 = [-Math.sin(yaw), 0, -Math.cos(yaw)];
    t.ok(angle(walked, ahead) < 5, `forward is ${ISOMETRIC ? "up the screen" : "along the view"} (${angle(walked, ahead).toFixed(1)}° off)`);
    t.ok(component<{ clip: number }>(run, "walking.json", "og.AnimationPlayer").clip !== component<{ clip: number }>(run, "before.json", "og.AnimationPlayer").clip, "the walk clip replaced the idle one");
    t.ok(value<{ position: V3 }>(run, "ship.json").position[1] > 3, "the dropship lifted off the pad");
    t.ok(position(run, "crate.json")[1] < 1, "the crate landed");

    // The framing: the main camera alone and online, seat 0's half with two on one device.
    const cam = value<Camera>(run, "camera.json");
    const eye: V3 = LOCAL ? position(run, "view.json") : cam.position;
    const look = forward(LOCAL ? component<{ quaternion: number[] }>(run, "view.json", "og.Transform").quaternion : cam.quaternion);
    const head: V3 = [after[0], after[1] + 1.6, after[2]];
    if (FIRST_PERSON) {
      t.ok(Math.hypot(eye[0] - head[0], eye[1] - head[1], eye[2] - head[2]) < 0.05, `first person: the camera is at the trickster's eye (${eye.map((n) => n.toFixed(2)).join(", ")})`);
      t.ok(angle(look, ahead) < 3, `looking along its heading (${angle(look, ahead).toFixed(1)}° off)`);
      const layers = component<{ layers: number }>(run, "after.json", "og.Model").layers;
      const draws = LOCAL ? component<{ layers: number }>(run, "view.json", "og.CameraView").layers : cam.layers;
      t.ok((layers & draws) === 0, `its own body is not drawn by its eye (body layers ${layers}, camera ${draws})`);
    } else if (ISOMETRIC) {
      const pitch = (Math.asin(-look[1]) * 180) / Math.PI;
      const heading = (Math.atan2(-look[0], -look[2]) * 180) / Math.PI;
      t.ok(Math.abs(pitch - ISO.pitch) < 0.5, `isometric: the camera looks ${pitch.toFixed(2)}° down (${ISO.pitch})`);
      t.ok(Math.abs(heading - ISO.yaw) < 0.5, `turned ${heading.toFixed(2)}° (${ISO.yaw})`);
      t.ok(LOCAL ? component<{ orthographic: boolean }>(run, "view.json", "og.CameraView").orthographic : cam.orthographic, "on an orthographic projection");
      const to: V3 = [after[0] - eye[0], after[1] - eye[1], after[2] - eye[2]];
      t.ok(angle(to, look) < 1.5, `the trickster is in the middle of the view (${angle(to, look).toFixed(2)}° off)`);
    } else {
      const back = Math.hypot(eye[0] - after[0], eye[2] - after[2]);
      t.ok(back > 3 && back < 6.5, `third person: the camera is ${back.toFixed(2)} m behind the trickster`);
      t.ok(eye[1] > head[1], `and above its head (${eye[1].toFixed(2)} m)`);
      t.ok(eye[2] > after[2], "behind it, on the side it walked away from");
      const to: V3 = [head[0] - eye[0], head[1] - eye[1], head[2] - eye[2]];
      t.ok(angle(to, look) < 6, `looking at it (${angle(to, look).toFixed(1)}° off)`);
    }
    t.frame([FRAME], () => t.equal(run.png(FRAME).width, 1280, `${FRAME} is 1280 wide`));
    checkFrames(run, t, [FRAME], "");
  },
});
