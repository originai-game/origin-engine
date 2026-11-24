// Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.
//! The IDL's module constants, generated from the IDL; `og_api::consts`
//! is them. The TypeScript side's is packages/engine/src/generated/consts.ts.

/// The most pointers og.Pointers tracks at once: ids 0 (the primary pointer) to 4.
pub const MAX_POINTERS: u32 = 5;

/// The most gamepads og.Gamepad reads: pads 0 to 7.
pub const MAX_GAMEPADS: u32 = 8;

/// Maximum distinct primitive CoC sources in an interrupted camera blend; identical sources coalesce before overflow selection.
pub const MAX_CAMERA_COC_SOURCES: u32 = 4;

/// ParticleSystem.startTick's default: never written. A looping emitter emits as if started at tick 0; a one-shot (`looping false`) emits nothing until its startTick is written, so a burst spawned in `start` waits for its first use. Write it back to re-arm a one-shot without firing it.
pub const PARTICLES_ARMED: u32 = 4294967295;

/// The most point and spot lights one frame draws, among those in view; past it the ones brightest at the camera are kept.
pub const MAX_LIGHTS: u32 = 256;

/// The most point and spot lights that shade one screen cluster (a 1/16 by 1/9 tile of the view and one of 24 depth slices); past it the ones brightest at the camera are kept there.
pub const MAX_CLUSTER_LIGHTS: u32 = 64;

/// The UI's layer among screen sprites and screen text (SpriteDesc and TextDesc with screen): those at or below it draw under every og.ui element, those above it over every one, modals, toasts and the screen transition included, in their layers' order either way.
pub const UI_LAYER: i32 = 1000;

/// The most og.ReflectionProbe components one frame samples, nearest the camera first; the rest are skipped.
pub const MAX_REFLECTION_PROBES: u32 = 8;

/// The most hits og.physics.castRayAll returns.
pub const MAX_RAYCAST_HITS: u32 = 256;

/// RigidBody.lock bit: no translation along x.
pub const LOCK_TRANSLATION_X: u32 = 1;

/// RigidBody.lock bit: no translation along y.
pub const LOCK_TRANSLATION_Y: u32 = 2;

/// RigidBody.lock bit: no translation along z.
pub const LOCK_TRANSLATION_Z: u32 = 4;

/// RigidBody.lock bit: no rotation about x.
pub const LOCK_ROTATION_X: u32 = 8;

/// RigidBody.lock bit: no rotation about y.
pub const LOCK_ROTATION_Y: u32 = 16;

/// RigidBody.lock bit: no rotation about z (the 2D profile's only rotation).
pub const LOCK_ROTATION_Z: u32 = 32;

/// Physics.profile: rapier2d, bodies in the xy plane rotating about z.
pub const PROFILE_2D: u32 = 2;

/// Physics.profile: rapier3d.
pub const PROFILE_3D: u32 = 3;

/// `CharacterEvent.kind`: a jump from the ground (or in coyote time).
pub const CHARACTER_JUMPED: u32 = 1;

/// `CharacterEvent.kind`: touched the ground after being in the air.
pub const CHARACTER_LANDED: u32 = 2;

/// `CharacterEvent.kind`: a jump off a wall.
pub const CHARACTER_WALL_JUMPED: u32 = 3;

/// `CharacterEvent.kind`: a jump in the air (PlatformerController.airJumps).
pub const CHARACTER_AIR_JUMPED: u32 = 4;

/// `CharacterEvent.kind`: left the ground without jumping (ran off an edge).
pub const CHARACTER_LEFT_GROUND: u32 = 5;

/// `CharacterEvent.kind`: hit a ceiling while rising.
pub const CHARACTER_BUMPED: u32 = 6;

/// `CharacterEvent.kind`: opposing solid surfaces crushed the character.
pub const CHARACTER_CRUSHED: u32 = 7;

/// `Cloth.pins` bit: the whole top row is pinned.
pub const PIN_TOP: u32 = 1;

/// `Cloth.pins` bit: the top-left and top-right corners are pinned.
pub const PIN_TOP_CORNERS: u32 = 2;

/// `Cloth.pins` bit: the whole left column is pinned (a flag on a pole).
pub const PIN_LEFT: u32 = 4;

/// `Cloth.pins` bit: every fourth point of the top row (curtain rings).
pub const PIN_TOP_RINGS: u32 = 8;

/// QueryFilter.kindFlags bit: include dynamic bodies.
pub const QUERY_DYNAMIC: u32 = 1;

/// QueryFilter.kindFlags bit: include fixed bodies and static colliders.
pub const QUERY_FIXED: u32 = 2;

/// QueryFilter.kindFlags bit: include kinematic bodies and controllers.
pub const QUERY_KINEMATIC: u32 = 4;

/// Weather.kind: clear.
pub const WEATHER_NONE: u32 = 0;

/// Weather.kind: rain streaks falling with the wind, the light dimmed and a grey distance fog.
pub const WEATHER_RAIN: u32 = 1;

/// Weather.kind: flakes drifting down on gusts, a pale fog.
pub const WEATHER_SNOW: u32 = 2;

/// Weather.kind: dust blowing along the wind near the ground, a warm thick fog.
pub const WEATHER_DUST: u32 = 3;

/// Weather.kind: no particles, a dense low height fog.
pub const WEATHER_MIST: u32 = 4;

/// og.Season.start: the northern spring equinox, about 20 March (the autumn equinox south of the equator).
pub const SEASON_SPRING: u32 = 79;

/// og.Season.start: the northern summer solstice, about 21 June (the winter solstice south of the equator).
pub const SEASON_SUMMER: u32 = 172;

/// og.Season.start: the northern autumn equinox, about 23 September (the spring equinox south of the equator).
pub const SEASON_AUTUMN: u32 = 265;

/// og.Season.start: the northern winter solstice, about 21 December (the summer solstice south of the equator).
pub const SEASON_WINTER: u32 = 355;

/// SoundEnded.reason: a one-shot played to its end.
pub const SOUND_FINISHED: u32 = 0;

/// SoundEnded.reason: `playing` was set to false.
pub const SOUND_STOPPED: u32 = 1;

/// SoundEnded.reason: the AudioSource was removed from a living entity.
pub const SOUND_REMOVED: u32 = 2;

/// SoundEnded.reason: `sound` changed while it played.
pub const SOUND_REPLACED: u32 = 3;

/// SoundEnded.reason: more sources played than Audio.maxVoices and this one stood lowest (priority, then volume, then age).
pub const SOUND_CULLED: u32 = 4;

/// SoundEnded.reason: AudioSource.retrigger started the sound again from the top; a new voice plays it.
pub const SOUND_RETRIGGERED: u32 = 5;

/// NavAgent.status: not seeking.
pub const NAV_IDLE: u32 = 0;

/// NavAgent.status: following a path or a flow field.
pub const NAV_MOVING: u32 = 1;

/// NavAgent.status: within stopDistance of the target.
pub const NAV_ARRIVED: u32 = 2;

/// NavAgent.status: no path reaches the target, or the agent is off its grid or navmesh.
pub const NAV_UNREACHABLE: u32 = 3;

/// NavAgent.status: the target is cut off; moving to the reachable point nearest it.
pub const NAV_PARTIAL: u32 = 4;

/// NavAgent.status: crossing an off-mesh link (a jump, a ladder, a door, a teleport).
pub const NAV_ON_LINK: u32 = 5;

/// AiAgent.status: the tree or state is still running.
pub const AI_RUNNING: u32 = 1;

/// AiAgent.status: the tree succeeded on the last think.
pub const AI_SUCCESS: u32 = 2;

/// AiAgent.status: the tree failed on the last think.
pub const AI_FAILURE: u32 = 3;

/// AiActionStopped.result: the action stopped before the game finished it (another branch won, the state changed, the agent restarted).
pub const AI_INTERRUPTED: u32 = 4;

/// The core methods that are not service methods: the store and its save slots, the leaderboards, telemetry and the catalog's head — what any host answers from its own storage. The provider's CORE_METHODS is these plus SERVICE_METHODS; capabilities is these, the services and the extensions.
pub const CORE_ONLY_METHODS: &str = "store.get store.set store.list leaderboard.submit leaderboard.top leaderboard.around telemetry catalog.head";

/// The service methods: purchases, ads, achievements, the player, friends, invites and share (docs/platform-asks.md #19). They need the platform's services, so a host without one answers it `unavailable`.
pub const SERVICE_METHODS: &str = "purchase.products purchase.buy purchase.entitlements purchase.consume ad.ready ad.show achievement.unlock achievement.progress achievement.list player.get friends.list invite.send invite.list share";

/// Methods one host defines: `point` (Origin Point) and `ai.mark` are OriginGame's, not core methods (docs/platform-asks.md).
pub const HOST_EXTENSIONS: &str = "point ai.mark";

/// Every error code an answer may carry (PlatformError.code); a call may be retried later for RETRYABLE_CODES.
pub const ERROR_CODES: &str = "offline timeout invalid unauthorized not_found conflict too_large rate_limited unavailable failed cancelled no_fill";

/// The error codes whose call may succeed later (`retryable: true`).
pub const RETRYABLE_CODES: &str = "offline timeout unavailable rate_limited no_fill";

/// The global the page publishes the probe on.
pub const PROBE_GLOBAL: &str = "__runtime_probe__";

/// The RuntimeProbe contract revision both sides speak.
pub const PROBE_VERSION: u32 = 5;

/// The wire revision hello reports.
pub const PROBE_WIRE_VERSION: u32 = 1;

/// The page query parameter that starts a deterministic replay from a uint32 seed.
pub const REPLAY_SEED_PARAM: &str = "replay-seed";

/// The longest playtest one request may advance, in fixed steps.
pub const PLAYTEST_MAX_TICKS: u32 = 3600;

/// The most operations one probe program may carry.
pub const PROBE_PROGRAM_MAX_OPERATIONS: u32 = 256;

/// The long side capture, marks and key frames fit an image to unless asked otherwise: a whole frame a model reads in few tokens; a detail is cut at full resolution as its own image (capture's region).
pub const CAPTURE_FIT: u32 = 1280;

/// `StreamEvent.state`: the group is unloaded (StreamState unloaded).
pub const STREAM_UNLOADED: u32 = 0;

/// `StreamEvent.state`: the group waits for room in the budget (StreamState queued).
pub const STREAM_QUEUED: u32 = 1;

/// `StreamEvent.state`: the group's pack is being read (StreamState loading).
pub const STREAM_LOADING: u32 = 2;

/// `StreamEvent.state`: the group's pack is available (StreamState resident), independently of texture worker progress.
pub const STREAM_RESIDENT: u32 = 3;

/// `StreamEvent.state`: the group failed (StreamState failed; og.stream.status says why).
pub const STREAM_FAILED: u32 = 4;

/// Fractured.cause: a collision's impulse (og.CollisionStarted) past the prop's impactThreshold.
pub const CAUSE_IMPACT: u32 = 0;

/// Fractured.cause: og.destruction.damage.
pub const CAUSE_DAMAGE: u32 = 1;

/// Fractured.cause: og.destruction.explode; PixelDetached.cause: og.destruction.explode2d.
pub const CAUSE_EXPLOSION: u32 = 2;

/// Fractured.cause: the stress solver: a part that could no longer hold its own weight.
pub const CAUSE_STRESS: u32 = 3;

/// Fractured.cause: a chunk with children (the fracture block's `children`) took three times the damage that breaks a bond of its cross-section, and broke into them: they fly free, each its own body, leaving a hole.
pub const CAUSE_SHATTER: u32 = 4;

/// og.destruction.material: brickwork in mortar (no-tension masonry: stands by its weight, steps back over its piers).
pub const MATERIAL_BRICK: u32 = 0;

/// og.destruction.material: plain concrete (strong, breaks in chunky lumps, holds what is left).
pub const MATERIAL_CONCRETE: u32 = 1;

/// og.destruction.material: timber boards (light and weak across the grain: splinters where it lands or is struck).
pub const MATERIAL_WOOD: u32 = 2;

/// og.destruction.material: a glass pane (shatters at a touch, no stress solver).
pub const MATERIAL_GLASS: u32 = 3;

/// og.destruction.material: dry-laid stone (masonry with no mortar: no tension, heavier and tougher than brick).
pub const MATERIAL_STONE: u32 = 4;

/// Fractured.cause: og.destruction.fracture; PixelDetached.cause: og.destruction.fracture2d.
pub const CAUSE_FRACTURE: u32 = 5;

/// PixelDetached.cause: og.destruction.carve2d cut it loose.
pub const CAUSE_CARVE: u32 = 6;
