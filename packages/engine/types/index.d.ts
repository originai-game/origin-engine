// Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.
// @origingame/engine API v1.

// The Origin Engine API. Game code declares components and systems, and reads the world through typed views over engine memory.
// Every failure is a returned Result; desc objects may omit any field; nothing reads the wall clock inside a tick.

declare const valueType: unique symbol;
declare const handleType: unique symbol;

/**
 * Every failure in the API is a value, never a throw: check `ok` before
 * reading `value`. `reason` is a plain sentence that names the fix.
 */
export type Result<T> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly reason: string };

/** A column kind: what one field of a component or resource stores. */
export type ColumnKind = keyof ColumnKinds;

/**
 * One field of a reflected layout, made by a column factory
 * (`f32(100, { min: 0 })`, `Vector3()`). `default` is the value a new
 * component or resource starts with.
 */
export interface FieldSpec<K extends ColumnKind = ColumnKind> {
  readonly kind: K;
  readonly default: ColumnKinds[K]["value"];
  readonly options: FieldOptions;
  /** A `str` field's capacity in bytes; absent for every other kind. */
  readonly capacity?: number;
}

/** A reflected layout: field names to field specs. */
export type Fields = { readonly [field: string]: FieldSpec };

/** The value of a layout: each field as its column's value type. */
export type FieldValues<F extends Fields> = {
  -readonly [K in keyof F]: ColumnKinds[F[K]["kind"]]["value"];
};

/** The typed views of a layout: each field as its column's typed array. */
export type ColumnViews<F extends Fields> = {
  readonly [K in keyof F]: ColumnKinds[F[K]["kind"]]["array"];
};

/** A registered component: the token systems, queries and the world take. */
export interface ComponentType<F extends Fields = Fields, V = FieldValues<F>> {
  readonly kind: "component";
  readonly name: string;
  readonly fields: F;
  readonly [valueType]: V;
}

/** A registered resource: a reflected singleton. */
export interface ResourceType<F extends Fields = Fields, V = FieldValues<F>> {
  readonly kind: "resource";
  readonly name: string;
  readonly fields: F;
  readonly [valueType]: V;
}

/** A registered event type. */
export interface EventType<V = unknown> {
  readonly kind: "event";
  readonly name: string;
  readonly [valueType]: V;
}

/** Any component token. */
export type AnyComponent = ComponentType<any, any>;
/** Any resource token. */
export type AnyResource = ResourceType<any, any>;
/** Any event token. */
export type AnyEvent = EventType<any>;
/** Any reflected token a system may declare: a component or a resource. */
export type AnyReflected = AnyComponent | AnyResource;

/** The value a token stands for: a component's fields, a resource object, an event payload. */
export type Value<T> = T extends { readonly [valueType]: infer V } ? V : never;

/** The typed column views of a component token, as `ArchetypeView.column` returns them. */
export type View<T> = T extends ComponentType<infer F, any> ? ColumnViews<F> : never;

/** Every column kind, from the IDL's `column` declarations. */
export interface ColumnKinds {
  /** A 32-bit float, the default numeric field. */
  f32: { readonly array: Float32Array; readonly stride: 1; readonly value: number };
  /** A 64-bit float, for accumulated values that must not drift, such as elapsed time. */
  f64: { readonly array: Float64Array; readonly stride: 1; readonly value: number };
  /** A 32-bit unsigned integer: counters, flags, indices. */
  u32: { readonly array: Uint32Array; readonly stride: 1; readonly value: number };
  /** A 32-bit signed integer. */
  i32: { readonly array: Int32Array; readonly stride: 1; readonly value: number };
  /** A boolean, stored as one byte: 0 or 1. */
  bool: { readonly array: Uint8Array; readonly stride: 1; readonly value: boolean };
  /** A Vector2 field: x, y interleaved. */
  Vector2: { readonly array: Float32Array; readonly stride: 2; readonly value: Vector2 };
  /** A Vector3 field: x, y, z interleaved. */
  Vector3: { readonly array: Float32Array; readonly stride: 3; readonly value: Vector3 };
  /** A Quaternion field: x, y, z, w interleaved. */
  Quaternion: { readonly array: Float32Array; readonly stride: 4; readonly value: Quaternion };
  /** A Matrix4 field: sixteen column-major elements per entity. */
  Matrix4: { readonly array: Float32Array; readonly stride: 16; readonly value: Matrix4 };
  /** A Color field: r, g, b interleaved. */
  Color: { readonly array: Float32Array; readonly stride: 3; readonly value: Color };
  /**
   * Text of at most `capacity` bytes of UTF-8, stored inline and zero-padded, so it is world state like a number: a player's name, a label. `str(48)` holds any 12 characters (four bytes covers every script). The capacity is 1 to 4,096 bytes: a larger one is refused when the game starts (`a string field holds 1 to 4096 bytes`), so longer text goes in chunks, an entity each.
   * Get and set it as a JavaScript string; text over the capacity is refused with the fix named (an error value from world.insert/world.set, a throw from a resource's setter), never truncated. Its view is the raw bytes.
   */
  str: { readonly array: Uint8Array; readonly stride: number; readonly value: string };
  /** An Entity field: (index, generation) pairs. */
  Entity: { readonly array: Uint32Array; readonly stride: 2; readonly value: Entity };
  /** A Geometry handle field. */
  Geometry: { readonly array: Uint32Array; readonly stride: 2; readonly value: Geometry };
  /** A Material handle field. */
  Material: { readonly array: Uint32Array; readonly stride: 2; readonly value: Material };
  /** A ToneMapping field: the variant's index. */
  ToneMapping: { readonly array: Uint32Array; readonly stride: 1; readonly value: ToneMapping };
  /** A Texture handle field (0 is none). */
  Texture: { readonly array: Uint32Array; readonly stride: 2; readonly value: Texture };
  /** A Lut handle field (0 is none). */
  Lut: { readonly array: Uint32Array; readonly stride: 2; readonly value: Lut };
  /** An EnvironmentMap handle field (0 is none). */
  EnvironmentMap: { readonly array: Uint32Array; readonly stride: 2; readonly value: EnvironmentMap };
  /** A ParticleEmitter handle field (0 is none). */
  ParticleEmitter: { readonly array: Uint32Array; readonly stride: 2; readonly value: ParticleEmitter };
  /** An AoMethod field: the variant's index. */
  AoMethod: { readonly array: Uint32Array; readonly stride: 1; readonly value: AoMethod };
  /** A DofQuality field: the variant's index. */
  DofQuality: { readonly array: Uint32Array; readonly stride: 1; readonly value: DofQuality };
  /** A SpriteBlend field: the variant's index. */
  SpriteBlend: { readonly array: Uint32Array; readonly stride: 1; readonly value: SpriteBlend };
  /** A StripBlend field: the variant's index. */
  StripBlend: { readonly array: Uint32Array; readonly stride: 1; readonly value: StripBlend };
  /** A Palette handle field (0 is none). */
  Palette: { readonly array: Uint32Array; readonly stride: 2; readonly value: Palette };
  /** A DitherPattern field: the variant's index. */
  DitherPattern: { readonly array: Uint32Array; readonly stride: 1; readonly value: DitherPattern };
  /** A StyliseSpace field: the variant's index. */
  StyliseSpace: { readonly array: Uint32Array; readonly stride: 1; readonly value: StyliseSpace };
  /** A ReflectionProbeMode field: the variant's index. */
  ReflectionProbeMode: { readonly array: Uint32Array; readonly stride: 1; readonly value: ReflectionProbeMode };
  /** A LineMode field: the variant's index. */
  LineMode: { readonly array: Uint32Array; readonly stride: 1; readonly value: LineMode };
  /** A RenderTargetFormat field: the variant's index. */
  RenderTargetFormat: { readonly array: Uint32Array; readonly stride: 1; readonly value: RenderTargetFormat };
  /** A RenderTarget handle field (0 is none). */
  RenderTarget: { readonly array: Uint32Array; readonly stride: 2; readonly value: RenderTarget };
  /** A Post handle field (0 is none). */
  Post: { readonly array: Uint32Array; readonly stride: 2; readonly value: Post };
  /** A TextureArray handle field (0 is none). */
  TextureArray: { readonly array: Uint32Array; readonly stride: 2; readonly value: TextureArray };
  /** An Asset handle field. */
  Asset: { readonly array: Uint32Array; readonly stride: 2; readonly value: Asset };
  /** A Sound handle field. */
  Sound: { readonly array: Uint32Array; readonly stride: 2; readonly value: Sound };
  /** A SkeletonAsset handle field. */
  SkeletonAsset: { readonly array: Uint32Array; readonly stride: 2; readonly value: SkeletonAsset };
  /** A ClipAsset handle field. */
  ClipAsset: { readonly array: Uint32Array; readonly stride: 2; readonly value: ClipAsset };
  /** A GraphAsset handle field. */
  GraphAsset: { readonly array: Uint32Array; readonly stride: 2; readonly value: GraphAsset };
  /** A RootMotion field: the variant's index. */
  RootMotion: { readonly array: Uint32Array; readonly stride: 1; readonly value: RootMotion };
  /** A BlendshapeSource field: the variant's index. */
  BlendshapeSource: { readonly array: Uint32Array; readonly stride: 1; readonly value: BlendshapeSource };
  /** An IkSolver field: the variant's index. */
  IkSolver: { readonly array: Uint32Array; readonly stride: 1; readonly value: IkSolver };
  /** An IkSpace field: the variant's index. */
  IkSpace: { readonly array: Uint32Array; readonly stride: 1; readonly value: IkSpace };
  /** A FlipbookSheet handle field. */
  FlipbookSheet: { readonly array: Uint32Array; readonly stride: 2; readonly value: FlipbookSheet };
  /** A Rig2D handle field. */
  Rig2D: { readonly array: Uint32Array; readonly stride: 2; readonly value: Rig2D };
  /** A Tween handle field. */
  Tween: { readonly array: Uint32Array; readonly stride: 2; readonly value: Tween };
  /** A TilemapAsset handle field. */
  TilemapAsset: { readonly array: Uint32Array; readonly stride: 2; readonly value: TilemapAsset };
  /** An AudioBus field: the variant's index. */
  AudioBus: { readonly array: Uint32Array; readonly stride: 1; readonly value: AudioBus };
  /** A Navigation handle field. */
  Navigation: { readonly array: Uint32Array; readonly stride: 2; readonly value: Navigation };
  /** A Behaviour handle field. */
  Behaviour: { readonly array: Uint32Array; readonly stride: 2; readonly value: Behaviour };
  /** A TerrainData handle field. */
  TerrainData: { readonly array: Uint32Array; readonly stride: 2; readonly value: TerrainData };
  /** A Video handle field (0 is none). */
  Video: { readonly array: Uint32Array; readonly stride: 2; readonly value: Video };
  /** A Path handle field (0 is none). */
  Path: { readonly array: Uint32Array; readonly stride: 2; readonly value: Path };
  /** A PathKind field: the variant's index. */
  PathKind: { readonly array: Uint32Array; readonly stride: 1; readonly value: PathKind };
  /** A TimelineAsset handle field (0 is none). */
  TimelineAsset: { readonly array: Uint32Array; readonly stride: 2; readonly value: TimelineAsset };
}

// ── math ────────────────────────────────────────────────────────
// Plain math values in the three.js vocabulary. They are data: the engine owns the operations, so results are identical on every host.

/** A 2D vector, as three.js Vector2. */
export interface Vector2 {
  /** The x component. */
  x: number;
  /** The y component. */
  y: number;
}

/** A 3D vector, as three.js Vector3. The engine is right-handed with +Y up, as three.js. */
export interface Vector3 {
  /** The x component. */
  x: number;
  /** The y component. */
  y: number;
  /** The z component. */
  z: number;
}

/** A unit quaternion rotation, as three.js Quaternion. The zero value is the identity. */
export interface Quaternion {
  /** The x component. */
  x: number;
  /** The y component. */
  y: number;
  /** The z component. */
  z: number;
  /**
   * The w (scalar) component.
   *
   * @default 1
   */
  w: number;
}

/**
 * A 4x4 matrix, as three.js Matrix4: sixteen elements in column-major order. The zero value is the identity.
 */
export interface Matrix4 {
  /**
   * The elements, column-major, as three.js Matrix4.elements.
   *
   * @default [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]
   */
  elements: [number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number];
}

/**
 * A linear RGB colour, as three.js Color with colour management on. Components are usually between 0 and 1; above 1 is HDR.
 */
export interface Color {
  /** The red component. */
  r: number;
  /** The green component. */
  g: number;
  /** The blue component. */
  b: number;
}

/** A ray, as three.js Ray. */
export interface Ray {
  /** Where the ray starts. */
  readonly origin: Vector3;
  /** The unit direction the ray points in. */
  readonly direction: Vector3;
}

/**
 * An axis-aligned box, as three.js Box3. A box whose min exceeds its max on an axis is empty (box3.empty() makes one: min +Infinity, max -Infinity).
 */
export interface Box3 {
  /** The lower corner. */
  readonly min: Vector3;
  /** The upper corner. */
  readonly max: Vector3;
}

/** A sphere, as three.js Sphere. A negative radius is empty. */
export interface Sphere {
  /** The centre. */
  readonly center: Vector3;
  /**
   * The radius.
   *
   * @unit m (metres)
   */
  readonly radius: number;
}

/**
 * A plane, as three.js Plane: the points p with dot(normal, p) + constant = 0; normal is unit length, and the positive side is where it points.
 */
export interface Plane {
  /** The unit normal. */
  readonly normal: Vector3;
  /**
   * The signed distance from the origin to the plane, negated along the normal (three.js Plane.constant).
   *
   * @unit m (metres)
   */
  readonly constant: number;
}

/**
 * A view volume, as three.js Frustum: six planes whose normals point inwards (right, left, bottom, top, far, near, three.js's order). frustum.fromCamera and frustum.fromProjectionMatrix make one.
 */
export interface Frustum {
  /** The six planes, normals inwards. */
  readonly planes: readonly Plane[];
}

/**
 * Pure functions over Vector3, as three.js Vector3's methods of the same names: each returns a new value and changes none of its arguments. The value `vec3`: `vec3.add(a, b)`.
 * No member reads Math.sin, Math.pow or the clock: results are the same bits on every host, so a system may store them in world state.
 */
export interface Vector3Math {
  /**
   * a + b, as three.js Vector3.addVectors.
   *
   * @param a The first vector.
   * @param b The second vector.
   */
  add(a: Vector3, b: Vector3): Vector3;
  /**
   * a - b, as three.js Vector3.subVectors.
   *
   * @param a The vector subtracted from.
   * @param b The vector subtracted.
   */
  sub(a: Vector3, b: Vector3): Vector3;
  /**
   * v * s, as three.js Vector3.multiplyScalar.
   *
   * @param v The vector.
   * @param s The factor.
   */
  scale(v: Vector3, s: number): Vector3;
  /**
   * The dot product, as three.js Vector3.dot.
   *
   * @param a The first vector.
   * @param b The second vector.
   */
  dot(a: Vector3, b: Vector3): number;
  /**
   * The cross product a x b (right-handed), as three.js Vector3.crossVectors.
   *
   * @param a The first vector.
   * @param b The second vector.
   */
  cross(a: Vector3, b: Vector3): Vector3;
  /**
   * The Euclidean length, as three.js Vector3.length.
   *
   * @param v The vector.
   */
  length(v: Vector3): number;
  /**
   * The unit vector along v, as three.js Vector3.normalize; the zero vector stays zero.
   *
   * @param v The vector.
   */
  normalize(v: Vector3): Vector3;
  /**
   * a + (b - a) * t, as three.js Vector3.lerpVectors.
   *
   * @param a The value at t = 0.
   * @param b The value at t = 1.
   * @param t The fraction (not clamped).
   */
  lerp(a: Vector3, b: Vector3, t: number): Vector3;
  /**
   * The distance between two points, as three.js Vector3.distanceTo.
   *
   * @param a The first point.
   * @param b The second point.
   */
  distance(a: Vector3, b: Vector3): number;
  /**
   * v rotated by q, as three.js Vector3.applyQuaternion.
   *
   * @param v The vector.
   * @param q The unit quaternion.
   */
  applyQuaternion(v: Vector3, q: Quaternion): Vector3;
}

/**
 * Pure functions over Vector2, as three.js Vector2's methods of the same names, each returning a new value. The value `vec2`: `vec2.add(a, b)`. The same bits on every host.
 */
export interface Vector2Math {
  /**
   * a + b, as three.js Vector2.addVectors.
   *
   * @param a The first vector.
   * @param b The second vector.
   */
  add(a: Vector2, b: Vector2): Vector2;
  /**
   * a - b, as three.js Vector2.subVectors.
   *
   * @param a The vector subtracted from.
   * @param b The vector subtracted.
   */
  sub(a: Vector2, b: Vector2): Vector2;
  /**
   * v * s, as three.js Vector2.multiplyScalar.
   *
   * @param v The vector.
   * @param s The factor.
   */
  scale(v: Vector2, s: number): Vector2;
  /**
   * The dot product, as three.js Vector2.dot.
   *
   * @param a The first vector.
   * @param b The second vector.
   */
  dot(a: Vector2, b: Vector2): number;
  /**
   * The 2D cross product a.x * b.y - a.y * b.x (the z of the 3D cross product), as three.js Vector2.cross: positive when b is counter-clockwise of a.
   *
   * @param a The first vector.
   * @param b The second vector.
   */
  cross(a: Vector2, b: Vector2): number;
  /**
   * The Euclidean length, as three.js Vector2.length.
   *
   * @param v The vector.
   */
  length(v: Vector2): number;
  /**
   * The unit vector along v, as three.js Vector2.normalize; the zero vector stays zero.
   *
   * @param v The vector.
   */
  normalize(v: Vector2): Vector2;
  /**
   * a + (b - a) * t, as three.js Vector2.lerpVectors.
   *
   * @param a The value at t = 0.
   * @param b The value at t = 1.
   * @param t The fraction (not clamped).
   */
  lerp(a: Vector2, b: Vector2, t: number): Vector2;
  /**
   * The distance between two points, as three.js Vector2.distanceTo.
   *
   * @param a The first point.
   * @param b The second point.
   */
  distance(a: Vector2, b: Vector2): number;
  /**
   * v rotated counter-clockwise by angle about center, as three.js Vector2.rotateAround (the 2D form of applyQuaternion).
   *
   * @param v The point.
   * @param center The pivot.
   * @param angle The angle in radians, counter-clockwise.
   */
  rotateAround(v: Vector2, center: Vector2, angle: number): Vector2;
}

/**
 * Pure functions over Quaternion, as three.js Quaternion's methods, each returning a new value. The value `quat`: `quat.multiply(a, b)`. The same bits on every host (trigonometry from + - * / and sqrt).
 */
export interface QuaternionMath {
  /**
   * a * b: b's rotation, then a's, as three.js Quaternion.multiplyQuaternions.
   *
   * @param a The rotation applied second.
   * @param b The rotation applied first.
   */
  multiply(a: Quaternion, b: Quaternion): Quaternion;
  /**
   * The spherical interpolation from a to b by t along the shorter arc, as three.js Quaternion.slerpQuaternions.
   *
   * @param a The value at t = 0.
   * @param b The value at t = 1.
   * @param t The fraction, 0 to 1.
   */
  slerp(a: Quaternion, b: Quaternion, t: number): Quaternion;
  /**
   * The rotation of Euler angles, as three.js Quaternion.setFromEuler: the x, y and z of `angles` in radians, applied in `order` (three.js Euler.order, intrinsic: "XYZ" turns about x, then the turned y, then the turned z).
   *
   * @param angles The angles about x, y and z, in radians.
   * @param order "XYZ", "YXZ", "ZXY", "ZYX", "YZX" or "XZY". Default `"XYZ"`.
   */
  fromEuler(angles: Vector3, order?: string): Quaternion;
  /**
   * The inverse rotation (the conjugate of a unit quaternion), as three.js Quaternion.invert.
   *
   * @param q The rotation.
   */
  invert(q: Quaternion): Quaternion;
  /**
   * The vector v turned by q: vec3.applyQuaternion(v, q), from the rotation's side.
   *
   * @param q The rotation.
   * @param v The vector.
   */
  rotate(q: Quaternion, v: Vector3): Vector3;
}

/**
 * Pure functions over Matrix4 (column-major elements, as three.js), each returning a new value. The value `mat4`: `mat4.multiply(a, b)`.
 */
export interface Matrix4Math {
  /**
   * a * b: b's transform, then a's, as three.js Matrix4.multiplyMatrices.
   *
   * @param a The transform applied second.
   * @param b The transform applied first.
   */
  multiply(a: Matrix4, b: Matrix4): Matrix4;
  /**
   * The inverse, as three.js Matrix4.invert; a singular matrix inverts to all zeros, as three.js.
   *
   * @param m The matrix.
   */
  invert(m: Matrix4): Matrix4;
  /**
   * The point p transformed by m with the perspective divide, as three.js Vector3.applyMatrix4.
   *
   * @param m The transform.
   * @param p The point.
   */
  transformPoint(m: Matrix4, p: Vector3): Vector3;
  /**
   * The direction d turned by m's upper 3x3 and normalised, as three.js Vector3.transformDirection (translation ignored).
   *
   * @param m The transform.
   * @param d The direction.
   */
  transformDirection(m: Matrix4, d: Vector3): Vector3;
  /**
   * The transform of a position, a rotation and a scale, as three.js Matrix4.compose: what og.Transform's fields make.
   *
   * @param position The translation.
   * @param quaternion The rotation.
   * @param scale The scale.
   */
  compose(position: Vector3, quaternion: Quaternion, scale: Vector3): Matrix4;
}

/**
 * Pure functions over Box3, as three.js Box3's methods, each returning a new value. The value `box3`.
 */
export interface Box3Math {
  /**
   * The empty box (min +Infinity, max -Infinity), as new three.js Box3(): expandByPoint and union grow it.
   */
  empty(): Box3;
  /**
   * The smallest box holding every point, as three.js Box3.setFromPoints; no points is the empty box.
   *
   * @param points The points.
   */
  fromPoints(points: readonly Vector3[]): Box3;
  /**
   * Whether the box holds no point (min above max on some axis), as three.js Box3.isEmpty.
   *
   * @param box The box.
   */
  isEmpty(box: Box3): boolean;
  /**
   * Whether p is inside or on the box, as three.js Box3.containsPoint.
   *
   * @param box The box.
   * @param p The point.
   */
  containsPoint(box: Box3, p: Vector3): boolean;
  /**
   * Whether other lies wholly inside box, as three.js Box3.containsBox.
   *
   * @param box The outer box.
   * @param other The inner box.
   */
  containsBox(box: Box3, other: Box3): boolean;
  /**
   * Whether two boxes overlap or touch, as three.js Box3.intersectsBox.
   *
   * @param a The first box.
   * @param b The second box.
   */
  intersectsBox(a: Box3, b: Box3): boolean;
  /**
   * Whether the box and the sphere overlap, as three.js Box3.intersectsSphere.
   *
   * @param box The box.
   * @param sphere The sphere.
   */
  intersectsSphere(box: Box3, sphere: Sphere): boolean;
  /**
   * The box grown to hold p, as three.js Box3.expandByPoint.
   *
   * @param box The box.
   * @param p The point.
   */
  expandByPoint(box: Box3, p: Vector3): Box3;
  /**
   * The smallest box holding both, as three.js Box3.union.
   *
   * @param a The first box.
   * @param b The second box.
   */
  union(a: Box3, b: Box3): Box3;
  /**
   * The centre, as three.js Box3.getCenter; the empty box's is the origin.
   *
   * @param box The box.
   */
  getCenter(box: Box3): Vector3;
  /**
   * The size along each axis, as three.js Box3.getSize; the empty box's is zero.
   *
   * @param box The box.
   */
  getSize(box: Box3): Vector3;
  /**
   * The box holding the eight corners transformed by m, as three.js Box3.applyMatrix4.
   *
   * @param box The box.
   * @param m The transform.
   */
  applyMatrix4(box: Box3, m: Matrix4): Box3;
}

/**
 * Pure functions over Sphere, as three.js Sphere's methods, each returning a new value. The value `sphere`.
 */
export interface SphereMath {
  /**
   * A sphere around the points, as three.js Sphere.setFromPoints: centred on their bounding box's centre, with the radius of the farthest point.
   *
   * @param points The points.
   */
  fromPoints(points: readonly Vector3[]): Sphere;
  /**
   * Whether p is inside or on the sphere, as three.js Sphere.containsPoint.
   *
   * @param sphere The sphere.
   * @param p The point.
   */
  containsPoint(sphere: Sphere, p: Vector3): boolean;
  /**
   * The distance from the surface to p, negative inside, as three.js Sphere.distanceToPoint.
   *
   * @param sphere The sphere.
   * @param p The point.
   */
  distanceToPoint(sphere: Sphere, p: Vector3): number;
  /**
   * Whether two spheres overlap or touch, as three.js Sphere.intersectsSphere.
   *
   * @param a The first sphere.
   * @param b The second sphere.
   */
  intersectsSphere(a: Sphere, b: Sphere): boolean;
  /**
   * Whether the sphere and the box overlap, as three.js Sphere.intersectsBox.
   *
   * @param sphere The sphere.
   * @param box The box.
   */
  intersectsBox(sphere: Sphere, box: Box3): boolean;
  /**
   * The box around the sphere, as three.js Sphere.getBoundingBox.
   *
   * @param sphere The sphere.
   */
  getBoundingBox(sphere: Sphere): Box3;
  /**
   * The sphere transformed by m (its radius by m's largest axis scale), as three.js Sphere.applyMatrix4.
   *
   * @param sphere The sphere.
   * @param m The transform.
   */
  applyMatrix4(sphere: Sphere, m: Matrix4): Sphere;
}

/**
 * Pure functions over Plane, as three.js Plane's methods, each returning a new value. The value `plane`.
 */
export interface PlaneMath {
  /**
   * The plane through p with the normal n (normalised), as three.js Plane.setFromNormalAndCoplanarPoint.
   *
   * @param n The normal.
   * @param p A point on the plane.
   */
  fromNormalAndCoplanarPoint(n: Vector3, p: Vector3): Plane;
  /**
   * The plane through a, b and c, its normal (b - a) x (c - a) normalised (counter-clockwise points face you), as three.js Plane.setFromCoplanarPoints.
   *
   * @param a The first point.
   * @param b The second point.
   * @param c The third point.
   */
  fromCoplanarPoints(a: Vector3, b: Vector3, c: Vector3): Plane;
  /**
   * The same plane with a unit normal, as three.js Plane.normalize.
   *
   * @param plane The plane.
   */
  normalize(plane: Plane): Plane;
  /**
   * The signed distance from the plane to p, positive on the normal's side, as three.js Plane.distanceToPoint.
   *
   * @param plane The plane.
   * @param p The point.
   */
  distanceToPoint(plane: Plane, p: Vector3): number;
  /**
   * The point of the plane nearest p, as three.js Plane.projectPoint.
   *
   * @param plane The plane.
   * @param p The point.
   */
  projectPoint(plane: Plane, p: Vector3): Vector3;
  /**
   * Whether the plane passes through the box, as three.js Plane.intersectsBox.
   *
   * @param plane The plane.
   * @param box The box.
   */
  intersectsBox(plane: Plane, box: Box3): boolean;
  /**
   * Whether the plane passes through the sphere, as three.js Plane.intersectsSphere.
   *
   * @param plane The plane.
   * @param sphere The sphere.
   */
  intersectsSphere(plane: Plane, sphere: Sphere): boolean;
}

/**
 * Pure functions over Frustum, as three.js Frustum's methods. The value `frustum`: `frustum.intersectsBox(frustum.fromCamera(camera, 16 / 9), box)` culls as the renderer would.
 */
export interface FrustumMath {
  /**
   * The frustum of a view-projection matrix (projection * inverse camera world), as three.js Frustum.setFromProjectionMatrix (WebGL clip space, z from -1 to 1).
   *
   * @param m The view-projection matrix.
   */
  fromProjectionMatrix(m: Matrix4): Frustum;
  /**
   * The frustum og.Camera sees at an aspect ratio (width over height): its perspective projection (fov, near, far, zoom) or orthographic one (height, zoom), from its position and quaternion; shake, punch and kick, which are visual only, are left out.
   *
   * @param camera The og.Camera resource's value (world.resource(og.Camera)).
   * @param aspect The view's width over its height.
   */
  fromCamera(camera: Camera, aspect: number): Frustum;
  /**
   * Whether p is inside the frustum, as three.js Frustum.containsPoint.
   *
   * @param frustum The frustum.
   * @param p The point.
   */
  containsPoint(frustum: Frustum, p: Vector3): boolean;
  /**
   * Whether the box is at least partly inside, as three.js Frustum.intersectsBox (conservative near the corners, as three.js).
   *
   * @param frustum The frustum.
   * @param box The box.
   */
  intersectsBox(frustum: Frustum, box: Box3): boolean;
  /**
   * Whether the sphere is at least partly inside, as three.js Frustum.intersectsSphere.
   *
   * @param frustum The frustum.
   * @param sphere The sphere.
   */
  intersectsSphere(frustum: Frustum, sphere: Sphere): boolean;
}

/**
 * Pure functions over Ray, as three.js Ray's methods. The value `ray`. Each intersection answers the nearest point at or after the origin, or null.
 */
export interface RayMath {
  /**
   * The point t along the ray, as three.js Ray.at.
   *
   * @param ray The ray.
   * @param t The distance along the direction, in metres.
   */
  at(ray: Ray, t: number): Vector3;
  /**
   * The distance from p to the nearest point of the ray, as three.js Ray.distanceToPoint.
   *
   * @param ray The ray.
   * @param p The point.
   */
  distanceToPoint(ray: Ray, p: Vector3): number;
  /**
   * Where the ray enters the box (where it leaves, when it starts inside), as three.js Ray.intersectBox.
   *
   * @param ray The ray.
   * @param box The box.
   */
  intersectBox(ray: Ray, box: Box3): Vector3 | null;
  /**
   * Where the ray meets the sphere (where it leaves, when it starts inside), as three.js Ray.intersectSphere.
   *
   * @param ray The ray.
   * @param sphere The sphere.
   */
  intersectSphere(ray: Ray, sphere: Sphere): Vector3 | null;
  /**
   * Where the ray meets the plane (its origin when it lies in the plane), as three.js Ray.intersectPlane; null when parallel or pointing away.
   *
   * @param ray The ray.
   * @param plane The plane.
   */
  intersectPlane(ray: Ray, plane: Plane): Vector3 | null;
  /**
   * Where the ray meets the triangle a, b, c, as three.js Ray.intersectTriangle; with backfaceCulling a triangle facing away (clockwise from the origin) is missed.
   *
   * @param ray The ray.
   * @param a The first corner.
   * @param b The second corner.
   * @param c The third corner.
   * @param backfaceCulling Miss triangles facing away. Default `false`.
   */
  intersectTriangle(ray: Ray, a: Vector3, b: Vector3, c: Vector3, backfaceCulling?: boolean): Vector3 | null;
}

/**
 * Pure functions over Color (linear RGB), as three.js Color's methods with colour management on: hex and HSL are sRGB, converted to and from linear as three.js does. The value `color`: `color.fromHex(0xff8800)`. The same bits on every host (no Math.pow).
 */
export interface ColorMath {
  /**
   * The linear colour of an sRGB hex number, as new three.js Color(0xrrggbb).
   *
   * @param hex 0xrrggbb, sRGB.
   */
  fromHex(hex: number): Color;
  /**
   * The sRGB hex number of a linear colour, each channel clamped to 0..1 and rounded, as three.js Color.getHex.
   *
   * @param c The colour.
   */
  toHex(c: Color): number;
  /**
   * a + (b - a) * t in linear RGB, as three.js Color.lerpColors.
   *
   * @param a The value at t = 0.
   * @param b The value at t = 1.
   * @param t The fraction.
   */
  lerp(a: Color, b: Color, t: number): Color;
  /**
   * The linear colour of sRGB hue, saturation and lightness, as three.js Color.setHSL (hue wraps, saturation and lightness clamp to 0..1).
   *
   * @param h The hue, 0 to 1 around the wheel (0 red, 1/3 green, 2/3 blue).
   * @param s The saturation, 0 to 1.
   * @param l The lightness, 0 to 1.
   */
  fromHsl(h: number, s: number, l: number): Color;
}

/**
 * Vector3's pure functions: `vec3.add(a, b)`, `vec3.normalize(v)`, `vec3.applyQuaternion(v, q)`.
 */
export declare const vec3: Vector3Math;

/** Vector2's pure functions: `vec2.add(a, b)`, `vec2.rotateAround(v, c, angle)`. */
export declare const vec2: Vector2Math;

/**
 * Quaternion's pure functions: `quat.multiply(a, b)`, `quat.slerp(a, b, t)`, `quat.fromEuler({ x, y, z })`.
 */
export declare const quat: QuaternionMath;

/**
 * Matrix4's pure functions: `mat4.multiply(a, b)`, `mat4.invert(m)`, `mat4.transformPoint(m, p)`.
 */
export declare const mat4: Matrix4Math;

/** Box3's pure functions: `box3.fromPoints(points)`, `box3.intersectsBox(a, b)`. */
export declare const box3: Box3Math;

/** Sphere's pure functions: `sphere.containsPoint(s, p)`. */
export declare const sphere: SphereMath;

/** Plane's pure functions: `plane.distanceToPoint(pl, p)`. */
export declare const plane: PlaneMath;

/**
 * Frustum's pure functions: `frustum.fromCamera(camera, aspect)`, `frustum.intersectsBox(f, box)`.
 */
export declare const frustum: FrustumMath;

/** Ray's pure functions: `ray.intersectBox(r, box)`, `ray.intersectTriangle(r, a, b, c)`. */
export declare const ray: RayMath;

/**
 * Color's pure functions: `color.fromHex(0xff8800)`, `color.toHex(c)`, `color.fromHsl(h, s, l)`.
 */
export declare const color: ColorMath;

// ── reflect ─────────────────────────────────────────────────────
// Reflection: components and resources are declared once with their field layout and metadata, and registered by name.
// The inspector, Tune, the probe, scene text and the wiki read this one schema. Each column factory below makes one field.
// A field holds one value: there is no array or list column. A few values are numbered fields (`p0` to `p7`, as og.CameraRail does); a list of things is an entity each, with its own component, looped over by a query (a ring of skid marks, the laps of a race); a long record (a lap's taped inputs) is text in `str` fields of at most 4,096 bytes each, chunked over entities. Keep none of it in module variables: they are not world state, so a snapshot, a replay and a hot reload lose them.
// A handle (an og.load answer, a texture, a sound, a geometry, a material, an entity) is a 64-bit number: keep it in a field of its own kind (`Asset()`, `Texture()`, `Sound()`, `Geometry()`, `Material()`, `Entity()`, the factories of the API's handle types), in a component or a resource. A `u32` field refuses one (handles pass 2^32: `4294967298 is not a u32`); an `f64` holds the number but loses its type.

/**
 * Metadata of one field. Ranges and units are what Tune and the probe's bounds show. Nothing clamps writes: game code's writes (world.set, a resource field, a column) are stored as given, and writes from outside the game (the probe's `set`, `patch` and sweeps, Tune, a look's fields at cook) are refused outside min..max with the range named.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface FieldOptions {
  /**
   * The smallest meaningful value.
   *
   * @default null
   */
  readonly min?: number | null;
  /**
   * The largest meaningful value.
   *
   * @default null
   */
  readonly max?: number | null;
  /**
   * The increment Tune uses.
   *
   * @default null
   */
  readonly step?: number | null;
  /**
   * A unit name from the API's unit table, or empty.
   *
   * @default ""
   */
  readonly unit?: string;
  /**
   * The person-facing name; empty uses the field name.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * One sentence on what the field means.
   *
   * @default ""
   */
  readonly doc?: string;
  /**
   * Publish the field to Tune and to the probe's bounds and patch.
   *
   * @default false
   */
  readonly tunable?: boolean;
}

/**
 * A 32-bit float, the default numeric field.
 *
 * A `f32` field: stored in a `Float32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function f32(value?: number, options?: FieldOptions): FieldSpec<"f32">;

/**
 * A 64-bit float, for accumulated values that must not drift, such as elapsed time.
 *
 * A `f64` field: stored in a `Float64Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function f64(value?: number, options?: FieldOptions): FieldSpec<"f64">;

/**
 * A 32-bit unsigned integer: counters, flags, indices.
 *
 * A `u32` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function u32(value?: number, options?: FieldOptions): FieldSpec<"u32">;

/**
 * A 32-bit signed integer.
 *
 * A `i32` field: stored in a `Int32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function i32(value?: number, options?: FieldOptions): FieldSpec<"i32">;

/**
 * A boolean, stored as one byte: 0 or 1.
 *
 * A `bool` field: stored in a `Uint8Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function bool(value?: boolean, options?: FieldOptions): FieldSpec<"bool">;

/**
 * A Vector2 field: x, y interleaved.
 *
 * A `Vector2` field: stored in a `Float32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Vector2(value?: Vector2, options?: FieldOptions): FieldSpec<"Vector2">;

/**
 * A Vector3 field: x, y, z interleaved.
 *
 * A `Vector3` field: stored in a `Float32Array` with stride 3.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Vector3(value?: Vector3, options?: FieldOptions): FieldSpec<"Vector3">;

/**
 * A Quaternion field: x, y, z, w interleaved.
 *
 * A `Quaternion` field: stored in a `Float32Array` with stride 4.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Quaternion(value?: Quaternion, options?: FieldOptions): FieldSpec<"Quaternion">;

/**
 * A Matrix4 field: sixteen column-major elements per entity.
 *
 * A `Matrix4` field: stored in a `Float32Array` with stride 16.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Matrix4(value?: Matrix4, options?: FieldOptions): FieldSpec<"Matrix4">;

/**
 * A Color field: r, g, b interleaved.
 *
 * A `Color` field: stored in a `Float32Array` with stride 3.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Color(value?: Color, options?: FieldOptions): FieldSpec<"Color">;

/**
 * Text of at most `capacity` bytes of UTF-8, stored inline and zero-padded, so it is world state like a number: a player's name, a label. `str(48)` holds any 12 characters (four bytes covers every script). The capacity is 1 to 4,096 bytes: a larger one is refused when the game starts (`a string field holds 1 to 4096 bytes`), so longer text goes in chunks, an entity each.
 * Get and set it as a JavaScript string; text over the capacity is refused with the fix named (an error value from world.insert/world.set, a throw from a resource's setter), never truncated. Its view is the raw bytes.
 *
 * A `str` field: stored in a `Uint8Array` whose stride is its capacity.
 * @param capacity The most bytes of UTF-8 it holds, 1 to 4096: four per character covers any script.
 * @param value The field's default; omitted, the empty string.
 * @param options Label and docs (a string has no range and is not tunable).
 */
export declare function str(capacity: number, value?: string, options?: FieldOptions): FieldSpec<"str">;

/**
 * Declare a component: plain data stored per entity in engine memory, one column per field.
 * Registration is by name, so a hot reload re-registers the same component and migrates its fields by name.
 *
 * @param name The component's unique name, PascalCase, as it appears in scenes and dumps.
 * @param fields The layout: field names to column factories, as `{ hp: f32(100) }`.
 * @returns The token systems, queries and the world take.
 */
export declare function defineComponent<F extends Fields>(name: string, fields: F): ComponentType<F>;

/**
 * Declare a resource: a reflected singleton, such as a tuning table. Its fields start at their defaults.
 *
 * @param name The resource's unique name, PascalCase.
 * @param fields The layout, as for defineComponent.
 * @returns The token `world.resource` and system declarations take.
 */
export declare function defineResource<F extends Fields>(name: string, fields: F): ResourceType<F>;

// ── world ───────────────────────────────────────────────────────
// The world: an archetype ECS. Entities are generational handles; components are columns in engine memory.
// Structural changes (spawn, despawn, insert, remove) invalidate typed views; the API refuses a stale view.

/**
 * An entity: an index and a generation. The value is index + generation * 2^32, with the generation below 2^21, so it stays an exact JavaScript number.
 * Generations start at 1, so 0 is never a live entity.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Entity = number & { readonly [handleType]: "Entity" };

/**
 * An Entity field: (index, generation) pairs.
 *
 * A `Entity` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Entity(value?: Entity, options?: FieldOptions): FieldSpec<"Entity">;

/**
 * How to spawn an entity. `{}` spawns an unnamed entity at the origin with a Transform.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SpawnDesc {
  /**
   * A unique name; the probe and the command channel address the entity by it. Empty leaves it unnamed.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * The parent entity; the new entity's Transform is then relative to it.
   *
   * @default null
   */
  readonly parent?: Entity | null;
  /**
   * The initial Transform.position.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly position?: Vector3;
  /**
   * The initial Transform.quaternion.
   *
   * @default { x: 0, y: 0, z: 0, w: 1 }
   */
  readonly quaternion?: Quaternion;
  /**
   * The initial Transform.scale.
   *
   * @default { x: 1, y: 1, z: 1 }
   */
  readonly scale?: Vector3;
  /**
   * Spawn without a Transform: a pure data entity.
   *
   * @default false
   */
  readonly bare?: boolean;
}

/**
 * How world.restore takes a save.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RestoreDesc {
  /**
   * Map a type whose layout changed since the save field by field, by name, as a hot reload does: a field added since takes its default, a removed one is dropped, an enum value is mapped by its variant's name, a field of another type starts at its default, and a type this build no longer registers is dropped. Off, such a save is refused naming the type and its fields.
   *
   * @default false
   */
  readonly migrate?: boolean;
}

/**
 * A world.restore applied: the tick after it (the first whole tick after a save taken mid-tick, whose next step completes the saved tick) delivers this, read through world.events like any event. Everything in the world came back as saved, request numbers and resources the session filled included; the session did not (og-game §17), so a system that reads this resets what belongs to the session: it forgets requests answered long ago, asks the store again for the settings and progress, and closes a menu the player had open. Sent as that tick starts, not when the restore applies, so the world between the two hashes as the save did.
 *
 * The payload of the `WorldRestored` event (token `og.WorldRestored`).
 */
export interface WorldRestored {
  /**
   * The tick the world was at when the restore applied.
   *
   * @unit ticks (fixed simulation steps)
   */
  readonly from: number;
  /**
   * The saved tick: og.Time's tick right after the restore.
   *
   * @unit ticks (fixed simulation steps)
   */
  readonly tick: number;
}

/**
 * Which entities a query matches.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface QueryDesc {
  /**
   * Components an entity must have.
   *
   * @default []
   */
  readonly with?: readonly AnyComponent[];
  /**
   * Components an entity must not have.
   *
   * @default []
   */
  readonly without?: readonly AnyComponent[];
}

/**
 * The world a system, a command or a game callback is handed. Game code never holds a world beyond the call it was passed to.
 */
export interface World {
  /** Fixed steps taken since the world was created. */
  readonly tick: number;
  /**
   * Spawn an entity.
   *
   * @param desc Name, parent and initial transform.
   * @returns The new entity, or why it was refused (a duplicate name, a dead parent).
   */
  spawn(desc?: SpawnDesc): Result<Entity>;
  /**
   * Despawn an entity and its descendants.
   *
   * @param entity The entity.
   * @returns Refused for a dead entity.
   */
  despawn(entity: Entity): Result<void>;
  /**
   * Whether an entity handle is live.
   *
   * @param entity The entity.
   */
  alive(entity: Entity): boolean;
  /**
   * Whether an entity has a component.
   *
   * @param entity The entity.
   * @param component The component token.
   */
  has<C extends AnyComponent>(entity: Entity, component: C): boolean;
  /**
   * Add a component to an entity, or overwrite it when present. A structural change.
   *
   * @param entity The entity.
   * @param component The component token.
   * @param value Field values; omitted fields take their defaults.
   * @returns Refused for a dead entity or an unregistered component.
   */
  insert<C extends AnyComponent>(entity: Entity, component: C, value?: Partial<Value<C>>): Result<void>;
  /**
   * Remove a component from an entity. A structural change.
   *
   * @param entity The entity.
   * @param component The component token.
   * @returns Refused for a dead entity or a component it does not have.
   */
  remove<C extends AnyComponent>(entity: Entity, component: C): Result<void>;
  /**
   * Copy one entity's component value. For bulk work use a system's typed views.
   *
   * @param entity The entity.
   * @param component The component token.
   */
  get<C extends AnyComponent>(entity: Entity, component: C): Result<Value<C>>;
  /**
   * Write some fields of one entity's component.
   *
   * @param entity The entity.
   * @param component The component token.
   * @param value The fields to write.
   * @returns Refused for a dead entity or a component it does not have.
   */
  set<C extends AnyComponent>(entity: Entity, component: C, value: Partial<Value<C>>): Result<void>;
  /**
   * A resource's live value: the same handle in a world every time, its fields typed reads and writes of the resource's packed record (no JSON, whatever its size). The record is fetched again only by the first read after another engine call or a system boundary, and writes reach the engine in one call before the next engine call and when the system or command returns, so two handles agree and a read after an engine call sees the engine's value (og-core §4). Inside a system, the resource must be among the system's reads or writes (writes to set a field): an undeclared one throws in a dev or release build, naming the system and the resource, and fails the tick; a publish build refuses it and logs it once per system in the engine log.
   *
   * @param resource The resource token.
   * @returns Refused for an unregistered resource (or, in a publish build, one the running system did not declare).
   */
  resource<R extends AnyResource>(resource: R): Result<Value<R>>;
  /**
   * The entity with a name.
   *
   * @param name The name given at spawn.
   * @returns Null when no live entity has the name.
   */
  find(name: string): Entity | null;
  /**
   * An entity's name.
   *
   * @param entity The entity.
   * @returns Null for an unnamed or dead entity.
   */
  nameOf(entity: Entity): string | null;
  /**
   * A query over the current archetypes, for commands and callbacks (GameDesc.start, a command, a UI callback). Inside a system it is refused, by design: a system receives its one query (every component of its reads and writes) as its first argument; loop over another archetype in a system of its own.
   *
   * @param desc The components to match.
   */
  query(desc?: QueryDesc): Result<Query>;
  /**
   * The events of one type emitted during the previous tick and this one, oldest first. Inside a system the event must be among its reads or writes (undeclared: thrown, or in a publish build refused and logged once).
   *
   * @param event The event token.
   */
  events<E extends AnyEvent>(event: E): readonly Value<E>[];
  /**
   * Emit an event; readers see it this tick and the next. Inside a system the event must be among its writes (undeclared: thrown, or in a publish build refused and logged once).
   *
   * @param event The event token.
   * @param payload The event's fields.
   */
  emit<E extends AnyEvent>(event: E, payload: Value<E>): Result<void>;
  /**
   * Spawn a loaded scene or glTF asset into the world: a root entity from the desc, then (for a scene) every scene entity under it with its components and names, or (for a glTF) a Model on the root.
   * Scene entity names must be free: a name already in use refuses the whole instantiation before anything spawns.
   *
   * @param asset A ready asset from og.load.
   * @param desc Name, parent and transform of the root entity.
   * @returns The root entity, or why it was refused (an asset still loading, a failed asset).
   */
  instantiate(asset: Asset, desc?: SpawnDesc): Result<Entity>;
  /**
   * The whole world as text, to save a game's progress in the platform store: every entity with its components and names, every resource (og.Time's clock and the og.Rng streams too), the pending events and the modules' state (physics, animation, flipbooks, tweens, tilemap and terrain edits), each type with its schema hash, deflated and base64-encoded after `og-save:1:`. Every build has it, publish included, and the same world saves to the same text on both hosts. Save it with og.platform.saveSlot (`{ slot: "slot1", text, meta: { name, chimes } }`), which spreads it over as many store keys as it needs (a key holds 64 KiB) and writes the slot's manifest last, then og.platform.loadSlot and world.restore of the answer's `text` (examples/hollowbell, a 2D platformer of 189 entities on a 96 x 48 tile map, saves to about 25 KiB; examples/kestrel-stair, a 3D world of 93 entities with skinned characters, cloths and minds, to about 78 KB; a world of a few entities to about 11 KiB, most of it the table of every registered type). Inside a system it is the world in the middle of that tick, as that system sees it: what earlier systems wrote this tick is in it, a structural change the system queued (applied when it returns) is not, and it names the system, so a world restored from it completes that tick: its next step runs the systems after this one and the engine's (physics included) with the tick's own events, then ends the tick, and the world goes on tick for tick as the one that saved did (a replay from it matches; make the snapshot the system's last write of the tick, since what it does after the call is not in the save). Between ticks (GameDesc.start, a command, a UI callback) it is the world between two ticks. Physics carries its mesh, convex-hull and height-field colliders by their geometry's handle and scale, not their triangles (og.physics.setMesh). Asset and material handles in components are saved as their numbers: load what a saved entity uses in the same order on every launch (in GameDesc.start) before restoring.
   *
   * @returns The save, or why not (the world could not be read).
   */
  snapshot(): Result<string>;
  /**
   * Replace the world with a save that world.snapshot wrote: entities, components, names, resources, events and the modules' state, so the restored world hashes as the saved one did, on either host. The session stays: input (og.Input, og.Pointers, og.Gamepad), the player's bus volumes (og.Audio), the UI's focus and hover, and what lives outside the world (the platform's calls and clock, the language, the loaded assets, the command channel's scheduled steps, which keep their remaining delays though og.Time takes the saved tick, so an `og run --commands` script goes on to its last step or `quit` whether the save is of a later tick or an earlier one). Called in a system or a command it applies when the tick ends, after the last system; elsewhere (GameDesc.start, between ticks) at once. Either way the tick after it delivers og.WorldRestored, for the systems that reset what lives in the session. A save a system took mid-tick restores to that moment of its tick (og.Time reads its tick): the next step completes it (the systems after the saver, then the engine's), so the ticks after it are the saved run's. The meshes and height fields its colliders name must be loaded or registered first (a save naming one that is not is refused). A save holds only the world: progress kept in module variables is not in it, so keep it in components and resources.
   *
   * @param save The text world.snapshot returned.
   * @param desc Whether a type whose layout changed since the save is mapped field by field.
   * @returns Refused, with nothing changed, for text world.snapshot did not write, a save from a newer engine, or a type whose layout changed since the save (a field added, removed or retyped, an enum's variants): the reason names the type and the fields, and the fix (`{ migrate: true }`, or keeping a saved type's fields stable between releases).
   */
  restore(save: string, desc?: RestoreDesc): Result<void>;
}

/** A set of matched archetypes. Iterate archetypes, never entities one by one. */
export interface Query {
  /** The matched archetypes this tick. */
  archetypes(): readonly ArchetypeView[];
  /** The number of matched entities. */
  count(): number;
}

/**
 * One archetype's columns for this tick. Views are invalidated by structural changes; using a stale view throws with the tick and the system it belonged to.
 */
export interface ArchetypeView {
  /** The number of entities in the archetype. */
  readonly count: number;
  /** The tick this view belongs to. */
  readonly tick: number;
  /** The entities as (index, generation) pairs: entity i is at 2i and 2i + 1. */
  readonly entities: Uint32Array;
  /**
   * The handle of entity i.
   *
   * @param index The row, from 0 to count - 1.
   */
  entity(index: number): Entity;
  /**
   * Whether the archetype carries a component.
   *
   * @param component The component token.
   */
  has<C extends AnyComponent>(component: C): boolean;
  /**
   * The typed views of a component's fields: one typed array per field, `count * stride` long.
   * Writing requires the component among the system's writes; reading among its reads or writes.
   *
   * @param component The component token.
   */
  column<C extends AnyComponent>(component: C): View<C>;
}

// ── system ──────────────────────────────────────────────────────
// Systems: functions the engine calls once per tick with their matched archetypes.
// A system declares what it reads and writes; a write to an undeclared component is refused at registration with the field name.
// Its query is every component of its reads and writes together: an entity matches only when it has all of them (and none of `without`). A component declared to read one entity with world.get (the player's car) therefore narrows the loop to entities that have it too, often to none, silently. world.query is refused inside a system, by design (a system reaches only what it declared, so its access is known before it runs): loop over each archetype in a system of its own, declaring only that archetype's components, and carry what crosses between systems in a resource.
// Resources and events it reaches through the world must be declared too: in a dev or release build (og run, og test) an undeclared read or write throws with the system's name and the record and fails the tick; a publish build refuses it and logs it once per system in the engine log, and the game keeps running. og.Rng among the reads may fork (a fork reads the seed only and leaves the stream as it is); a draw moves the stream, so it needs og.Rng among the writes.

/**
 * A system's declared access and order.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SystemDesc {
  /**
   * Components, resources and events the system reads (world.events). Every component here is also required by its query: a matched entity has all of the reads' and writes' components.
   *
   * @default []
   */
  readonly reads?: readonly (AnyReflected | AnyEvent)[];
  /**
   * Components, resources and events the system writes (world.emit). Every component here is also required by its query, with the reads'.
   *
   * @default []
   */
  readonly writes?: readonly (AnyReflected | AnyEvent)[];
  /**
   * Components a matched entity must not have.
   *
   * @default []
   */
  readonly without?: readonly AnyComponent[];
  /**
   * Names of systems that must run before this one.
   *
   * @default []
   */
  readonly after?: readonly string[];
  /**
   * Names of systems that must run after this one.
   *
   * @default []
   */
  readonly before?: readonly string[];
}

/**
 * A system's body. It is called once per tick; loop over `q.archetypes()`, then over rows.
 *
 * @param q The archetypes matching the declared components.
 * @param time The Time resource for this tick.
 * @param world The world, for resources, events and structural changes.
 */
export type SystemRun = (q: Query, time: Time, world: World) => void;

/** A registered system. */
export interface System {
  /** The system's unique name. */
  readonly name: string;
  /** What it reads. */
  readonly reads: readonly AnyReflected[];
  /** What it writes. */
  readonly writes: readonly AnyReflected[];
}

/**
 * Declare a system.
 *
 * @param name The system's unique name, PascalCase.
 * @param desc Reads, writes and order.
 * @param run The body.
 */
export declare function defineSystem(name: string, desc: SystemDesc, run: SystemRun): System;

// ── game ────────────────────────────────────────────────────────
// The game entry: what the bundle's default export declares.

/**
 * The game's own facts for the probe's state(): a small JSON object, computed from the world without changing it.
 *
 * @param world The world, read only.
 */
export type FactsFn = (world: World) => unknown;

/**
 * The game's setup: called once before the first tick, after GameDesc.scene has spawned. Spawn, insert, load and set resources here; systems then only read and write their views.
 *
 * @param world The world.
 */
export type StartFn = (world: World) => void;

/**
 * A game: its tick, scenes, systems and commands.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GameDesc {
  /**
   * The fixed tick rate. Left unset, project.ogt's `tick` applies (else 60); tick, seed, scenes and scene set both here and in project.ogt to different values refuse the engine, naming both.
   *
   * @default 60
   * @range 1..240
   * @unit hz (ticks or cycles per second)
   */
  readonly tick?: number;
  /**
   * The project seed every Rng stream derives from; left unset, project.ogt's `seed` (else 0). A replay overrides it.
   *
   * @default 0
   */
  readonly seed?: number;
  /**
   * Scene names to `.ogt` paths, as `{ main: "scenes/main.ogt" }`; left unset, project.ogt's `scenes` block.
   *
   * @default {}
   */
  readonly scenes?: Readonly<Record<string, string>>;
  /**
   * The scene loaded first; left unset, project.ogt's `scene` (else main).
   *
   * @default "main"
   */
  readonly scene?: string;
  /**
   * The systems, in declaration order before `after`/`before` constraints apply.
   *
   * @default []
   */
  readonly systems?: readonly System[];
  /**
   * The game's own resources, created at their defaults.
   *
   * @default []
   */
  readonly resources?: readonly AnyResource[];
  /**
   * Game actions registered into the command channel, by action name.
   *
   * @default {}
   */
  readonly commands?: Readonly<Record<string, CommandHandler>>;
  /**
   * Each game action's argument form and one sentence, by its name in commands, for `help`: `{ place: { usage: "<x>:<y>", doc: "Move the hero there." } }`. An action without an entry is listed by name alone.
   *
   * @default {}
   */
  readonly commandHelp?: Readonly<Record<string, CommandDesc>>;
  /**
   * The most ticks one frame may run to catch up with the clock; the accumulator drops the rest.
   *
   * @default 4
   * @range 1..60
   * @unit ticks (fixed simulation steps)
   */
  readonly maxCatchUp?: number;
  /**
   * The game's own facts for the probe's state().
   *
   * @default null
   */
  readonly facts?: FactsFn | null;
  /**
   * Called once before the first tick, after the first scene spawned. A throw fails the run: no tick runs on the half-built world, `og run`, `og test` and `og dev` exit non-zero naming the error with its stack, and the web shell shows it.
   *
   * @default null
   */
  readonly start?: StartFn | null;
  /**
   * How the renderer is created: antialiasing and shadow map size. Read once, when the host attaches the renderer.
   *
   * @default {}
   */
  readonly render?: RenderOptions;
  /**
   * A look (`looks/<name>.ogt`, og.load.look) applied after the first scene spawned and before start runs, so start may still change any of it; its antialias is the renderer's when render leaves antialias out. Empty applies none.
   *
   * @default ""
   */
  readonly look?: string;
}

/** A declared game, the bundle's default export. */
export interface Game {
  /**
   * GameDesc.tick, or 60 when it is unset (the engine then runs at project.ogt's `tick` when the host passes the project, as og run and og build do).
   */
  readonly tick: number;
  /** The systems in declaration order. */
  readonly systems: readonly System[];
}


// ── transform ───────────────────────────────────────────────────
// Transforms in the three.js vocabulary: position, quaternion and scale, composed into matrixWorld down the parent chain.

/**
 * Where an entity is, relative to its parent (or the world).
 *
 * The value of the `Transform` component (token `og.Transform`).
 */
export interface Transform {
  /**
   * The position, as three.js Object3D.position.
   *
   * @unit m (metres)
   */
  position: Vector3;
  /** The rotation, as three.js Object3D.quaternion. */
  quaternion: Quaternion;
  /**
   * The scale, as three.js Object3D.scale.
   *
   * @default { x: 1, y: 1, z: 1 }
   */
  scale: Vector3;
}

/**
 * The entity this one's Transform is relative to. Set by SpawnDesc.parent.
 *
 * The value of the `Parent` component (token `og.Parent`).
 */
export interface Parent {
  /** The parent. */
  entity: Entity;
}

/**
 * The world matrix the engine composes after the systems run, as three.js Object3D.matrixWorld.
 *
 * The value of the `MatrixWorld` component (token `og.MatrixWorld`).
 */
export interface MatrixWorld {
  /**
   * The composed world matrix.
   *
   * @readonly written by the engine
   */
  readonly matrix: Matrix4;
}





// ── time ────────────────────────────────────────────────────────
// Simulation time. Game code never reads the wall clock; it reads Time.

/**
 * The fixed tick's clock. Every value is a function of the tick count, so a replay sees the same numbers. The world keeps the time scale in a second resource, og.TimeScale (`scale`, `hitStop`, and `elapsed`, which is scaledElapsed), set through og.time.setScale and og.time.hitStop; read og.Time, which carries all of it. The probe's `get:og.Time` answers every field here, as a system's `time` does; `set:og.TimeScale.scale=0.5` slows time from a script.
 *
 * The `Time` resource (token `og.Time`).
 */
export interface Time {
  /**
   * Fixed steps taken since load.
   *
   * @unit ticks (fixed simulation steps)
   * @readonly written by the engine
   */
  readonly tick: number;
  /**
   * The length of one tick: 1 / rate.
   *
   * @unit s (seconds of simulation time)
   * @readonly written by the engine
   */
  readonly dt: number;
  /**
   * Simulation time since load: tick * dt.
   *
   * @unit s (seconds of simulation time)
   * @readonly written by the engine
   */
  readonly elapsed: number;
  /**
   * The tick rate, from GameDesc.tick.
   *
   * @unit hz (ticks or cycles per second)
   * @readonly written by the engine
   */
  readonly rate: number;
  /**
   * Global slow motion, set with og.time.setScale: 1 is normal, 0.5 half speed, 0 stopped. Physics, animation, flipbooks and tweens advance by scaledDt; the tick itself, the UI, input and the probe keep their pace.
   *
   * @default 1
   * @range 0..10
   * @readonly written by the engine
   */
  readonly scale: number;
  /**
   * Ticks of hit stop left (og.time.hitStop): while above 0, scaledDt is 0.
   *
   * @unit ticks (fixed simulation steps)
   * @readonly written by the engine
   */
  readonly hitStop: number;
  /**
   * This tick's scaled length: 0 during a hit stop, else dt * scale. Move gameplay by it so slow motion and hit stop reach game code too.
   *
   * @unit s (seconds of simulation time)
   * @readonly written by the engine
   */
  readonly scaledDt: number;
  /**
   * Scaled seconds since load: the sum of every finished tick's scaledDt.
   *
   * @unit s (seconds of simulation time)
   * @readonly written by the engine
   */
  readonly scaledElapsed: number;
}



// ── rng ─────────────────────────────────────────────────────────
// Deterministic randomness. Every random draw comes from an Rng stream; the sequence for a seed is identical on every host and in the Web scaffold's rng.ts.
// Inside a system: forking needs og.Rng among the reads (a fork is derived from the seed and the label, and leaves the main stream where it is); drawing from og.Rng itself (next, range, integer, chance) moves its state, so it needs og.Rng among the writes. A stream a fork returned is the system's own to draw from.

/**
 * One generator stream, forked from the Rng resource for a subsystem that must not perturb the main sequence.
 */
export interface RngStream {
  /** The stream's derived seed. */
  readonly seed: number;
  /** Uniform in [0, 1). */
  next(): number;
  /**
   * Uniform in [low, high).
   *
   * @param low The inclusive lower bound.
   * @param high The exclusive upper bound.
   */
  range(low: number, high: number): number;
  /**
   * A uniform integer in [low, high].
   *
   * @param low The inclusive lower bound.
   * @param high The inclusive upper bound.
   */
  integer(low: number, high: number): number;
  /**
   * True with probability p.
   *
   * @param p The probability, between 0 and 1.
   */
  chance(p: number): boolean;
  /**
   * A fresh stream on a seed derived from this stream's seed and a label.
   *
   * @param label The subsystem's name; the same label always gives the same stream.
   */
  fork(label: string): RngStream;
}

/**
 * The world's main random stream, seeded from GameDesc.seed (or the replay seed). Its state is part of every snapshot.
 *
 * The `Rng` resource (token `og.Rng`).
 */
export interface Rng {
  /**
   * The seed the stream started from.
   *
   * @readonly written by the engine
   */
  readonly seed: number;
  /**
   * The generator state; snapshot and restore carry it.
   *
   * @readonly written by the engine
   */
  readonly state: number;
  /** Uniform in [0, 1). */
  next(): number;
  /**
   * Uniform in [low, high).
   *
   * @param low The inclusive lower bound.
   * @param high The exclusive upper bound.
   */
  range(low: number, high: number): number;
  /**
   * A uniform integer in [low, high].
   *
   * @param low The inclusive lower bound.
   * @param high The inclusive upper bound.
   */
  integer(low: number, high: number): number;
  /**
   * True with probability p.
   *
   * @param p The probability, between 0 and 1.
   */
  chance(p: number): boolean;
  /**
   * A stream for a subsystem, derived from the seed and a label, so drawing from it leaves the main sequence alone. The same label gives the same stream every time, on every host, wherever the main stream is; og.Rng among a system's reads is enough.
   *
   * @param label The subsystem's name, such as `spawner`.
   */
  fork(label: string): RngStream;
}

// ── input ───────────────────────────────────────────────────────
// Input as the game sees it: the Input resource for polling, events for edges.
// The shell collects pointer, keyboard and wheel input on the main thread; the probe and the command channel inject through the same path.
// Several pointers at once (multi-touch) are og.Pointers; the first four gamepads, in the W3C standard mapping, are og.Gamepad. Both are world state, as og.Input is, so they replay.

/** The most pointers og.Pointers tracks at once: ids 0 (the primary pointer) to 4. */
export declare const MAX_POINTERS: 5;

/** The most gamepads og.Gamepad reads: pads 0 to 3. */
export declare const MAX_GAMEPADS: 4;

/**
 * A gamepad button in the W3C standard mapping, in its index order (a is button 0).
 * - `"a"`: The bottom face button (A, Cross).
 * - `"b"`: The right face button (B, Circle).
 * - `"x"`: The left face button (X, Square).
 * - `"y"`: The top face button (Y, Triangle).
 * - `"leftBumper"`: The left shoulder button.
 * - `"rightBumper"`: The right shoulder button.
 * - `"leftTrigger"`: The left trigger, as a button (its analog value is the leftTrigger axis).
 * - `"rightTrigger"`: The right trigger, as a button.
 * - `"back"`: Back, Select or View.
 * - `"start"`: Start, Options or Menu.
 * - `"leftStick"`: The left stick pressed in.
 * - `"rightStick"`: The right stick pressed in.
 * - `"up"`: The d-pad's up.
 * - `"down"`: The d-pad's down.
 * - `"left"`: The d-pad's left.
 * - `"right"`: The d-pad's right.
 * - `"home"`: The home or guide button.
 */
export type GamepadButton = "a" | "b" | "x" | "y" | "leftBumper" | "rightBumper" | "leftTrigger" | "rightTrigger" | "back" | "start" | "leftStick" | "rightStick" | "up" | "down" | "left" | "right" | "home";

/**
 * An analog input of a gamepad.
 * - `"leftX"`: The left stick's x, -1 (left) to 1 (right).
 * - `"leftY"`: The left stick's y, -1 (up) to 1 (down), as the Gamepad API reports it.
 * - `"rightX"`: The right stick's x.
 * - `"rightY"`: The right stick's y, down positive.
 * - `"leftTrigger"`: The left trigger, 0 to 1.
 * - `"rightTrigger"`: The right trigger, 0 to 1.
 */
export type GamepadAxis = "leftX" | "leftY" | "rightX" | "rightY" | "leftTrigger" | "rightTrigger";

/**
 * What an injected key event does.
 * - `"press"`: Down, then up on the next tick.
 * - `"down"`: Held down until an up.
 * - `"up"`: Released.
 */
export type KeyAction = "press" | "down" | "up";

/**
 * What an injected pointer event does.
 * - `"click"`: Down, then up on the next tick, at the point.
 * - `"move"`: A move to the point.
 * - `"down"`: A press at the point.
 * - `"up"`: A release at the point.
 */
export type PointerAction = "click" | "move" | "down" | "up";

/**
 * A physical pointer button; `none` is a hover move.
 * - `"none"`: No button.
 * - `"left"`: The primary button, or a touch.
 * - `"middle"`: The middle button.
 * - `"right"`: The secondary button.
 */
export type PointerButton = "none" | "left" | "middle" | "right";

/** Modifier keys held during a key event. */
export interface Modifiers {
  /** Alt or Option. */
  readonly alt: boolean;
  /** Control. */
  readonly control: boolean;
  /** Command on macOS, the Windows key on Windows. */
  readonly platform: boolean;
  /** Shift. */
  readonly shift: boolean;
}

/**
 * A key event. `key` names the physical key as KeyboardEvent.code does: ArrowRight, Space, KeyA.
 */
export interface KeyInput {
  /** The physical key code. */
  readonly key: string;
  /** Press, down or up. */
  readonly action: KeyAction;
  /** Held modifiers; omitted means none. */
  readonly modifiers?: Modifiers;
  /**
   * When it happened, in ticks from the start of the tick that reads it (-1 to 1; omitted is 0): the KeyDown, KeyUp, PointerDown or PointerUp it raises carry it. An engine extension of protocol 5.
   *
   * @range -1..1
   */
  readonly at?: number;
}

/** A pointer event at a point in CSS pixels of the play surface. */
export interface PointerInput {
  /** Click, move, down or up. */
  readonly action: PointerAction;
  /**
   * Horizontal position.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly x: number;
  /**
   * Vertical position.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly y: number;
  /** The button; `none` for a hover move. */
  readonly button: PointerButton;
  /**
   * Which pointer: 0 the primary one (the mouse, the first touch; what protocol 5 means), 1 to 4 further touches held at the same time (an engine extension). Omitted is 0.
   *
   * @range ..4
   */
  readonly id?: number;
  /**
   * When it happened, in ticks from the start of the tick that reads it (-1 to 1; omitted is 0): the KeyDown, KeyUp, PointerDown or PointerUp it raises carry it. An engine extension of protocol 5.
   *
   * @range -1..1
   */
  readonly at?: number;
}

/**
 * A gamepad's whole state (an engine extension of protocol 5): the shell sends one whenever a pad changes, and a playtest can send its own.
 */
export interface GamepadInput {
  /**
   * Which pad, 0 to 3; omitted is 0.
   *
   * @range ..3
   */
  readonly pad?: number;
  /** Whether the pad is connected (omitted is true); a disconnected pad holds nothing. */
  readonly connected?: boolean;
  /** The buttons held (omitted is none). */
  readonly buttons?: readonly GamepadButton[];
  /** Left stick x, y, right stick x, y; each -1 to 1, y down. Missing values are 0. */
  readonly axes?: readonly number[];
  /** The left and right analog triggers, 0 to 1. Missing values are 0. */
  readonly triggers?: readonly number[];
}

/** A wheel or trackpad scroll at a point. */
export interface WheelInput {
  /**
   * Horizontal position.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly x: number;
  /**
   * Vertical position.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly y: number;
  /**
   * Horizontal scroll amount.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly deltaX: number;
  /**
   * Vertical scroll amount.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly deltaY: number;
}

/**
 * Typed or composed text (an engine extension of protocol 5): what the page's `beforeinput` inserted (a key's character, a paste, an input method's commit), or with `composing` an input method's composition so far. Raises TextInput events; the UI's focused text field takes them.
 */
export interface TextEntry {
  /** The text. */
  readonly text: string;
  /**
   * An input method's composition, not yet committed: it replaces the composition before it, and empty ends it. Omitted is false: committed text.
   */
  readonly composing?: boolean;
}

/**
 * What an assistive technology asks of a UI control.
 * - `"focus"`: Move the UI's focus to it (with the focus ring).
 * - `"activate"`: Press it, as Enter on it would (a text control takes the focus).
 * - `"increment"`: Step an adjustable control (a slider, a stepper) up.
 * - `"decrement"`: Step an adjustable control down.
 */
export type A11yActionKind = "focus" | "activate" | "increment" | "decrement";

/**
 * A screen reader's request on a UI control (an engine extension of protocol 5): the web shell's mirror of og.ui.a11y() sends it when a reader focuses, activates or adjusts a mirrored node. Raises an A11yAction event; the UI acts on it at the start of the next frame, against the last frame's controls (a path naming no focusable control of the topmost dialog is ignored; increment and decrement also take a stepper's spinbutton, whose buttons are the focusable controls).
 */
export interface A11yRequest {
  /** What is asked. */
  readonly action: A11yActionKind;
  /** The control's path, as og.ui.a11y() names it. */
  readonly path: string;
}

/**
 * One input event in the game's own vocabulary, as the probe's input() and playtest scripts carry it.
 */
export type InjectedInput =
  /** A key event. */
  | ({ readonly kind: "key" } & KeyInput)
  /** A pointer event. */
  | ({ readonly kind: "pointer" } & PointerInput)
  /** A wheel event. */
  | ({ readonly kind: "wheel" } & WheelInput)
  /** A gamepad's state. */
  | ({ readonly kind: "gamepad" } & GamepadInput)
  /** Typed or composed text. */
  | ({ readonly kind: "text" } & TextEntry)
  /** A screen reader's request on a UI control. */
  | ({ readonly kind: "a11y" } & A11yRequest);

/**
 * The input state at the start of this tick (og-input's og.Input, version 2). Edges (pressed, released) last exactly one tick; read the state through the methods.
 * Key sets are 128 bits over og-input's KEY_CODES (KeyboardEvent.code names): code i is bit i % 32 of word i / 32. Button masks: left 1, middle 2, right 4.
 *
 * The `Input` resource (token `og.Input`).
 */
export interface Input {
  /**
   * The pointer position.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly pointer: Vector2;
  /**
   * Whether a pointer event has placed the pointer over the play surface.
   *
   * @readonly written by the engine
   */
  readonly pointerInside: boolean;
  /**
   * Scroll accumulated during this tick.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly wheel: Vector2;
  /**
   * Held pointer buttons, as a mask.
   *
   * @readonly written by the engine
   */
  readonly buttons: number;
  /**
   * Buttons that went down this tick.
   *
   * @readonly written by the engine
   */
  readonly buttonsPressed: number;
  /**
   * Buttons that went up this tick.
   *
   * @readonly written by the engine
   */
  readonly buttonsReleased: number;
  /**
   * Buttons a click releases at the end of the next tick.
   *
   * @readonly written by the engine
   */
  readonly buttonsReleaseNext: number;
  /**
   * Held keys, word 0: codes 0 to 31 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly keys0: number;
  /**
   * Held keys, word 1: codes 32 to 63 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly keys1: number;
  /**
   * Held keys, word 2: codes 64 to 95 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly keys2: number;
  /**
   * Held keys, word 3: codes 96 to 127 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly keys3: number;
  /**
   * Keys that went down this tick, word 0: codes 0 to 31 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly pressed0: number;
  /**
   * Keys that went down this tick, word 1: codes 32 to 63 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly pressed1: number;
  /**
   * Keys that went down this tick, word 2: codes 64 to 95 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly pressed2: number;
  /**
   * Keys that went down this tick, word 3: codes 96 to 127 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly pressed3: number;
  /**
   * Keys that went up this tick, word 0: codes 0 to 31 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly released0: number;
  /**
   * Keys that went up this tick, word 1: codes 32 to 63 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly released1: number;
  /**
   * Keys that went up this tick, word 2: codes 64 to 95 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly released2: number;
  /**
   * Keys that went up this tick, word 3: codes 96 to 127 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly released3: number;
  /**
   * Keys a press releases at the end of the next tick, word 0: codes 0 to 31 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly releaseNext0: number;
  /**
   * Keys a press releases at the end of the next tick, word 1: codes 32 to 63 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly releaseNext1: number;
  /**
   * Keys a press releases at the end of the next tick, word 2: codes 64 to 95 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly releaseNext2: number;
  /**
   * Keys a press releases at the end of the next tick, word 3: codes 96 to 127 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly releaseNext3: number;
  /**
   * Whether a key is held.
   *
   * @param key A KeyboardEvent.code name, such as KeyW or Space.
   */
  down(key: string): boolean;
  /**
   * Whether a key went down this tick.
   *
   * @param key A KeyboardEvent.code name.
   */
  pressed(key: string): boolean;
  /**
   * Whether a key went up this tick.
   *
   * @param key A KeyboardEvent.code name.
   */
  released(key: string): boolean;
  /**
   * -1, 0 or 1 from two keys, such as `axis("KeyA", "KeyD")`.
   *
   * @param negative The key for -1.
   * @param positive The key for +1.
   */
  axis(negative: string, positive: string): number;
  /**
   * A 2D axis from four keys, such as `axis2("KeyA", "KeyD", "KeyS", "KeyW")`: each component -1, 0 or 1.
   *
   * @param left The key for x = -1.
   * @param right The key for x = +1.
   * @param down The key for y = -1.
   * @param up The key for y = +1.
   */
  axis2(left: string, right: string, down: string, up: string): Vector2;
  /**
   * Whether a pointer button is held.
   *
   * @param button The button.
   */
  buttonDown(button: PointerButton): boolean;
  /**
   * Whether a pointer button went down this tick.
   *
   * @param button The button.
   */
  buttonPressed(button: PointerButton): boolean;
  /**
   * Whether a pointer button went up this tick.
   *
   * @param button The button.
   */
  buttonReleased(button: PointerButton): boolean;
}

/**
 * Every pointer on the play surface at once, at the start of this tick (og-input's og.Pointers): a touch stick and a touch button can be held together. Pointer 0 is og.Input's pointer; ids 1 to 4 are further touches, in the order they landed (the lowest free id).
 * Edges last exactly one tick, as og.Input's do. Masks are bit `id`; read them through the methods.
 *
 * The `Pointers` resource (token `og.Pointers`).
 */
export interface Pointers {
  /**
   * Pointers held down, as a mask.
   *
   * @readonly written by the engine
   */
  readonly heldMask: number;
  /**
   * Pointers that went down this tick.
   *
   * @readonly written by the engine
   */
  readonly pressedMask: number;
  /**
   * Pointers that went up this tick.
   *
   * @readonly written by the engine
   */
  readonly releasedMask: number;
  /**
   * Pointers a click releases at the end of the next tick.
   *
   * @readonly written by the engine
   */
  readonly releaseNextMask: number;
  /**
   * Where pointer 0 is.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly position0: Vector2;
  /**
   * Where pointer 1 is.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly position1: Vector2;
  /**
   * Where pointer 2 is.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly position2: Vector2;
  /**
   * Where pointer 3 is.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly position3: Vector2;
  /**
   * Where pointer 4 is.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly position4: Vector2;
  /**
   * Where pointer 0 went down.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly start0: Vector2;
  /**
   * Where pointer 1 went down.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly start1: Vector2;
  /**
   * Where pointer 2 went down.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly start2: Vector2;
  /**
   * Where pointer 3 went down.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly start3: Vector2;
  /**
   * Where pointer 4 went down.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly start4: Vector2;
  /**
   * Whether a pointer is held.
   *
   * @param id The pointer, 0 to 4.
   */
  down(id: number): boolean;
  /**
   * Whether a pointer went down this tick.
   *
   * @param id The pointer, 0 to 4.
   */
  pressed(id: number): boolean;
  /**
   * Whether a pointer went up this tick.
   *
   * @param id The pointer, 0 to 4.
   */
  released(id: number): boolean;
  /**
   * Where a pointer is, in CSS pixels.
   *
   * @param id The pointer, 0 to 4.
   */
  position(id: number): Vector2;
  /**
   * Where a pointer went down, in CSS pixels: a touch control claims the pointers that started on it.
   *
   * @param id The pointer, 0 to 4.
   */
  start(id: number): Vector2;
  /** How many pointers are held. */
  count(): number;
}

/**
 * The first four gamepads in the W3C standard mapping, at the start of this tick (og-input's og.Gamepad). On the web the shell polls the pads each display frame and sends their state through the input ring, so a pad is world state and replays; it also maps pads to keys for games that only read keys.
 * Button masks are bit `GamepadButton` index per pad; edges last exactly one tick, and a press and a release between two ticks leave both edges set. Read them through the methods.
 *
 * The `Gamepad` resource (token `og.Gamepad`).
 */
export interface Gamepad {
  /**
   * Connected pads: bit `pad`.
   *
   * @readonly written by the engine
   */
  readonly connectedMask: number;
  /**
   * Held buttons of pad 0.
   *
   * @readonly written by the engine
   */
  readonly buttons0: number;
  /**
   * Held buttons of pad 1.
   *
   * @readonly written by the engine
   */
  readonly buttons1: number;
  /**
   * Held buttons of pad 2.
   *
   * @readonly written by the engine
   */
  readonly buttons2: number;
  /**
   * Held buttons of pad 3.
   *
   * @readonly written by the engine
   */
  readonly buttons3: number;
  /**
   * Buttons of pad 0 that went down this tick.
   *
   * @readonly written by the engine
   */
  readonly pressed0: number;
  /**
   * Buttons of pad 1 that went down this tick.
   *
   * @readonly written by the engine
   */
  readonly pressed1: number;
  /**
   * Buttons of pad 2 that went down this tick.
   *
   * @readonly written by the engine
   */
  readonly pressed2: number;
  /**
   * Buttons of pad 3 that went down this tick.
   *
   * @readonly written by the engine
   */
  readonly pressed3: number;
  /**
   * Buttons of pad 0 that went up this tick.
   *
   * @readonly written by the engine
   */
  readonly released0: number;
  /**
   * Buttons of pad 1 that went up this tick.
   *
   * @readonly written by the engine
   */
  readonly released1: number;
  /**
   * Buttons of pad 2 that went up this tick.
   *
   * @readonly written by the engine
   */
  readonly released2: number;
  /**
   * Buttons of pad 3 that went up this tick.
   *
   * @readonly written by the engine
   */
  readonly released3: number;
  /**
   * The left stick of pad 0, -1 to 1, y down.
   *
   * @readonly written by the engine
   */
  readonly leftStick0: Vector2;
  /**
   * The left stick of pad 1, -1 to 1, y down.
   *
   * @readonly written by the engine
   */
  readonly leftStick1: Vector2;
  /**
   * The left stick of pad 2, -1 to 1, y down.
   *
   * @readonly written by the engine
   */
  readonly leftStick2: Vector2;
  /**
   * The left stick of pad 3, -1 to 1, y down.
   *
   * @readonly written by the engine
   */
  readonly leftStick3: Vector2;
  /**
   * The right stick of pad 0, -1 to 1, y down.
   *
   * @readonly written by the engine
   */
  readonly rightStick0: Vector2;
  /**
   * The right stick of pad 1, -1 to 1, y down.
   *
   * @readonly written by the engine
   */
  readonly rightStick1: Vector2;
  /**
   * The right stick of pad 2, -1 to 1, y down.
   *
   * @readonly written by the engine
   */
  readonly rightStick2: Vector2;
  /**
   * The right stick of pad 3, -1 to 1, y down.
   *
   * @readonly written by the engine
   */
  readonly rightStick3: Vector2;
  /**
   * The left (x) and right (y) triggers of pad 0, 0 to 1.
   *
   * @readonly written by the engine
   */
  readonly triggers0: Vector2;
  /**
   * The left (x) and right (y) triggers of pad 1, 0 to 1.
   *
   * @readonly written by the engine
   */
  readonly triggers1: Vector2;
  /**
   * The left (x) and right (y) triggers of pad 2, 0 to 1.
   *
   * @readonly written by the engine
   */
  readonly triggers2: Vector2;
  /**
   * The left (x) and right (y) triggers of pad 3, 0 to 1.
   *
   * @readonly written by the engine
   */
  readonly triggers3: Vector2;
  /**
   * Whether a pad is connected.
   *
   * @param pad The pad, 0 to 3; omitted is 0.
   */
  connected(pad?: number): boolean;
  /**
   * Whether a button is held.
   *
   * @param button The button.
   * @param pad The pad, 0 to 3; omitted is 0.
   */
  down(button: GamepadButton, pad?: number): boolean;
  /**
   * Whether a button went down this tick.
   *
   * @param button The button.
   * @param pad The pad, 0 to 3; omitted is 0.
   */
  pressed(button: GamepadButton, pad?: number): boolean;
  /**
   * Whether a button went up this tick.
   *
   * @param button The button.
   * @param pad The pad, 0 to 3; omitted is 0.
   */
  released(button: GamepadButton, pad?: number): boolean;
  /**
   * An analog input: a stick's x or y (-1 to 1, y down) or a trigger (0 to 1). Dead zones are the game's.
   *
   * @param axis Which input.
   * @param pad The pad, 0 to 3; omitted is 0.
   */
  axis(axis: GamepadAxis, pad?: number): number;
}

/**
 * A key went down (a down for a held key is a repeat and sets no edge). Input events are sent between ticks and read by the next tick's systems.
 *
 * The payload of the `KeyDown` event (token `og.KeyDown`).
 */
export interface KeyDown {
  /** The physical key code. */
  readonly key: string;
  /** Whether this is an auto-repeat. */
  readonly repeat: boolean;
  /**
   * When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. The shell stamps it from the browser event's timeStamp, the command channel from the step's time; it is part of the event, so a replay carries it.
   *
   * @range -1..1
   */
  readonly at: number;
}

/**
 * Text typed or composed (an engine extension): the shell sends it from the page's `beforeinput` and composition events, the probe's `type:`, `compose:` and `commit:` from scripts. A text longer than 255 bytes arrives as several events in order. Key events still arrive for the keys; a text field types from these once a world has had one.
 *
 * The payload of the `TextInput` event (token `og.TextInput`).
 */
export interface TextInput {
  /** The text (whole characters). */
  readonly text: string;
  /**
   * An input method's composition so far, not committed: it replaces the composition before it (empty ends it). False: committed text to insert.
   */
  readonly composing: boolean;
  /** Continues the composition of the event before it (a composition longer than one event). */
  readonly more: boolean;
}

/**
 * A key went up.
 *
 * The payload of the `KeyUp` event (token `og.KeyUp`).
 */
export interface KeyUp {
  /** The physical key code. */
  readonly key: string;
  /**
   * When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. The shell stamps it from the browser event's timeStamp, the command channel from the step's time; it is part of the event, so a replay carries it.
   *
   * @range -1..1
   */
  readonly at: number;
}

/**
 * A pointer button went down (or a touch landed).
 *
 * The payload of the `PointerDown` event (token `og.PointerDown`).
 */
export interface PointerDown {
  /** Horizontal position in CSS pixels. */
  readonly x: number;
  /** Vertical position in CSS pixels. */
  readonly y: number;
  /** The button. */
  readonly button: PointerButton;
  /** Which pointer: 0 the primary one, 1 to 4 further touches (og.Pointers). */
  readonly id: number;
  /**
   * When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. The shell stamps it from the browser event's timeStamp, the command channel from the step's time; it is part of the event, so a replay carries it.
   *
   * @range -1..1
   */
  readonly at: number;
}

/**
 * A pointer button went up (or a touch lifted).
 *
 * The payload of the `PointerUp` event (token `og.PointerUp`).
 */
export interface PointerUp {
  /** Horizontal position in CSS pixels. */
  readonly x: number;
  /** Vertical position in CSS pixels. */
  readonly y: number;
  /** The button. */
  readonly button: PointerButton;
  /** Which pointer: 0 the primary one, 1 to 4 further touches. */
  readonly id: number;
  /**
   * When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. The shell stamps it from the browser event's timeStamp, the command channel from the step's time; it is part of the event, so a replay carries it.
   *
   * @range -1..1
   */
  readonly at: number;
}

/**
 * A pointer moved.
 *
 * The payload of the `PointerMove` event (token `og.PointerMove`).
 */
export interface PointerMove {
  /** Horizontal position in CSS pixels. */
  readonly x: number;
  /** Vertical position in CSS pixels. */
  readonly y: number;
  /** Which pointer: 0 the primary one, 1 to 4 further touches. */
  readonly id: number;
}

/**
 * A wheel or trackpad scroll.
 *
 * The payload of the `Wheel` event (token `og.Wheel`).
 */
export interface Wheel {
  /** Horizontal scroll in CSS pixels. */
  readonly deltaX: number;
  /** Vertical scroll in CSS pixels. */
  readonly deltaY: number;
}

/**
 * A screen reader's request on a UI control (an engine extension): the web shell sends it from its mirror of og.ui.a11y(); the UI takes it at the start of the next frame. A path longer than 255 bytes is refused.
 *
 * The payload of the `A11yAction` event (token `og.A11yAction`).
 */
export interface A11yAction {
  /** What is asked. */
  readonly action: A11yActionKind;
  /** The control's path, as og.ui.a11y() names it. */
  readonly path: string;
}

// ── camera ──────────────────────────────────────────────────────
// The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names.
// Camera controllers are components (OrbitCamera, FollowCamera, Follow2D) on the entity the camera orbits or follows; the systems that drive them ship in TypeScript as og.systems.cameras.

/**
 * The camera the frame is rendered from. Rendering reads it after the tick; it never feeds back into the simulation.
 *
 * The `Camera` resource (token `og.Camera`).
 */
export interface Camera {
  /**
   * Where the camera is.
   *
   * @unit m (metres)
   */
  position: Vector3;
  /** Its rotation; the camera looks down its local -Z. */
  quaternion: Quaternion;
  /**
   * The vertical field of view of the perspective projection.
   *
   * @default 50
   * @range 1..179
   * @unit deg (degrees)
   * @tunable
   */
  fov: number;
  /**
   * The near clipping plane.
   *
   * @default 0.1
   * @range 0.001..
   * @unit m (metres)
   */
  near: number;
  /**
   * The far clipping plane.
   *
   * @default 2000
   * @range 0.01..
   * @unit m (metres)
   */
  far: number;
  /**
   * The zoom factor, as three.js Camera.zoom.
   *
   * @default 1
   * @range 0.01..100
   * @tunable
   */
  zoom: number;
  /** Use an orthographic projection (the 2D profile) instead of a perspective one. */
  orthographic: boolean;
  /**
   * The visible world height of the orthographic projection.
   *
   * @default 10
   * @range 0.01..
   * @unit m (metres)
   */
  height: number;
  /**
   * The clear colour, as three.js Scene.background. The default is the brand ink #14161b in linear RGB.
   *
   * @default { r: 0.00699, g: 0.00803, b: 0.01096 }
   */
  background: Color;
  /**
   * Exposure before tone mapping, as three.js toneMappingExposure.
   *
   * @default 1
   * @range 0..16
   * @tunable
   */
  exposure: number;
  /**
   * Screen shake, 0 to 1, added by og.camera.shake and decayed by traumaDecay every tick (unscaled, so a hit stop still shakes). The offset and roll grow with trauma squared and follow smooth noise of the tick: visual only, a replay shakes the same.
   *
   * @range 0..1
   * @tunable
   */
  trauma: number;
  /**
   * How much trauma drains per second.
   *
   * @default 1.5
   * @range 0..
   * @unit perSecond (a rate per second: the fraction of a value that decays in one second)
   * @tunable
   */
  traumaDecay: number;
  /**
   * The largest offset at trauma 1, as a fraction of the view height.
   *
   * @default 0.04
   * @range 0..
   * @tunable
   */
  shakeOffset: number;
  /**
   * The largest roll at trauma 1.
   *
   * @default 2
   * @range 0..
   * @unit deg (degrees)
   * @tunable
   */
  shakeRoll: number;
  /**
   * How fast the shake moves.
   *
   * @default 15
   * @range 0..
   * @unit hz (ticks or cycles per second)
   */
  shakeFrequency: number;
  /**
   * A zoom punch in progress (og.camera.punch): the view is narrowed by this fraction, decaying by punchDecay. Visual only.
   *
   * @range -0.9..4
   */
  punch: number;
  /**
   * How fast a punch springs back (exponential, per second).
   *
   * @default 8
   * @range 0..
   * @unit perSecond (a rate per second: the fraction of a value that decays in one second)
   * @tunable
   */
  punchDecay: number;
  /**
   * A kick in progress (og.camera.kick): the view is offset by this, decaying by kickDecay. Visual only.
   *
   * @unit m (metres)
   */
  kick: Vector3;
  /**
   * How fast a kick returns (exponential, per second).
   *
   * @default 12
   * @range 0..
   * @unit perSecond (a rate per second: the fraction of a value that decays in one second)
   * @tunable
   */
  kickDecay: number;
  /**
   * The followed entity is in an og.CameraZone: the view keeps inside zoneMin to zoneMax. Written by the og.camera system.
   */
  zoneActive: boolean;
  /**
   * The zone the view keeps inside: its bottom-left corner.
   *
   * @unit m (metres)
   */
  zoneMin: Vector2;
  /**
   * Its top-right corner.
   *
   * @unit m (metres)
   */
  zoneMax: Vector2;
  /**
   * A room transition is in progress from the zone zoneFromMin to zoneFromMax: the view scrolls from where that zone kept it to where the new one does.
   */
  zoneFromActive: boolean;
  /**
   * The zone being left: its bottom-left corner.
   *
   * @unit m (metres)
   */
  zoneFromMin: Vector2;
  /**
   * Its top-right corner.
   *
   * @unit m (metres)
   */
  zoneFromMax: Vector2;
  /**
   * How far the transition has come, 0 to 1 (eased; 1 is settled).
   *
   * @default 1
   * @range 0..1
   */
  zoneScroll: number;
  /**
   * How long the transition takes: the entered zone's scrollTime.
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  zoneScrollTime: number;
  /**
   * Three.js layers, as Camera.layers.mask: the frame draws the meshes, models, world sprites, particle systems, lines and points whose layers share a bit with it. 1 (layer 0) by default, as every object's.
   *
   * @default 1
   */
  layers: number;
  /**
   * Rotate the camera to look at a point, as three.js Camera.lookAt.
   *
   * @param target The point to look at.
   */
  lookAt(target: Vector3): void;
  /**
   * Where a world point lands on the play surface.
   *
   * @param point A world point.
   * @returns CSS pixels, or null behind the camera.
   */
  worldToScreen(point: Vector3): Vector2 | null;
  /**
   * The ray through a point of the play surface, for picking.
   *
   * @param x Horizontal position in CSS pixels.
   * @param y Vertical position in CSS pixels.
   */
  screenToRay(x: number, y: number): Ray;
}

/**
 * Orbits og.Camera around the entity carrying it, as three.js OrbitControls: a left-button drag on the play surface turns it, the wheel zooms, autoRotate turns it on its own.
 * Camera controllers are game-level systems the engine ships in TypeScript (og.systems.cameras); list them in GameDesc.systems: `systems: [...og.systems.cameras, ...yours]`. The first entity carrying one, in query order, drives the camera.
 * The camera sits `distance` from the orbited point (the entity's position plus `target`) in the direction given by `azimuth` about +Y and `polar` from +Y, and looks at the point. Every angle is computed with deterministic arithmetic, so a replay moves the camera identically on every host.
 *
 * The value of the `OrbitCamera` component (token `og.OrbitCamera`).
 */
export interface OrbitCamera {
  /**
   * The orbited point, relative to the entity's position.
   *
   * @unit m (metres)
   */
  target: Vector3;
  /**
   * How far the camera is from the orbited point, as three.js OrbitControls' distance.
   *
   * @default 10
   * @range 0.01..
   * @unit m (metres)
   * @tunable
   */
  distance: number;
  /**
   * The closest the wheel zooms in, as three.js OrbitControls.minDistance.
   *
   * @default 1
   * @range 0.01..
   * @unit m (metres)
   */
  minDistance: number;
  /**
   * The farthest the wheel zooms out, as three.js OrbitControls.maxDistance.
   *
   * @default 100
   * @range 0.01..
   * @unit m (metres)
   */
  maxDistance: number;
  /**
   * The horizontal angle about +Y; 0 puts the camera on the +Z side of the point.
   *
   * @unit rad (radians)
   * @tunable
   */
  azimuth: number;
  /**
   * The angle from +Y: near 0 looks straight down, half pi is level with the point.
   *
   * @default 1.1
   * @range 0..3.1416
   * @unit rad (radians)
   * @tunable
   */
  polar: number;
  /**
   * The smallest polar angle a drag reaches, as three.js OrbitControls.minPolarAngle.
   *
   * @default 0.05
   * @range 0..3.1416
   * @unit rad (radians)
   */
  minPolarAngle: number;
  /**
   * The largest polar angle a drag reaches, as three.js OrbitControls.maxPolarAngle.
   *
   * @default 3.0916
   * @range 0..3.1416
   * @unit rad (radians)
   */
  maxPolarAngle: number;
  /**
   * Drag sensitivity, as three.js OrbitControls.rotateSpeed: at 1, dragging the play surface's height turns a full circle.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  rotateSpeed: number;
  /**
   * Wheel sensitivity, as three.js OrbitControls.zoomSpeed: at 1, 100 CSS pixels of wheel change the distance by a tenth.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  zoomSpeed: number;
  /**
   * Turn about the point on its own while not dragged, as three.js OrbitControls.autoRotate.
   */
  autoRotate: boolean;
  /**
   * How fast autoRotate turns.
   *
   * @default 0.5
   * @unit radps (radians per second)
   * @tunable
   */
  autoRotateSpeed: number;
  /**
   * Read the pointer and the wheel; false keeps only autoRotate and the fields' own values.
   *
   * @default true
   */
  enabled: boolean;
  /**
   * Controller state: a left-button drag that started on the play surface is in progress.
   *
   * @readonly written by the engine
   */
  readonly dragging: boolean;
  /**
   * Controller state: where the pointer was on the previous tick of a drag.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly pointer: Vector2;
}

/**
 * Chases the entity carrying it with og.Camera: a third-person 3D view that eases towards a point offset from the entity and looks at it. Driven by og.systems.cameras; the first entity carrying one, in query order, drives the camera.
 *
 * The value of the `FollowCamera` component (token `og.FollowCamera`).
 */
export interface FollowCamera {
  /**
   * Where the camera sits relative to the entity: in the entity's own axes with followRotation, else in world axes.
   *
   * @default { x: 0, y: 2, z: 6 }
   * @unit m (metres)
   */
  offset: Vector3;
  /**
   * The point the camera looks at, relative to the entity's position.
   *
   * @default { x: 0, y: 1, z: 0 }
   * @unit m (metres)
   */
  lookOffset: Vector3;
  /** Turn the offset with the entity's rotation, so the camera stays behind it. */
  followRotation: boolean;
  /**
   * Roughly how long the camera takes to catch up with a moved entity; 0 snaps it there every tick.
   *
   * @default 0.25
   * @range 0..
   * @unit s (seconds of simulation time)
   * @tunable
   */
  smoothTime: number;
}

/**
 * Keeps og.Camera on the entity carrying it in the xy plane, for 2D games: the camera moves only when the entity leaves the dead zone, eases by smoothTime (smoothTimeY for y when set) and, with clamp, stays inside the bounds; lookAhead leads it in the direction the entity moves. The camera keeps its z, rotation and projection. Driven by og.systems.cameras; og.CameraZone entities then keep the view inside the zone the entity is in.
 *
 * The value of the `Follow2D` component (token `og.Follow2D`).
 */
export interface Follow2D {
  /**
   * Where the camera centre sits relative to the entity.
   *
   * @unit m (metres)
   */
  offset: Vector2;
  /**
   * Half the width and height of the box around the camera centre inside which the entity moves without moving the camera.
   *
   * @unit m (metres)
   */
  deadZone: Vector2;
  /**
   * Roughly how long the camera takes to catch up; 0 snaps it every tick.
   *
   * @default 0.15
   * @range 0..
   * @unit s (seconds of simulation time)
   * @tunable
   */
  smoothTime: number;
  /**
   * The same for y alone (a platformer follows jumps more lazily than runs); below 0 uses smoothTime.
   *
   * @default -1
   * @unit s (seconds of simulation time)
   * @tunable
   */
  smoothTimeY: number;
  /**
   * Lead the entity by its velocity times this many seconds (0: no look-ahead).
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   * @tunable
   */
  lookAhead: number;
  /**
   * The largest lead along x and y.
   *
   * @default { x: 3, y: 1.5 }
   * @range 0..
   * @unit m (metres)
   */
  lookAheadMax: Vector2;
  /**
   * Roughly how long the lead takes to turn when the entity does.
   *
   * @default 0.4
   * @range 0..
   * @unit s (seconds of simulation time)
   * @tunable
   */
  lookAheadSmoothing: number;
  /**
   * Controller state: the current lead.
   *
   * @unit m (metres)
   * @readonly written by the engine
   */
  readonly lead: Vector2;
  /**
   * Controller state: the entity's position on the previous tick.
   *
   * @unit m (metres)
   * @readonly written by the engine
   */
  readonly last: Vector2;
  /** Keep the camera centre between boundsMin and boundsMax. */
  clamp: boolean;
  /**
   * The smallest x and y the camera centre takes with clamp.
   *
   * @unit m (metres)
   */
  boundsMin: Vector2;
  /**
   * The largest x and y the camera centre takes with clamp.
   *
   * @unit m (metres)
   */
  boundsMax: Vector2;
}

/**
 * A rectangle of the xy plane (min to max, world units, not moved by the entity's Transform) that keeps a 2D view inside it while the entity followed by og.Follow2D is in it: rooms, arenas, a level's bounds. Of overlapping zones the highest priority wins, then the first in query order. The engine's og.camera system picks the zone after the game's systems (into og.Camera's zone fields); entering another zone scrolls the view across over the new zone's scrollTime, or jumps when it is 0. A zone smaller than the view centres it.
 *
 * The value of the `CameraZone` component (token `og.CameraZone`).
 */
export interface CameraZone {
  /**
   * The bottom-left corner.
   *
   * @unit m (metres)
   */
  min: Vector2;
  /**
   * The top-right corner.
   *
   * @default { x: 10, y: 10 }
   * @unit m (metres)
   */
  max: Vector2;
  /** Higher wins where zones overlap. */
  priority: number;
  /**
   * Seconds the view takes to scroll from the zone the entity left into this one (a room transition, eased in and out); 0 jumps at once. Visual only: the camera resource records the transition, so replays scroll the same.
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  scrollTime: number;
}

/**
 * Moves og.Camera along a spline through p0 to p(points - 1) (Catmull-Rom, through every point) to the point nearest the entity carrying it, easing by smoothTime: a corridor, a side-scroller's track, a 3D chase on rails. A perspective camera looks at the entity; an orthographic one keeps its z and rotation. Applied by the engine's og.camera system after the game's systems.
 *
 * The value of the `CameraRail` component (token `og.CameraRail`).
 */
export interface CameraRail {
  /**
   * How many of p0...p7 the rail uses (2 to 8).
   *
   * @default 2
   * @range 2..8
   */
  points: number;
  /**
   * Rail point 0.
   *
   * @unit m (metres)
   */
  p0: Vector3;
  /**
   * Rail point 1.
   *
   * @default { x: 10, y: 0, z: 0 }
   * @unit m (metres)
   */
  p1: Vector3;
  /**
   * Rail point 2.
   *
   * @unit m (metres)
   */
  p2: Vector3;
  /**
   * Rail point 3.
   *
   * @unit m (metres)
   */
  p3: Vector3;
  /**
   * Rail point 4.
   *
   * @unit m (metres)
   */
  p4: Vector3;
  /**
   * Rail point 5.
   *
   * @unit m (metres)
   */
  p5: Vector3;
  /**
   * Rail point 6.
   *
   * @unit m (metres)
   */
  p6: Vector3;
  /**
   * Rail point 7.
   *
   * @unit m (metres)
   */
  p7: Vector3;
  /**
   * Roughly how long the camera takes to catch up along the rail; 0 snaps.
   *
   * @default 0.2
   * @range 0..
   * @unit s (seconds of simulation time)
   * @tunable
   */
  smoothTime: number;
  /**
   * A path (og.path.create, or a scene's `path` by og.path.find) the rail follows instead of p0 to p(points - 1): any number of points, catmullRom, bezier or linear, closed or not. 0 uses the points.
   */
  path: Path;
}




// ── render ──────────────────────────────────────────────────────
// What is drawn: meshes with materials. Draws are instanced by (geometry, material) automatically; there is no per-object draw call.

/**
 * Vertex and index data, as three.js BufferGeometry. Comes from a loaded glTF.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Geometry = number & { readonly [handleType]: "Geometry" };

/**
 * A material instance.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Material = number & { readonly [handleType]: "Material" };

/**
 * A cooked texture.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Texture = number & { readonly [handleType]: "Texture" };

/**
 * A Geometry handle field.
 *
 * A `Geometry` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Geometry(value?: Geometry, options?: FieldOptions): FieldSpec<"Geometry">;

/**
 * A Material handle field.
 *
 * A `Material` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Material(value?: Material, options?: FieldOptions): FieldSpec<"Material">;

/**
 * Which faces are drawn, as three.js FrontSide, BackSide and DoubleSide.
 * - `"front"`: Front faces only.
 * - `"back"`: Back faces only.
 * - `"double"`: Both.
 */
export type Side = "front" | "back" | "double";

/**
 * The backend the renderer runs on; the probe's hello reports it.
 * - `"webgpu"`: WebGPU in a browser: the full feature set.
 * - `"webgl2"`: The WebGL2 fallback: no compute, CPU culling and particles, tiled lights.
 * - `"vulkan"`: Native Vulkan on a GPU.
 * - `"vulkan-cpu"`: Native Vulkan on a CPU rasterizer (Mesa lavapipe or llvmpipe, SwiftShader): the executors' sandboxes. Frame times there say nothing about a player's GPU.
 * - `"metal"`: Native Metal.
 * - `"none"`: No renderer (a sandbox without a GPU; natively also OG_RENDERER=none): the simulation, the dumps and the world hash are the same as with one, a screenshot step writes no PNG and reports its frame unchecked, and capture answers null.
 */
export type RenderTier = "webgpu" | "webgl2" | "vulkan" | "vulkan-cpu" | "metal" | "none";

/**
 * Antialiasing, as og-render's Antialias. RenderCapabilities.antialias reports what the device resolved.
 * - `"tier"`: The tier's method: TAA on the full tier (WebGPU, Metal, Vulkan), 4x MSAA on WebGL2, FXAA where the HDR format cannot be multisampled.
 * - `"off"`: One sample per pixel.
 * - `"msaa4"`: 4x MSAA on any tier: sharp sprites and text, no temporal blur; FXAA where the HDR format cannot be multisampled.
 * - `"taa"`: Temporal antialiasing on any tier: a sub-pixel jitter per frame and a reprojected history; converges over about eight frames. RenderOptions.taaHistory and taaSharpen refine it.
 * - `"fxaa"`: FXAA on any tier: one full-screen pass over the tonemapped image (the overlay stays sharp), no extra samples and nothing to converge; softer than MSAA on fine texture.
 */
export type Antialias = "tier" | "off" | "msaa4" | "taa" | "fxaa";

/**
 * How a point light's shadow map is stored, as og-render's PointShadows. RenderCapabilities.pointShadows reports what the device resolved.
 * - `"auto"`: Cube faces on the full tier, paraboloids on WebGL2.
 * - `"cube"`: Six 90 degree faces: six shadow passes per light.
 * - `"paraboloid"`: Two paraboloid hemispheres (below and above the light): two passes per light instead of six, coarser, and large casters' silhouettes approximate (their edges stay straight in the curved projection).
 */
export type PointShadows = "auto" | "cube" | "paraboloid";

/**
 * How TAA resamples its history each frame, as og-render's TaaHistory.
 * - `"bilinear"`: One bilinear tap: cheap, slightly soft while the camera moves.
 * - `"catmullRom"`: A Catmull-Rom filter over 4x4 texels (9 taps): the history stays sharp under camera motion.
 */
export type TaaHistory = "bilinear" | "catmullRom";

/**
 * What the debug overlay shows besides og.debug's draws (og.debug.setView, RenderOptions.debugView, `debug:<view>`), drawn over the finished frame. Streams add views as they add what to show.
 * - `"none"`: Only og.debug's draws.
 * - `"physics"`: The colliders' shapes, joints and contacts as the last physics step left them.
 * - `"nav"`: The navigation grids (blocked cells) and navmeshes (polygon edges), and every og.NavAgent's path.
 * - `"bounds"`: Every drawn mesh's bounds (its geometry's box at its transform).
 * - `"lod"`: Every og.Model tinted by the level of detail it draws (green the full mesh, yellow level 1, orange level 2, red level 3 and beyond), so the levels' distances read in one frame.
 */
export type DebugView = "none" | "physics" | "nav" | "bounds" | "lod";

/**
 * How the renderer is created for a game (GameDesc.render). The host reads it once, when it attaches the renderer.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RenderOptions {
  /**
   * The antialiasing method.
   *
   * @default "tier"
   */
  readonly antialias?: Antialias;
  /**
   * The side of each of the directional light's three cascade maps, in texels; smaller saves memory on phones (three 2048 maps are 64 MB).
   *
   * @default 2048
   * @range 256..4096
   */
  readonly shadowMapSize?: number;
  /**
   * How point lights' shadows are stored: cube faces, or two paraboloid hemispheres (fewer passes, coarser).
   *
   * @default "auto"
   */
  readonly pointShadows?: PointShadows;
  /**
   * With TAA: how the history is resampled.
   *
   * @default "bilinear"
   */
  readonly taaHistory?: TaaHistory;
  /**
   * With TAA: a sharpening pass after it, 0 (none) to 1; it never pushes a pixel past its neighbours' range.
   *
   * @default 0
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly taaSharpen?: number;
  /**
   * A debug overlay from the first frame (og.debug.setView, the command channel's `debug:<view>`): none in a game, a view while building one. Not in publish builds.
   *
   * @default "none"
   */
  readonly debugView?: DebugView;
}

/**
 * A physically based metallic-roughness material with three.js MeshStandardMaterial parameter names.
 * It is og-render's reflected StandardMaterial field for field: metalnessMap and roughnessMap read a glTF metallic-roughness texture (blue metal, green rough), so give both the same texture; alphaTest above 0 discards below that alpha; transparent blends, sorted back to front.
 * The stylised fields: unlit (three.js MeshBasicMaterial), toon with its gradientMap ramp, a rim (fresnel) light, a hit flash, dissolve with a glowing edge, and a per-material outline. All are off at their defaults. og.MaterialStyle fills the ones a material leaves at their defaults unless the material sets styled false.
 * The MeshPhysicalMaterial fields, under three.js's names and glTF's KHR_materials_* meanings, all off at their defaults: clearcoat with clearcoatMap, clearcoatRoughnessMap and clearcoatNormalMap; sheen (velvet, cloth) with sheenColor, sheenRoughness and their maps; transmission (glass, water) refracting the opaque scene behind through thickness, attenuationColor and ior; iridescence (soap film, oil) with its film's iridescenceIOR and iridescenceThicknessRange; anisotropy (brushed metal) along anisotropyRotation or anisotropyMap. They draw on every tier, WebGL2 included; unlit and toon materials ignore them.
 * A transmissive material is drawn after the opaque scene, back to front with the transparent ones, and refracts what was drawn before it (not other transmissive or transparent objects); thickness is in the mesh's own units, scaled with it.
 * normalMap is read in glTF's tangent frame: red tilts towards +u, green up the image. The tangents come from the model (the cook generates MikkTSpace ones where a glTF has none) or are generated when the geometry is made; normals follow a non-uniform scale correctly.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface StandardMaterialDesc {
  /**
   * The base colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  readonly color?: Color;
  /**
   * The base colour texture.
   *
   * @default null
   */
  readonly map?: Texture | null;
  /**
   * How metallic the surface is.
   *
   * @default 0
   * @range 0..1
   */
  readonly metalness?: number;
  /**
   * Metalness from the texture's blue channel.
   *
   * @default null
   */
  readonly metalnessMap?: Texture | null;
  /**
   * How rough the surface is.
   *
   * @default 1
   * @range 0..1
   */
  readonly roughness?: number;
  /**
   * Roughness from the texture's green channel.
   *
   * @default null
   */
  readonly roughnessMap?: Texture | null;
  /**
   * A tangent-space normal map.
   *
   * @default null
   */
  readonly normalMap?: Texture | null;
  /**
   * Ambient occlusion from the texture's red channel.
   *
   * @default null
   */
  readonly aoMap?: Texture | null;
  /**
   * The emitted colour.
   *
   * @default { r: 0, g: 0, b: 0 }
   */
  readonly emissive?: Color;
  /**
   * A multiplier on the emitted colour.
   *
   * @default 1
   * @range 0..
   */
  readonly emissiveIntensity?: number;
  /**
   * The emitted colour texture.
   *
   * @default null
   */
  readonly emissiveMap?: Texture | null;
  /**
   * The opacity; below 1 only with transparent.
   *
   * @default 1
   * @range 0..1
   */
  readonly opacity?: number;
  /**
   * Blend instead of writing opaque pixels.
   *
   * @default false
   */
  readonly transparent?: boolean;
  /**
   * Discard pixels whose alpha is below this.
   *
   * @default 0
   * @range 0..1
   */
  readonly alphaTest?: number;
  /**
   * Which faces are drawn.
   *
   * @default "front"
   */
  readonly side?: Side;
  /**
   * Shade with face normals.
   *
   * @default false
   */
  readonly flatShading?: boolean;
  /**
   * A multiplier on image-based lighting.
   *
   * @default 1
   * @range 0..
   */
  readonly envMapIntensity?: number;
  /**
   * How strongly the normal map bends the normal, per tangent axis.
   *
   * @default { x: 1, y: 1 }
   */
  readonly normalScale?: Vector2;
  /**
   * How strongly aoMap darkens the ambient light.
   *
   * @default 1
   * @range 0..
   */
  readonly aoMapIntensity?: number;
  /**
   * No lighting at all, as three.js MeshBasicMaterial: the colour (times map) plus emissive.
   *
   * @default false
   */
  readonly unlit?: boolean;
  /**
   * Cel shading, as three.js MeshToonMaterial: light is quantized by gradientMap (three built-in bands without one), highlights are hard.
   *
   * @default false
   */
  readonly toon?: boolean;
  /**
   * The toon ramp: a texture whose first row maps the light's angle (left dark, right lit) to the light received, read without filtering.
   *
   * @default null
   */
  readonly gradientMap?: Texture | null;
  /**
   * Rim (fresnel) light added at grazing angles, unshadowed (linear).
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  readonly rimColor?: Color;
  /**
   * The rim light's strength; 0 is none.
   *
   * @default 0
   * @range 0..
   */
  readonly rimIntensity?: number;
  /**
   * How tight the rim is (higher: thinner).
   *
   * @default 3
   * @range 0.1..
   */
  readonly rimPower?: number;
  /**
   * The colour a hit flash mixes towards (linear, may exceed 1).
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  readonly flashColor?: Color;
  /**
   * How far the final colour is mixed towards flashColor (0 none, 1 flat): animate it per tick for a hit flash.
   *
   * @default 0
   * @range 0..1
   */
  readonly flash?: number;
  /**
   * The dissolved fraction: 0 whole, 1 gone. Pixels whose noise is below it are discarded (shadows too); the band just above glows with dissolveEdgeColor.
   *
   * @default 0
   * @range 0..1
   */
  readonly dissolve?: number;
  /**
   * The dissolve noise from the texture's red channel; without one, value noise over the object-space position.
   *
   * @default null
   */
  readonly dissolveMap?: Texture | null;
  /**
   * The glowing edge (linear, usually above 1 so it blooms).
   *
   * @default { r: 4, g: 1.5, b: 0.375 }
   */
  readonly dissolveEdgeColor?: Color;
  /**
   * The width of the glowing band in noise units.
   *
   * @default 0.0625
   * @range 0..1
   */
  readonly dissolveEdgeWidth?: number;
  /**
   * The frequency of the built-in noise (cells per object-space unit).
   *
   * @default 4
   * @range 0..
   */
  readonly dissolveScale?: number;
  /**
   * This material's outline colour (linear).
   *
   * @default { r: 0, g: 0, b: 0 }
   */
  readonly outlineColor?: Color;
  /**
   * This material's outline width in pixels; 0 is none (PostEffects.outline outlines everything). A hull outline, as toon shading draws one: the object's silhouette, where one part of it passes in front of another, and its creases against other objects (where it meets the ground or a wall), never the edges between its own faces, so a .vox model or any flat-shaded, hard-edged mesh keeps clean faces inside its silhouette; a face seen at a grazing angle (a box's side nearly edge-on, a large ground plane) is not lined across, only where it ends. Creases inside an object are Stylise.inkDepth's.
   *
   * @default 0
   * @range 0..31
   */
  readonly outlineWidth?: number;
  /**
   * Whether og.MaterialStyle (a look's scene-wide toon, rim, outline, flat facets, shadow tint and outline distance) reaches this material. false keeps it exactly as declared under any look: a lantern's lacquer, a UI-like prop, a hero kept photographic in a toon scene. A material with a MeshPhysicalMaterial lobe (clearcoat, sheen, transmission, iridescence, anisotropy, an ior other than 1.5) keeps its physical shading under the style's toon even when styled (toon would drop those lobes) and takes the rest of the style; its own toon true still makes it toon.
   *
   * @default true
   */
  readonly styled?: boolean;
  /**
   * A clear lacquer over the surface, as three.js MeshPhysicalMaterial.clearcoat: a second, 4 % reflective specular layer on the surface's normal before normalMap (car paint, varnished wood, wet plastic); 0 is none.
   *
   * @default 0
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly clearcoat?: number;
  /**
   * How rough the clear coat is, as three.js MeshPhysicalMaterial.clearcoatRoughness.
   *
   * @default 0
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly clearcoatRoughness?: number;
  /**
   * The coat's strength from the texture's red channel, times clearcoat (MeshPhysicalMaterial.clearcoatMap).
   *
   * @default null
   */
  readonly clearcoatMap?: Texture | null;
  /**
   * The coat's roughness from the texture's green channel, times clearcoatRoughness.
   *
   * @default null
   */
  readonly clearcoatRoughnessMap?: Texture | null;
  /**
   * A tangent-space normal map of the coat alone; the base keeps normalMap (MeshPhysicalMaterial.clearcoatNormalMap).
   *
   * @default null
   */
  readonly clearcoatNormalMap?: Texture | null;
  /**
   * How strongly clearcoatNormalMap bends the coat's normal, per tangent axis.
   *
   * @default { x: 1, y: 1 }
   */
  readonly clearcoatNormalScale?: Vector2;
  /**
   * A velvet-like layer that brightens grazing angles (cloth, fuzz, peach skin), as MeshPhysicalMaterial.sheen: its strength, times sheenColor; 0 is none.
   *
   * @default 0
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly sheen?: number;
  /**
   * The sheen's colour (linear), times sheenColorMap; black is none.
   *
   * @default { r: 0, g: 0, b: 0 }
   */
  readonly sheenColor?: Color;
  /**
   * How rough the sheen is: low keeps it to a thin rim, high spreads it over the surface.
   *
   * @default 1
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly sheenRoughness?: number;
  /**
   * The sheen's colour from the texture's RGB (sRGB).
   *
   * @default null
   */
  readonly sheenColorMap?: Texture | null;
  /**
   * The sheen's roughness from the texture's alpha channel.
   *
   * @default null
   */
  readonly sheenRoughnessMap?: Texture | null;
  /**
   * How much light passes through, as MeshPhysicalMaterial.transmission (glass, water, thin plastic): the opaque scene behind is refracted and blurred by roughness; 0 is none. Unlike opacity, the surface keeps its reflections.
   *
   * @default 0
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly transmission?: number;
  /**
   * Transmission from the texture's red channel.
   *
   * @default null
   */
  readonly transmissionMap?: Texture | null;
  /**
   * The volume's thickness under the surface in the mesh's own units (glTF KHR_materials_volume): how far refraction bends the view and how much attenuationColor absorbs; 0 is a thin wall.
   *
   * @default 0
   * @range 0..
   */
  readonly thickness?: number;
  /**
   * Thickness from the texture's green channel.
   *
   * @default null
   */
  readonly thicknessMap?: Texture | null;
  /**
   * The distance light travels in the volume before it is tinted to attenuationColor (world units); 0 is no absorption, three.js's Infinity.
   *
   * @default 0
   * @range 0..
   */
  readonly attenuationDistance?: number;
  /**
   * The colour white light becomes after attenuationDistance in the volume (linear).
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  readonly attenuationColor?: Color;
  /**
   * The index of refraction (glTF KHR_materials_ior): the reflectance of a dielectric, ((ior - 1) / (ior + 1))^2 (4 % at 1.5), and how far transmission refracts.
   *
   * @default 1.5
   * @range 1..2.333
   */
  readonly ior?: number;
  /**
   * A thin-film interference layer, as MeshPhysicalMaterial.iridescence (soap bubbles, oil, beetle shells); 0 is none.
   *
   * @default 0
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly iridescence?: number;
  /**
   * The film's index of refraction.
   *
   * @default 1.3
   * @range 1..2.333
   */
  readonly iridescenceIOR?: number;
  /**
   * The film's thickness in nanometres: x where iridescenceThicknessMap is black, y where it is white and everywhere without the map (three.js's [100, 400]).
   *
   * @default { x: 100, y: 400 }
   */
  readonly iridescenceThicknessRange?: Vector2;
  /**
   * Iridescence from the texture's red channel.
   *
   * @default null
   */
  readonly iridescenceMap?: Texture | null;
  /**
   * The film's thickness between iridescenceThicknessRange's ends from the texture's green channel.
   *
   * @default null
   */
  readonly iridescenceThicknessMap?: Texture | null;
  /**
   * Stretches the specular highlight along a direction of the surface, as MeshPhysicalMaterial.anisotropy (brushed metal, hair, satin); 0 is none, 1 fully stretched. Needs the geometry's tangents (generated when it has none).
   *
   * @default 0
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly anisotropy?: number;
  /**
   * The stretch's direction in radians from the tangent (+u) towards the bitangent.
   *
   * @default 0
   */
  readonly anisotropyRotation?: number;
  /**
   * The stretch's direction in red and green (-1 to 1 in the tangent frame, turned by anisotropyRotation) and its strength in blue, times anisotropy.
   *
   * @default null
   */
  readonly anisotropyMap?: Texture | null;
  /**
   * The base colour from layer mapLayer of a texture array (og.load.textureArray) instead of map: variants of one look (tiles, skins, damage states) are one material, each Mesh picking its layer with Mesh.materialLayer, and every mesh of the material is one draw whichever layer it shows. A material with mapArray or normalMapArray reads its base colour and normal map from arrays only (map and normalMap are not read); it draws on every tier, WebGL2 included. Physical materials (the MeshPhysicalMaterial fields) and shader materials do not take arrays.
   *
   * @default null
   */
  readonly mapArray?: TextureArray | null;
  /**
   * The layer of mapArray drawn where the Mesh does not pick one (past the array's last layer: the last).
   *
   * @default 0
   */
  readonly mapLayer?: number;
  /**
   * A tangent-space normal map from layer normalMapLayer of a texture array (linear: cook it with `kind normal`), the counterpart of mapArray.
   *
   * @default null
   */
  readonly normalMapArray?: TextureArray | null;
  /**
   * The layer of normalMapArray read where the Mesh does not pick one.
   *
   * @default 0
   */
  readonly normalMapLayer?: number;
  /**
   * Draw the edges of the mesh's triangles instead of its faces, as three.js Material.wireframe: one CSS pixel wide, in color plus the emissive light, unlit, depth tested; on every tier (lines, not a polygon mode WebGL2 lacks). Its meshes cast no shadow.
   *
   * @default false
   */
  readonly wireframe?: boolean;
}

/**
 * Draws a geometry with a material at the entity's MatrixWorld, as a three.js Mesh.
 * The material is a handle from og.render.standardMaterial or from a loaded glTF (GltfInfo.materials); its StandardMaterialDesc maps to the renderer as that struct's docs say.
 *
 * The value of the `Mesh` component (token `og.Mesh`).
 */
export interface Mesh {
  /** The geometry. */
  geometry: Geometry;
  /** The material. */
  material: Material;
  /**
   * Cast shadows, as three.js Object3D.castShadow: drawn into the sun's cascades and the shadowed lights' maps. Off by default: a lamp's own glass around its light, a glow card or a decal-like slab casts none.
   */
  castShadow: boolean;
  /**
   * Receive shadows, as three.js Object3D.receiveShadow: darkened where the sun's cascades and the shadowed lights' maps say something stands between it and the light. On by default (three.js's is off): every mesh received shadows before the flag was applied. false lights it as if unshadowed by every light, while it still casts as castShadow says: a glowing sign, a hero kept readable in a dark alley.
   *
   * @default true
   */
  receiveShadow: boolean;
  /**
   * Whether it is drawn.
   *
   * @default true
   */
  visible: boolean;
  /**
   * A hit flash on this mesh only: the lit colour mixed towards flashColor by this much (og.tween.flash fades it out). Carried with its transform to the lit shaders, as og.Model's, so a flashing mesh is one draw with the other meshes of its geometry and material, and the amount is continuous.
   *
   * @range 0..1
   * @tunable
   */
  flash: number;
  /**
   * The flash's linear colour, 0 to 1 a channel (8 bits a channel).
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  flashColor: Color;
  /**
   * The texture-array layer this mesh's material reads (StandardMaterialDesc.mapArray and normalMapArray), in place of the material's mapLayer and normalMapLayer: one material, one draw, a layer per mesh (a tile set, a crowd's skins). -1 keeps the material's layers; a material without arrays ignores it.
   *
   * @default -1
   * @range -1..
   * @tunable
   */
  materialLayer: number;
  /**
   * Three.js layers, as Object3D.layers.mask: bit n set puts it on layer n. A camera (og.Camera, og.CameraView, og.Mirror) draws it when their masks share a bit, and og.render.raycast tests it when its own `layers` does. Shadow maps take every caster whatever its layers, so a body hidden from a first-person camera still casts.
   *
   * @default 1
   */
  layers: number;
}

/**
 * The sun, as three.js DirectionalLight: parallel light along the entity's local -Z (aim it with transform.lookAt, as a camera). The first one found is drawn, with three shadow cascades.
 * Without one, the renderer's default sun (from above and to the side, intensity 3) lights the scene.
 *
 * The value of the `DirectionalLight` component (token `og.DirectionalLight`).
 */
export interface DirectionalLight {
  /**
   * The light's linear colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  color: Color;
  /**
   * The radiance multiplier, as three.js DirectionalLight.intensity.
   *
   * @default 3
   * @range 0..
   * @tunable
   */
  intensity: number;
  /**
   * Cast shadows through three cascades, as three.js Light.castShadow.
   *
   * @default true
   */
  castShadow: boolean;
  /**
   * How far from the camera shadows reach; they fade over the last tenth. Smaller is sharper.
   *
   * @default 200
   * @range 1..
   * @unit m (metres)
   */
  shadowDistance: number;
  /**
   * Soft shadows (PCSS): the penumbra's width per metre between caster and receiver, the tangent of the sun's apparent radius (the real sun's is 0.005). 0 keeps the hard one-texel edge; 0.01 to 0.05 is a soft sun, sharp at contact and blurring with distance. Costs a blocker search and a 16-tap filter per shadowed pixel.
   *
   * @range 0..0.2
   * @tunable
   */
  shadowSoftness: number;
  /**
   * The fraction of each shadow cascade's depth over which it fades into the next, hiding the seam where the shadow's resolution changes. 0 is none; 0.1 to 0.2 is typical (those pixels read two cascades).
   *
   * @range 0..0.5
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  cascadeBlend: number;
}

/**
 * A light shining in every direction from the entity's position, as three.js PointLight. Its radiance at distance d is color * intensity / d^2, faded smoothly to zero at distance.
 * Lights are clustered (Forward+): each pixel shades only the lights whose distance reaches it, so hundreds are cheap. Lights out of view are skipped; up to MAX_LIGHTS in view are drawn, and past that (or past what one screen cluster holds, MAX_CLUSTER_LIGHTS) the ones brightest at the camera are kept. Of those in view with castShadow, RenderCapabilities.maxShadowedLights (8 on the full tier, 4 on WebGL2) get shadows, the brightest at the camera first.
 *
 * The value of the `PointLight` component (token `og.PointLight`).
 */
export interface PointLight {
  /**
   * The light's linear colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  color: Color;
  /**
   * The radiance at one metre.
   *
   * @default 10
   * @range 0..
   * @tunable
   */
  intensity: number;
  /**
   * Where the light reaches zero, as three.js PointLight.distance (never infinite here).
   *
   * @default 10
   * @range 0.01..
   * @unit m (metres)
   * @tunable
   */
  distance: number;
  /**
   * Cast shadows: six cube faces or two paraboloid hemispheres (RenderOptions.pointShadows).
   */
  castShadow: boolean;
  /**
   * A light cookie projected all around the light: a cube of six square faces in one texture, stacked top to bottom as +X, -X, +Y, -Y, +Z, -Z (a KTX2 cube's face order and a cube map's orientation, world axes; the texture is six times as tall as it is wide). The light's colour is multiplied by the texel its direction meets, so a lamp behind a pierced shade throws its pattern on the walls. 0 is none.
   */
  cookie: Texture;
}

/**
 * A cone of light along the entity's local -Z, as three.js SpotLight.
 *
 * The value of the `SpotLight` component (token `og.SpotLight`).
 */
export interface SpotLight {
  /**
   * The light's linear colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  color: Color;
  /**
   * The radiance at one metre on the axis.
   *
   * @default 10
   * @range 0..
   * @tunable
   */
  intensity: number;
  /**
   * Where the light reaches zero.
   *
   * @default 10
   * @range 0.01..
   * @unit m (metres)
   * @tunable
   */
  distance: number;
  /**
   * The half-angle of the outer cone, as three.js SpotLight.angle.
   *
   * @default 0.5236
   * @range 0..1.5708
   * @unit rad (radians)
   * @tunable
   */
  angle: number;
  /**
   * The fraction of the cone over which the light fades out, as three.js SpotLight.penumbra.
   *
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  penumbra: number;
  /** Cast a shadow map. */
  castShadow: boolean;
  /**
   * A light cookie, as three.js SpotLight.map: the texture projected along the cone (its centre on the axis, its edges at the outer angle, its top towards world up), multiplying the light's colour, so a spot throws a window's bars or a stained-glass pattern. 0 is none. A shadowed spot keeps its shadow over the pattern.
   */
  cookie: Texture;
}

/**
 * The curve from the lit HDR image to the display, as three.js toneMapping. RenderSettings.toneMapped false clamps instead, whichever is chosen.
 * - `"aces"`: ACES (the fitted filmic curve), as three.js ACESFilmicToneMapping: the default.
 * - `"agx"`: AgX (Blender 4's and three.js AgXToneMapping's fit, base look): bright saturated lights go to white instead of skewing hue, and keep their hue further up than under ACES.
 * - `"none"`: No curve: the exposed linear colour is clamped to 0..1, so 2D colours come out exactly as authored at exposure 1 (the same as toneMapped false).
 */
export type ToneMapping = "aces" | "agx" | "none";

/**
 * A ToneMapping field: the variant's index.
 *
 * A `ToneMapping` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function ToneMapping(value?: ToneMapping, options?: FieldOptions): FieldSpec<"ToneMapping">;

/**
 * The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping. Read after each tick; it never feeds back into the simulation. Camera.exposure is the manual exposure.
 *
 * The `RenderSettings` resource (token `og.RenderSettings`).
 */
export interface RenderSettings {
  /**
   * The strength of the image-based ambient light from the sky; og.Environment.intensity takes its place while an environment map is set.
   *
   * @default 0.55
   * @range 0..
   * @tunable
   */
  environmentIntensity: number;
  /**
   * Draw the procedural sky behind the scene instead of Camera.background (with an og.Environment map set, the map).
   */
  sky: boolean;
  /** Light above the threshold bleeds into its surroundings. */
  bloom: boolean;
  /**
   * Brightness after exposure where bloom starts; 1 is display white.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  bloomThreshold: number;
  /**
   * Softens the threshold over threshold plus or minus knee; 0 is a hard cut.
   *
   * @default 0.5
   * @range 0..
   */
  bloomKnee: number;
  /**
   * How much of the blurred light is added back.
   *
   * @default 0.6
   * @range 0..
   * @tunable
   */
  bloomIntensity: number;
  /**
   * Meter every frame to middle grey, then apply Camera.exposure as compensation. With both adapt speeds 0 the exposure is the current frame's alone; otherwise it adapts over time like an eye, starting at the metered exposure of the first frame drawn after the first tick (a frame drawn before it, such as the web shell's first frame behind the play gate, meters at once and is never adapted from), so every host draws the same exposures for the same ticks.
   */
  autoExposure: boolean;
  /**
   * How fast auto exposure rises when the view gets darker, in EV per second of game time: each drawn frame moves it by at most the speed times the ticks since the last drawn frame (never the wall clock, so a replay and every host adapt the same, however many frames each drew); 0 is instant. Snapshots and probe sweeps carry the adapted exposure.
   *
   * @range 0..100
   * @unit evPerSecond (exposure stops (EV) per second of simulation time)
   * @tunable
   */
  adaptSpeedUp: number;
  /**
   * How fast auto exposure falls when the view gets brighter, in EV per second of game time; 0 is instant. Eyes adapt to brightness faster than to the dark: 3 down and 1 up is a natural pair.
   *
   * @range 0..100
   * @unit evPerSecond (exposure stops (EV) per second of simulation time)
   * @tunable
   */
  adaptSpeedDown: number;
  /**
   * Tone mapping by toneMapping; false clamps, so 2D colours come out exactly as authored at exposure 1.
   *
   * @default true
   */
  toneMapped: boolean;
  /**
   * Which curve tone maps the frame while toneMapped is true: aces (the default), agx or none.
   */
  toneMapping: ToneMapping;
  /**
   * A multiplier on every camera's Camera.exposure, as three.js renderer.toneMappingExposure is one for the whole frame: a look's brightness without touching the cameras; 1 leaves them as they are.
   *
   * @default 1
   * @range 0..16
   * @tunable
   */
  exposure: number;
  /**
   * The texture Decal components take their uvMin to uvMax region from; 0 draws decals as solid colour.
   */
  decalAtlas: Texture;
  /**
   * A multiplier on every model's projected size when its level of detail is picked (og.Model.lodBias multiplies it again): 1 is the engine's choice, 2 keeps full meshes twice as far, 0.5 simplifies sooner; 0 draws every model at its full mesh.
   *
   * @default 1
   * @range 0..16
   * @tunable
   */
  lodBias: number;
  /**
   * Draw the baked lighting of the scenes that have it (og bake lightmaps): static models with their lightmaps, everything else inside a LightVolume with its probes. False draws every scene as if unbaked, its ambient light from the sky or the environment map: to compare, or to see the realtime lights alone.
   *
   * @default true
   */
  bakedLighting: boolean;
}

/**
 * A colour lookup table for PostEffects.lut, from og.load.lut (a `.cube` file).
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Lut = number & { readonly [handleType]: "Lut" };

/**
 * A prefiltered HDR environment map for Environment.map, from og.load.environment (an `.ogt` of `type environment` naming an equirectangular `.hdr` or `.exr`, a KTX2 cube map, or six face images).
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type EnvironmentMap = number & { readonly [handleType]: "EnvironmentMap" };

/**
 * A particle emitter asset for ParticleSystem.emitter, from og.load.emitter (an `.ogt` of `type particles`).
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type ParticleEmitter = number & { readonly [handleType]: "ParticleEmitter" };

/**
 * A Texture handle field (0 is none).
 *
 * A `Texture` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Texture(value?: Texture, options?: FieldOptions): FieldSpec<"Texture">;

/**
 * A Lut handle field (0 is none).
 *
 * A `Lut` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Lut(value?: Lut, options?: FieldOptions): FieldSpec<"Lut">;

/**
 * An EnvironmentMap handle field (0 is none).
 *
 * A `EnvironmentMap` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function EnvironmentMap(value?: EnvironmentMap, options?: FieldOptions): FieldSpec<"EnvironmentMap">;

/**
 * Image-based light from a cooked HDR environment map instead of the sky, as three.js scene.environment, and the map behind the scene, as scene.background = the environment. Load the map with og.load.environment: the cook has already prefiltered it (GGX levels for reflections from mirror to rough, harmonics for the diffuse light), so setting it costs a few copy passes on any tier, WebGL2 included. Without a map the sky lights the scene as before (og.Sky, a DayNight, or the built-in one). Read after each tick; it never feeds back into the simulation.
 *
 * The `Environment` resource (token `og.Environment`).
 */
export interface Environment {
  /** The map from og.load.environment; 0 lights by the sky. */
  map: EnvironmentMap;
  /**
   * The strength of the map's light on every surface, as three.js scene.environmentIntensity; while a map is set it replaces RenderSettings.environmentIntensity.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  intensity: number;
  /**
   * Turns the map about +Y, as three.js scene.environmentRotation.y: positive turns what was at +X towards -Z. The light and the background turn together.
   *
   * @unit deg (degrees)
   * @tunable
   */
  rotation: number;
  /**
   * Draw the map behind the scene, as three.js scene.background = the environment (Camera.background, og.Sky and RenderSettings.sky give way); false leaves the background as they set it.
   */
  background: boolean;
  /**
   * Multiplies the map where it is drawn behind the scene, as three.js scene.backgroundIntensity; the light is not changed.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  backgroundIntensity: number;
}

/**
 * A ParticleEmitter handle field (0 is none).
 *
 * A `ParticleEmitter` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function ParticleEmitter(value?: ParticleEmitter, options?: FieldOptions): FieldSpec<"ParticleEmitter">;

/**
 * Which ambient occlusion algorithm runs.
 * - `"auto"`: GTAO on the full tier, SSAO on WebGL2.
 * - `"gtao"`: Ground-truth ambient occlusion (3 slices): the better looking one.
 * - `"ssao"`: Hemisphere SSAO with 8 samples: the cheaper one.
 */
export type AoMethod = "auto" | "gtao" | "ssao";

/**
 * An AoMethod field: the variant's index.
 *
 * A `AoMethod` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function AoMethod(value?: AoMethod, options?: FieldOptions): FieldSpec<"AoMethod">;

/**
 * How many taps depth of field gathers per pixel.
 * - `"low"`: 16 taps at half resolution.
 * - `"high"`: 48 taps at half resolution: rounder, smoother bokeh.
 */
export type DofQuality = "low" | "high";

/**
 * A DofQuality field: the variant's index.
 *
 * A `DofQuality` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function DofQuality(value?: DofQuality, options?: FieldOptions): FieldSpec<"DofQuality">;

/**
 * How a sprite is blended over what is below it.
 * - `"alpha"`: Ordinary alpha blending.
 * - `"additive"`: Added (light, fire, magic): order-independent within a layer.
 */
export type SpriteBlend = "alpha" | "additive";

/**
 * A SpriteBlend field: the variant's index.
 *
 * A `SpriteBlend` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function SpriteBlend(value?: SpriteBlend, options?: FieldOptions): FieldSpec<"SpriteBlend">;

/**
 * How a trail, ribbon or beam is blended.
 * - `"additive"`: Added: glowing streaks, lasers, magic.
 * - `"alpha"`: Alpha blended: smoke trails, tyre marks.
 */
export type StripBlend = "additive" | "alpha";

/**
 * A StripBlend field: the variant's index.
 *
 * A `StripBlend` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function StripBlend(value?: StripBlend, options?: FieldOptions): FieldSpec<"StripBlend">;

/**
 * The post stack beyond RenderSettings' bloom, exposure and tone mapping, and camera shake. Every effect is off by default (its switch false or its amount 0) and then costs nothing: no pass is recorded. Read after each tick; it never feeds back into the simulation.
 * Screen-space ambient occlusion, outlines, depth of field, motion blur and god rays run a depth-normal prepass, turned on with them. Grading, vignette, aberration, grain and the flash apply to the display colour after tone mapping; the UI and screen-space sprites (the overlay) come after all of it.
 *
 * The `PostEffects` resource (token `og.PostEffects`).
 */
export interface PostEffects {
  /** Screen-space ambient occlusion: darkens the ambient light in creases and contact areas. */
  ao: boolean;
  /**
   * The world-space radius searched for occluders.
   *
   * @default 0.6
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  aoRadius: number;
  /**
   * 0 none, 1 full; above 1 exaggerates.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  aoIntensity: number;
  /** The algorithm. */
  aoMethod: AoMethod;
  /**
   * Outlines every opaque surface from depth and normal edges, creases between the faces of one object included (materials with outlineWidth get their own hull outline instead).
   */
  outline: boolean;
  /**
   * The line colour, linear.
   *
   * @default { r: 0.02, g: 0.02, b: 0.03 }
   */
  outlineColor: Color;
  /**
   * Line width in pixels, drawn inside the silhouette.
   *
   * @default 2
   * @range 0..31
   * @tunable
   */
  outlineWidth: number;
  /**
   * The relative depth step that counts as a silhouette (0.04 is 4 %).
   *
   * @default 0.04
   * @range 0..
   */
  outlineDepthThreshold: number;
  /**
   * The normal change that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees).
   *
   * @default 0.4
   * @range 0..
   */
  outlineNormalThreshold: number;
  /** Bokeh depth of field: pixels away from the focus distance blur into discs. */
  depthOfField: boolean;
  /**
   * The distance from the camera that is sharp.
   *
   * @default 10
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  dofFocusDistance: number;
  /**
   * The half-width of the sharp band around it.
   *
   * @default 2
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  dofFocusRange: number;
  /**
   * The largest blur radius, as a fraction of the image height (0.012 is about 9 px at 720p).
   *
   * @default 0.012
   * @range 0..0.1
   */
  dofMaxBlur: number;
  /**
   * How quickly blur grows outside the band: full blur focusRange times (1 + falloff) away.
   *
   * @default 4
   * @range 0..
   */
  dofFalloff: number;
  /** Taps per pixel. */
  dofQuality: DofQuality;
  /** Per-object and camera motion blur along the screen-space velocity. */
  motionBlur: boolean;
  /**
   * The shutter as a fraction of the frame (0.5 is a 180 degree shutter).
   *
   * @default 0.5
   * @range 0..2
   * @tunable
   */
  motionBlurShutter: number;
  /**
   * Taps along the velocity.
   *
   * @default 12
   * @range 4..32
   */
  motionBlurSamples: number;
  /**
   * The longest blur in pixels.
   *
   * @default 40
   * @range 1..
   */
  motionBlurMaxPixels: number;
  /**
   * Screen-space light shafts from the DirectionalLight (the sun must be in front of the camera).
   */
  godRays: boolean;
  /**
   * How bright the shafts are.
   *
   * @default 0.5
   * @range 0..
   * @tunable
   */
  godRaysIntensity: number;
  /**
   * How fast the rays fade along their length.
   *
   * @default 0.965
   * @range 0.5..1
   */
  godRaysDecay: number;
  /**
   * How far towards the sun each ray samples (a fraction of the distance).
   *
   * @default 1
   * @range 0..
   */
  godRaysDensity: number;
  /**
   * A linear tint on the light the rays carry.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  godRaysTint: Color;
  /**
   * Taps per ray.
   *
   * @default 48
   * @range 8..128
   */
  godRaysSamples: number;
  /**
   * Sky brightness (linear, before exposure) below which the sky carries no rays.
   *
   * @default 2
   * @range 0..
   */
  godRaysThreshold: number;
  /**
   * Order-independent 2D glow: sprites with SpriteDesc.glow (and 2D particles with a glow) are blurred and added back, with or without bloom.
   */
  glow: boolean;
  /**
   * How much of the blurred glow is added back.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  glowIntensity: number;
  /**
   * Colour grading by a lookup table from og.load.lut; 0 is none (and so is a LUT whose streaming group is not resident).
   */
  lut: Lut;
  /**
   * 0 leaves the image as tone mapped, 1 applies the table fully.
   *
   * @default 1
   * @range 0..1
   * @tunable
   */
  lutIntensity: number;
  /** Darkened (or tinted) edges of the frame. */
  vignette: boolean;
  /**
   * 0 none, 1 black corners.
   *
   * @default 0.35
   * @range 0..1
   * @tunable
   */
  vignetteIntensity: number;
  /**
   * Where darkening starts, as a fraction of the half-diagonal.
   *
   * @default 0.55
   * @range 0..1
   */
  vignetteRadius: number;
  /**
   * The width of the fade.
   *
   * @default 0.45
   * @range 0..1
   */
  vignetteSmoothness: number;
  /** The linear colour the edges move towards. */
  vignetteColor: Color;
  /**
   * Red and blue split towards the edges, in pixels at the corners at 720p; 0 is off.
   *
   * @range 0..20
   * @tunable
   */
  chromaticAberration: number;
  /**
   * Luminance-weighted noise amplitude in display units, a new pattern every tick (0.04 is subtle); 0 is off.
   *
   * @range 0..1
   * @tunable
   */
  filmGrain: number;
  /**
   * The grain size in pixels.
   *
   * @default 1.5
   * @range 0.5..
   */
  filmGrainSize: number;
  /**
   * A full-screen flash: the display colour mixed towards flashColor by this much (animate it per tick); 0 is off.
   *
   * @range 0..1
   * @tunable
   */
  flash: number;
  /**
   * The flash's linear display colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  flashColor: Color;
  /**
   * Camera shake held at a fixed level (decay it yourself): 0 none, 1 the most (the amplitude is trauma squared). Prefer og.camera.shake, which decays by itself; the frame shakes by the larger of the two. Smooth value noise by the tick, so a replay shakes the same. Visual only.
   *
   * @range 0..1
   * @tunable
   */
  shakeTrauma: number;
  /**
   * The largest offset at trauma 1, as a fraction of the view height.
   *
   * @default 0.05
   * @range 0..
   */
  shakeMaxOffset: number;
  /**
   * The largest roll at trauma 1.
   *
   * @default 3
   * @range 0..
   * @unit deg (degrees)
   */
  shakeMaxRoll: number;
  /**
   * How fast the shake moves.
   *
   * @default 12
   * @range 0..
   * @unit hz (ticks or cycles per second)
   */
  shakeFrequency: number;
  /** The shake's noise seed. */
  shakeSeed: number;
  /**
   * Screen-space reflections on the full tier: smooth surfaces reflect what the frame already drew by marching the depth-normal prepass along the reflected ray, faded by roughness and at the screen's edges, over the reflection probes and the environment where the ray misses. WebGL2 draws none (RenderCapabilities.ssr is false) and smooth surfaces keep the probes' reflection.
   */
  ssr: boolean;
  /**
   * How far a reflected ray marches, in world units.
   *
   * @default 20
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  ssrMaxDistance: number;
  /**
   * How far behind the depth buffer a surface is taken to extend: a ray passing behind a surface by less than this hits it.
   *
   * @default 0.3
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  ssrThickness: number;
  /**
   * Steps along each ray (a binary refinement follows the hit).
   *
   * @default 32
   * @range 4..128
   */
  ssrSteps: number;
  /**
   * The fraction of the screen's edge, and of the march's end, over which a reflection fades into the probes'.
   *
   * @default 0.15
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  ssrFade: number;
  /**
   * How much of the reflection is laid over the frame; 1 is physically weighted by the surface's Fresnel.
   *
   * @default 1
   * @range 0..1
   * @tunable
   */
  ssrIntensity: number;
  /**
   * The first custom post pass (og.load.post, a `type post` asset; 0 is none). PostEffects.custom1 to custom8 are an ordered list, as many as og-render runs (RenderCapabilities.maxCustomPosts): each runs over the one before it (the lit image for the first), after depth of field and motion blur and before exposure, bloom and the tone curve, and reads the lit image, depth and normals too (it turns the prepass on). Its parameters are set with og.render.setPostParam. A pass the device fails to compile is reported (og.render.postErrors) and skipped.
   */
  custom1: Post;
  /** The second custom post pass, over the first's output (0 skips the slot). */
  custom2: Post;
  /** The third custom post pass. */
  custom3: Post;
  /** The fourth custom post pass. */
  custom4: Post;
  /** The fifth custom post pass. */
  custom5: Post;
  /** The sixth custom post pass. */
  custom6: Post;
  /** The seventh custom post pass. */
  custom7: Post;
  /** The eighth and last custom post pass, over every one before it. */
  custom8: Post;
}

/**
 * A colour palette for Stylise.palette, from og.load.palette (a `.hex` or GIMP `.gpl` file of up to 256 colours).
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Palette = number & { readonly [handleType]: "Palette" };

/**
 * A look from og.load.look (an `.ogt` of `type look`): the frame's whole art direction as data, applied with og.render.applyLook.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Look = number & { readonly [handleType]: "Look" };

/**
 * A Palette handle field (0 is none).
 *
 * A `Palette` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Palette(value?: Palette, options?: FieldOptions): FieldSpec<"Palette">;

/**
 * The threshold pattern Stylise.dither spreads the steps of posterize and palette with.
 * - `"none"`: No dither: every pixel rounds to its nearest step or colour, in flat bands.
 * - `"bayer2"`: Ordered dither by a 2 by 2 Bayer matrix: the coarsest checker, as on early handhelds.
 * - `"bayer4"`: Ordered dither by a 4 by 4 Bayer matrix: the cross-hatched ramps of 8- and 16-bit art.
 * - `"bayer8"`: Ordered dither by an 8 by 8 Bayer matrix: finer ramps with 64 levels between two colours.
 * - `"blueNoise"`: Blue noise: an even grain without a visible grid (a 64 by 64 void-and-cluster tile the engine ships, repeated), fixed per pixel so a still frame does not crawl.
 * - `"interleavedGradient"`: Interleaved gradient noise: computed per pixel without a texture; a finer diagonal structure than blueNoise up close.
 */
export type DitherPattern = "none" | "bayer2" | "bayer4" | "bayer8" | "blueNoise" | "interleavedGradient";

/**
 * A DitherPattern field: the variant's index.
 *
 * A `DitherPattern` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function DitherPattern(value?: DitherPattern, options?: FieldOptions): FieldSpec<"DitherPattern">;

/**
 * Where Stylise lays a pattern (the paint canvas, the hatching, the halftone).
 * - `"screen"`: Over the frame in logical pixels, as a print or a sheet of paper over the screen: the pattern stays put while the camera moves and the scene slides under it.
 * - `"surface"`: On the surfaces, through the depth-normal prepass's position and normal (three world planes blended by the normal): the pattern sticks to the scene as the camera moves and wraps round forms, a tonal art map's strokes on the objects. Hatching and halftone keep about their pixel spacing by fading between whole octaves of world spacing; the sky keeps the screen pattern. The frame runs the prepass PostEffects.outline uses.
 * - `"object"`: On the surfaces as surface, but in each mesh instance's own space (three planes of the instance blended by its normal, in metres along its axes): a moving, turning or skinned object carries its strokes, dots and grain instead of sliding through them, and keeps them however it is scaled. Terrain, scattered copies and the vertices a material shader moves keep the world pattern, the sky the screen pattern. The frame runs the prepass and an object pass that draws the opaque instances again.
 */
export type StyliseSpace = "screen" | "surface" | "object";

/**
 * A StyliseSpace field: the variant's index.
 *
 * A `StyliseSpace` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function StyliseSpace(value?: StyliseSpace, options?: FieldOptions): FieldSpec<"StyliseSpace">;

/**
 * Stylisation primitives that move the frame away from the photographic: a low-resolution pixel grid scaled up by whole numbers, fewer tones or a fixed palette through an ordered or blue-noise dither, halftone dots, cross-hatching, painterly smoothing over a canvas, ink lines on tonal edges, flat colour bands by depth, and the vertex snapping, affine textures and coarse depth of early 3D hardware. They compose: a look (og.load.look) sets this with PostEffects and RenderSettings, and the game's art direction is in that data, not in the engine. Every primitive is off at its zero value (0, false, none) and then costs nothing: no pass is recorded and no pipeline is built. RenderCapabilities.stylise names the primitives this renderer draws; a field it does not name is carried and read back but not drawn. Read after each tick; it never feeds back into the simulation.
 * Order in the frame: depth bands colour the lit scene with the fog, before tone mapping; after tone mapping and PostEffects' grading come paint, hatching, halftone and ink, then posterize and palette quantise through the dither, all at the logical resolution, and the pixel grid is scaled up last. The UI and screen-space sprites are drawn over the scaled frame at the output's resolution, unstylised.
 *
 * The `Stylise` resource (token `og.Stylise`).
 */
export interface Stylise {
  /**
   * The logical resolution: the scene is rendered this many pixels tall and scaled up to the output by a whole number with nearest filtering, as a low-resolution three.js render target drawn with NearestFilter; 0 renders at the output's resolution. The height sets the scale (pixelUpscale); the width is the output's aspect times the height, rounded, but at most the output's width over the scale, so the last column gives way rather than the scale: 240 rows at 1280x720 are 426.67 columns, rounded to 427, and 427 x 3 = 1281 does not fit, so the grid is 426 x 240 at 3x (1278 x 720, a 1-pixel border each side). Heights that divide the output's evenly fill it: 180 (4x), 240 (3x) and 360 (2x) at 720p, 270 (4x) and 360 (3x) at 1080p.
   *
   * @range 0..2160
   */
  pixelHeight: number;
  /**
   * The whole-number scale from the logical to the output resolution, a hard request: honoured whenever pixelHeight times it fits the output's height (a smaller scale than the largest leaves the border it implies: 240 lines at 2x in 1280x720 draw 854 x 480 in a 213 x 120 border), else the largest that fits; 0 takes the largest that fits (720 / 240 = 3, 1080 / 240 = 4, rounded down). A frame that does not fill the output is centred with Camera.background round it.
   *
   * @range 0..16
   */
  pixelUpscale: number;
  /**
   * Snap the camera and world sprites to the logical pixel grid, so a moving orthographic camera scrolls by whole pixels and sprites in a 3D world keep their edges (pixel sprites in 3D sets).
   *
   * @default true
   */
  pixelSnap: boolean;
  /**
   * Tones per colour channel after tone mapping (2 to 64): smooth gradients become flat steps; 0 is off.
   *
   * @range 0..64
   */
  posterize: number;
  /**
   * Quantise every pixel to the nearest colour of this palette (og.load.palette), measured in a perceptual space after posterize; 0 is none.
   */
  palette: Palette;
  /**
   * How the steps of posterize and palette are mixed: none rounds every pixel, an ordered or blue-noise pattern alternates the two nearest steps in proportion.
   */
  dither: DitherPattern;
  /**
   * The dither's amplitude in steps: 1 spreads exactly one step, less keeps flatter bands.
   *
   * @default 1
   * @range 0..2
   * @tunable
   */
  ditherStrength: number;
  /**
   * The size of one dither cell in logical pixels.
   *
   * @default 1
   * @range 1..8
   */
  ditherScale: number;
  /**
   * Halftone print: tones become dots on a rotated grid, larger where the image is darker; 0 is off, 1 dots only.
   *
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  halftone: number;
  /**
   * The dot grid's pitch in logical pixels.
   *
   * @default 6
   * @range 2..64
   */
  halftoneSize: number;
  /**
   * The dot grid's angle.
   *
   * @default 45
   * @unit deg (degrees)
   */
  halftoneAngle: number;
  /**
   * The ink of the dots (linear).
   *
   * @default { r: 0.02, g: 0.02, b: 0.03 }
   */
  halftoneColor: Color;
  /**
   * Where the dots lie: screen, a grid over the frame; surface, grids on the surfaces that stay on them as the camera moves and wrap round forms (halftoneSize is then about the pitch in pixels where they lie); object, the same grids in each instance's own space, so a moving object carries its dots.
   */
  halftoneSpace: StyliseSpace;
  /**
   * Pencil or ink cross-hatching from a tonal art map: darker tones add strokes in more directions over a paper-light ground; 0 is off, 1 strokes only.
   *
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  hatching: number;
  /**
   * The spacing of hatch lines in logical pixels.
   *
   * @default 6
   * @range 2..64
   */
  hatchingScale: number;
  /**
   * How many tones the hatching distinguishes, each adding one stroke direction or density (1 to 6).
   *
   * @default 4
   * @range 1..6
   */
  hatchingLevels: number;
  /**
   * The direction of the first stroke set.
   *
   * @default 45
   * @unit deg (degrees)
   */
  hatchingAngle: number;
  /**
   * The stroke colour (linear): graphite, sepia or ink.
   *
   * @default { r: 0.04, g: 0.035, b: 0.03 }
   */
  hatchingColor: Color;
  /**
   * Where the strokes lie: screen, stripes over the frame that follow the tone alone; surface, strokes laid on the surfaces that follow the tone and the form, wrapping round curved objects and staying on them as the camera moves (hatchingScale is then about the spacing in pixels where they lie); object, the same strokes in each instance's own space, so a moving or turning object carries its strokes.
   */
  hatchingSpace: StyliseSpace;
  /**
   * Hatching from luminance and depth: the depth-normal prepass's silhouettes and creases darken the tone the strokes follow, at this strength, so strokes gather along contours and folds as inkDepth inks them (the edges are inkDepthThreshold's and inkNormalThreshold's, found within one stroke spacing). 0 is the tone alone; above 0 the frame runs the prepass PostEffects.outline uses.
   *
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  hatchingDepth: number;
  /**
   * Painterly smoothing (a Kuwahara filter): flat regions become strokes of one colour this many logical pixels across while edges stay sharp; 0 is off.
   *
   * @range 0..8
   * @tunable
   */
  paintRadius: number;
  /**
   * How far the strokes follow the image's edges: 0 square brush cells, 1 strokes stretched along every contour (an anisotropic Kuwahara).
   *
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  paintAnisotropy: number;
  /**
   * Pigment pooling at the edges of colour regions, as wet watercolour dries darker at its rim; 0 is none.
   *
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  paintEdgeDarkening: number;
  /**
   * Colour bleeding across edges into its neighbours, wet in wet; 0 is none.
   *
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  paintBleed: number;
  /**
   * A tileable paper or canvas texture (og.load.texture) laid over the frame by paintCanvasStrength; 0 is none.
   */
  paintCanvas: Texture;
  /**
   * How much the canvas texture modulates the frame: 0 none, 1 its full grain.
   *
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  paintCanvasStrength: number;
  /**
   * Canvas texels per logical pixel, with paintCanvasSpace screen (and on the sky).
   *
   * @default 1
   * @range 0.125..8
   */
  paintCanvasScale: number;
  /**
   * Where the canvas lies: screen, over the frame (the scene slides under the paper as the camera moves); surface, on the surfaces, so the grain sticks to the scene, foreshortened with the floor and walls; object, in each instance's own space, so a moving object carries its grain.
   */
  paintCanvasSpace: StyliseSpace;
  /**
   * Canvas texels per metre with paintCanvasSpace surface: 128 repeats a 256-texel canvas every two metres.
   *
   * @default 128
   * @range 1..4096
   */
  paintCanvasWorldScale: number;
  /**
   * Ink lines where the tone changes sharply (luminance edges), over every surface, sprite and texture alike: comic and storybook contours without PostEffects.outline's depth prepass. inkDepth adds the prepass's silhouettes and creases.
   */
  ink: boolean;
  /**
   * The luminance step that counts as an edge (0 to 1 in display units): lower draws more lines.
   *
   * @default 0.1
   * @range 0..1
   * @tunable
   */
  inkThreshold: number;
  /**
   * The line width in logical pixels.
   *
   * @default 1
   * @range 0.5..8
   * @tunable
   */
  inkWidth: number;
  /**
   * The line colour (linear).
   *
   * @default { r: 0.02, g: 0.02, b: 0.03 }
   */
  inkColor: Color;
  /**
   * With ink: lines also where the depth-normal prepass breaks (silhouettes against what is behind, creases between faces), at this strength: comic contours round flat colours that luminance alone misses. 0 is luminance only; above 0 the frame runs the prepass PostEffects.outline uses.
   *
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  inkDepth: number;
  /**
   * With inkDepth: the relative depth step to a farther neighbour that counts as a silhouette (0.04 is 4 %), as PostEffects.outlineDepthThreshold: lower inks smaller steps.
   *
   * @default 0.04
   * @range 0..
   */
  inkDepthThreshold: number;
  /**
   * With inkDepth: the normal change between neighbours of about the same depth that counts as a crease (1 - cos of the angle; 0.4 is about 53 degrees), as PostEffects.outlineNormalThreshold: lower inks gentler folds.
   *
   * @default 0.4
   * @range 0..2
   */
  inkNormalThreshold: number;
  /**
   * Colour bands by depth (2 to 8): the scene from depthBandsNear to depthBandsFar is cut into this many equal layers, the first in depthBandsNearColor, the last in depthBandsFarColor, those between stepping evenly from one to the other in lightness, chroma and hue (never through grey), for layered-silhouette and stage looks; 0 is off. For side and three-quarter views, where the depth runs across the frame: seen from above the play area lies at one depth and falls in one band.
   *
   * @range 0..8
   */
  depthBands: number;
  /**
   * Where the first band starts.
   *
   * @default 5
   * @range 0..
   * @unit m (metres)
   */
  depthBandsNear: number;
  /**
   * Where the last band ends; the last band (the far colour) starts one band's depth before it, and everything beyond it is the far colour too.
   *
   * @default 60
   * @range 0..
   * @unit m (metres)
   */
  depthBandsFar: number;
  /**
   * The nearest band's colour (linear, before exposure).
   *
   * @default { r: 0.02, g: 0.02, b: 0.03 }
   */
  depthBandsNearColor: Color;
  /**
   * The farthest band's colour (linear, before exposure).
   *
   * @default { r: 0.55, g: 0.6, b: 0.7 }
   */
  depthBandsFarColor: Color;
  /**
   * How far a band replaces the surfaces' own shading: 1 flat silhouettes in the band's colour; 0 a tint over the lit scene, its colours times the band's at unit luminance (a coloured filter: the shading, the value and the surfaces' own hues show through); between, a mix of the two.
   *
   * @default 1
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  depthBandsFlatten: number;
  /**
   * Snap every vertex of the 3D scene to a screen grid this many logical pixels wide, as fixed-point hardware did: geometry jitters by whole steps as it moves; 0 is off.
   *
   * @range 0..16
   */
  vertexSnap: number;
  /**
   * How far texture coordinates are interpolated without perspective correction: 1 fully affine, so textures warp and swim across large polygons as on early consoles; 0 is off.
   *
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  affineTexturing: number;
  /**
   * Quantise depth to this many bits, for the coarse depth buffers of early 3D (12 to 16 are period-like: distant surfaces fight); 0 keeps full precision.
   *
   * @range 0..24
   */
  depthBits: number;
}

/**
 * Stylised defaults for every lit material, so a whole scene takes one treatment without editing each material: toon shading and its ramp, a coloured shadow side, rim light, outlines whose width falls off with distance, and flat facets. Each applies to a material (og.render.standardMaterial or a loaded glTF's) that leaves that field at its own default; unlit and shader materials, materials with StandardMaterialDesc.styled false, sprites and the UI are untouched, and toon passes by a material with a MeshPhysicalMaterial lobe (clearcoat, sheen, transmission, iridescence, anisotropy, ior), which keeps its physical shading. All off (false, 0) is no change. A look sets it; RenderCapabilities.stylise names what this renderer draws. Read after each tick; it never feeds back into the simulation.
 *
 * The `MaterialStyle` resource (token `og.MaterialStyle`).
 */
export interface MaterialStyle {
  /**
   * Cel shading for every lit material, as StandardMaterialDesc.toon: light quantised by the ramp, hard highlights. A material with a physical lobe (clearcoat, transmission and the rest) keeps its physical shading; set its own toon to make it toon anyway.
   */
  toon: boolean;
  /**
   * The toon ramp for materials without their own (StandardMaterialDesc.gradientMap, og.load.texture): its first row maps the light's angle to the light received, read without filtering; 0 is three built-in bands.
   */
  gradientMap: Texture;
  /**
   * The colour the shadowed side of a toon or lit surface moves towards instead of black (linear): the cool or warm shadow of illustration.
   *
   * @default { r: 0.18, g: 0.16, b: 0.32 }
   */
  shadowTint: Color;
  /**
   * How far shadowed light moves towards shadowTint; 0 is off.
   *
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  shadowTintAmount: number;
  /**
   * Rim (fresnel) light for materials without their own (linear).
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  rimColor: Color;
  /**
   * The rim light's strength; 0 is none.
   *
   * @range 0..
   * @tunable
   */
  rimIntensity: number;
  /**
   * How tight the rim is (higher: thinner).
   *
   * @default 3
   * @range 0.1..
   */
  rimPower: number;
  /**
   * The outline colour for materials without their own (linear).
   *
   * @default { r: 0.02, g: 0.02, b: 0.03 }
   */
  outlineColor: Color;
  /**
   * The outline width in pixels at outlineDistance for materials without their own; 0 is none. A hull outline (StandardMaterialDesc.outlineWidth): silhouettes and creases against other objects, never the faces of one, so voxel and flat-shaded models stay clean. A surface seen at a grazing angle is lined only where it ends: a 1,600 m ground under a chase camera keeps its far edge and nothing else (no band along the horizon).
   *
   * @range 0..31
   * @tunable
   */
  outlineWidth: number;
  /**
   * The distance at which outlines are outlineWidth wide: nearer they grow, up to four times outlineWidth, farther they thin in proportion (an inked look keeps line weight by depth); 0 keeps every outline outlineWidth wide.
   *
   * @range 0..
   * @unit m (metres)
   */
  outlineDistance: number;
  /**
   * Shade every lit material with face normals, as StandardMaterialDesc.flatShading: visible facets for low-poly, paper-craft and voxel looks.
   */
  flatShading: boolean;
}

/** A loaded look: where it came from and what applying it sets. */
export interface LookInfo {
  /** Its handle. */
  readonly look: Look;
  /** Its asset id. */
  readonly path: string;
  /** Its name (the file's `name`, else its file name without `.ogt`). */
  readonly name: string;
  /**
   * The looks it builds on, nearest first: asset ids under assets/ and built-in preset slugs.
   */
  readonly extends: readonly string[];
  /**
   * The resources it sets, by IDL name (RenderSettings, PostEffects, Stylise, MaterialStyle, Fog, Sky, Environment). Applying it replaces each of them whole: the fields the look names take its values, the rest their defaults.
   */
  readonly resources: readonly string[];
  /**
   * The UI style its `ui` line names (a game's own or a built-in by slug), loaded with it; applying the look configures the UI with it.
   */
  readonly style: Style | null;
  /**
   * The antialiasing it asks for. GameDesc.look's is the renderer's when GameDesc.render leaves antialias out; a running renderer keeps its method.
   */
  readonly antialias: Antialias | null;
}

/**
 * 2D lighting of the sprite layers minLayer to maxLayer (sprites, 2D particles and og.Tilemap layers there, a map layer unless its `lit` property is false): their colour is multiplied by ambient plus every Light2D reaching the pixel, shadowed by Occluder2Ds and sprites with castShadow. Other layers (and the overlay) are drawn as authored. Off by default.
 *
 * The `Lighting2D` resource (token `og.Lighting2D`).
 */
export interface Lighting2D {
  /** Light the layers. */
  enabled: boolean;
  /**
   * Light everywhere (0 is darkness outside the lights).
   *
   * @default { r: 0.15, g: 0.15, b: 0.15 }
   */
  ambient: Color;
  /**
   * The lowest lit sprite layer.
   *
   * @default -1000000
   */
  minLayer: number;
  /**
   * The highest lit sprite layer (above it: UI and HUD layers stay unlit).
   *
   * @default 99
   */
  maxLayer: number;
  /**
   * The light map's resolution as a fraction of the target.
   *
   * @default 0.5
   * @range 0.25..1
   */
  resolution: number;
}

/**
 * A light in the 2D plane at the entity's position (z ignored): color times intensity at its centre, falling to zero at radius with (1 - d/r)^falloff, shadowed by occluders when castShadows. Lighting2D.enabled turns 2D lighting on; at most 64 are drawn.
 *
 * The value of the `Light2D` component (token `og.Light2D`).
 */
export interface Light2D {
  /**
   * The light's linear colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  color: Color;
  /**
   * The brightness at the centre.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  intensity: number;
  /**
   * Where the light reaches zero, in world units.
   *
   * @default 5
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  radius: number;
  /**
   * The exponent of the fade.
   *
   * @default 2
   * @range 0.1..
   */
  falloff: number;
  /**
   * The light's own size: larger gives softer shadows.
   *
   * @default 0.25
   * @range 0..
   * @unit m (metres)
   */
  sourceRadius: number;
  /**
   * Occluders between the light and a pixel shade it.
   *
   * @default true
   */
  castShadows: boolean;
}

/**
 * A rectangle that blocks 2D light (walls, crates, characters), centred on the entity's position and turned with it about z. Pixels inside it are not shadowed by it. At most 128 are drawn.
 *
 * The value of the `Occluder2D` component (token `og.Occluder2D`).
 */
export interface Occluder2D {
  /**
   * Its width in world units.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  width: number;
  /**
   * Its height in world units.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  height: number;
}

/**
 * A projected decal (scorch marks, blood, posters, bullet holes): the entity's unit box (-0.5 to 0.5 on each axis, scaled by its Transform) projected along its local -Z onto every lit opaque surface inside it, shaded with the surface. Its image is the uvMin to uvMax region of RenderSettings.decalAtlas (a solid colour without one). At most 32 are drawn.
 *
 * The value of the `Decal` component (token `og.Decal`).
 */
export interface Decal {
  /**
   * A linear colour multiplied with the atlas.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  color: Color;
  /**
   * The decal's opacity.
   *
   * @default 1
   * @range 0..1
   * @tunable
   */
  opacity: number;
  /** The atlas region's top-left corner, v down. */
  uvMin: Vector2;
  /**
   * The atlas region's bottom-right corner, v down.
   *
   * @default { x: 1, y: 1 }
   */
  uvMax: Vector2;
  /** Linear light added where the decal covers (glowing runes, lava). */
  emissive: Color;
  /**
   * The surface roughness under the decal; -1 keeps the surface's.
   *
   * @default -1
   * @range -1..1
   */
  roughness: number;
  /**
   * Surfaces turned further than this from the projection direction are not covered (the last 15 degrees fade).
   *
   * @default 75
   * @range 0..90
   * @unit deg (degrees)
   */
  angleFade: number;
  /**
   * Whether it is drawn.
   *
   * @default true
   */
  visible: boolean;
}

/**
 * ParticleSystem.startTick's default: never written. A looping emitter emits as if started at tick 0; a one-shot (`looping false`) emits nothing until its startTick is written, so a burst spawned in `start` waits for its first use. Write it back to re-arm a one-shot without firing it.
 */
export declare const PARTICLES_ARMED: 4294967295;

/**
 * Emits particles from an og.load.emitter asset at the entity's MatrixWorld. Particles are simulated by the renderer from (seed, startTick, the tick), so they are deterministic and replay, and never feed back into the simulation: compute on the full tier, the CPU on WebGL2, identical either way.
 *
 * The value of the `ParticleSystem` component (token `og.ParticleSystem`).
 */
export interface ParticleSystem {
  /** The emitter asset; 0 draws nothing. */
  emitter: ParticleEmitter;
  /** The random seed: systems with different seeds differ. */
  seed: number;
  /**
   * The tick emission started: write the tick (`time.tick`, `world.tick`) to fire a one-shot burst, or to restart one. Its default, PARTICLES_ARMED, is never written: a looping emitter emits as if started at tick 0, while a one-shot (`looping false`) stays armed and emits nothing until a tick is written, so effects spawned ahead of their first use wait.
   *
   * @default 4294967295
   */
  startTick: number;
  /**
   * The tick emission stops (the particles live out); 0 keeps emitting while the emitter loops.
   */
  stopTick: number;
  /**
   * Whether it is drawn (a hidden system loses its particles).
   *
   * @default true
   */
  visible: boolean;
  /**
   * Three.js layers, as Mesh.layers: its particles (billboards and 2D sprite particles alike) are drawn by the cameras whose masks share a bit with it (og.Camera, og.CameraView, og.Mirror).
   *
   * @default 1
   */
  layers: number;
}

/**
 * A streak behind the entity: its world positions over the last `lifetime` seconds as a camera-facing strip, from widthStart and color at the head to widthEnd and colorEnd at the tail.
 *
 * The value of the `Trail` component (token `og.Trail`).
 */
export interface Trail {
  /**
   * How long a position stays in the trail.
   *
   * @default 0.4
   * @range 0..
   * @unit s (seconds of simulation time)
   * @tunable
   */
  lifetime: number;
  /**
   * A new point is kept when the head has moved this far.
   *
   * @default 0.05
   * @range 0..
   * @unit m (metres)
   */
  minDistance: number;
  /**
   * The width at the head.
   *
   * @default 0.15
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  widthStart: number;
  /**
   * The width at the tail.
   *
   * @range 0..
   * @unit m (metres)
   */
  widthEnd: number;
  /**
   * The linear colour at the head; above 1 blooms.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  color: Color;
  /**
   * The alpha at the head.
   *
   * @default 1
   * @range 0..1
   */
  opacity: number;
  /**
   * The linear colour at the tail.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  colorEnd: Color;
  /**
   * The alpha at the tail.
   *
   * @range 0..1
   */
  opacityEnd: number;
  /** A texture along the strip; 0 is solid. */
  texture: Texture;
  /**
   * The texture repeats every this many world units; 0 stretches it once over the length.
   *
   * @range 0..
   * @unit m (metres)
   */
  uvTile: number;
  /** Additive or alpha blended. */
  blend: StripBlend;
  /**
   * Draw it in the 2D sprite order at `layer` (a sprite's trail over a tile background) instead of with the 3D scene.
   */
  layered: boolean;
  /** With layered: the sprite layer it is drawn in, before that layer's sprites. */
  layer: number;
}

/**
 * A strip from the entity's position to `end` whose texture scrolls with the tick: lasers, tethers, lightning (jitter).
 *
 * The value of the `Beam` component (token `og.Beam`).
 */
export interface Beam {
  /**
   * The far end, in world space.
   *
   * @unit m (metres)
   */
  end: Vector3;
  /**
   * The beam's width.
   *
   * @default 0.1
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  width: number;
  /**
   * The linear colour; above 1 blooms.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  color: Color;
  /**
   * The alpha.
   *
   * @default 1
   * @range 0..1
   */
  opacity: number;
  /** A texture along the beam; 0 is solid. */
  texture: Texture;
  /**
   * Texture repeats per world unit along the beam.
   *
   * @default 1
   * @range 0..
   */
  tile: number;
  /**
   * Texture scroll in repeats per second.
   *
   * @tunable
   */
  scroll: number;
  /**
   * Segments: 1 is straight, more let jitter bend it.
   *
   * @default 1
   * @range 1..64
   */
  segments: number;
  /**
   * Sideways jitter in world units, new every tick (lightning).
   *
   * @range 0..
   * @unit m (metres)
   */
  jitter: number;
  /** The jitter's seed. */
  seed: number;
  /** Additive or alpha blended. */
  blend: StripBlend;
}

/**
 * One point of a ribbon.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RibbonPoint {
  /**
   * Where it is.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly position?: Vector3;
  /**
   * The width there.
   *
   * @default 0.1
   * @range 0..
   * @unit m (metres)
   */
  readonly width?: number;
  /**
   * The linear colour there.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  readonly color?: Color;
  /**
   * The alpha there.
   *
   * @default 1
   * @range 0..1
   */
  readonly opacity?: number;
}

/**
 * An explicit camera-facing strip through points, drawn this frame only (immediate mode, as og.render.sprite): submit it every tick it should show.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RibbonDesc {
  /**
   * The points, in order (at least two).
   *
   * @default []
   */
  readonly points?: readonly RibbonPoint[];
  /**
   * A texture along the strip; null is solid.
   *
   * @default null
   */
  readonly texture?: Texture | null;
  /**
   * The texture repeats every this many world units; 0 stretches it once.
   *
   * @default 0
   * @range 0..
   * @unit m (metres)
   */
  readonly uvTile?: number;
  /**
   * Additive or alpha blended.
   *
   * @default "additive"
   */
  readonly blend?: StripBlend;
}


/**
 * A line or block of MSDF text in the world's xy plane facing +z, drawn this frame only (immediate mode: submit it every tick it should show).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TextDesc {
  /**
   * The font from og.load.font; null draws nothing.
   *
   * @default null
   */
  readonly font?: Font | null;
  /**
   * The text; `\n` starts a new line.
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * The baseline origin of the first line.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly position?: Vector3;
  /**
   * The em size in world units.
   *
   * @default 1
   * @range 0..
   */
  readonly size?: number;
  /**
   * Rotation about +z.
   *
   * @default 0
   * @unit rad (radians)
   */
  readonly rotation?: number;
  /**
   * The linear colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  readonly color?: Color;
  /**
   * The alpha.
   *
   * @default 1
   * @range 0..1
   */
  readonly opacity?: number;
  /**
   * Draw order among sprites and text; higher draws on top.
   *
   * @default 0
   */
  readonly layer?: number;
  /**
   * How each line sits on `position`: start puts its left end there, center its middle, end its right end.
   *
   * @default "start"
   */
  readonly align?: TextAlign;
  /**
   * World text: an outline around the glyphs, as a fraction (0 to 1) of the widest the font's distance range allows (a font cooked with a larger px_range takes thicker outlines).
   *
   * @default 0
   * @range 0..1
   */
  readonly outline?: number;
  /**
   * The outline's linear colour.
   *
   * @default { r: 0, g: 0, b: 0 }
   */
  readonly outlineColor?: Color;
  /**
   * Draw in screen space over the finished frame: position is the first baseline's origin in CSS pixels from the top left, y down, size in CSS pixels, rotation clockwise. With og.ui: under every UI element at a layer up to UI_LAYER (1000), over every one above it.
   *
   * @default false
   */
  readonly screen?: boolean;
  /**
   * Screen text: cut to [x, y, width, height] in CSS pixels, per pixel; null draws it whole.
   *
   * @default null
   */
  readonly clip?: [number, number, number, number] | null;
}

/**
 * Text to measure with og.render.measureText, laid out as og.render.text lays it out.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TextMeasureDesc {
  /**
   * The font from og.load.font.
   *
   * @default null
   */
  readonly font?: Font | null;
  /**
   * The text; `\n` starts a new line.
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * The em size in world units.
   *
   * @default 1
   * @range 0..
   */
  readonly size?: number;
}

/** The extent of a laid-out text, in world units. */
export interface TextMetrics {
  /** The widest line's advance. */
  readonly width: number;
  /** Lines times the font's line height. */
  readonly height: number;
  /** How many lines. */
  readonly lines: number;
}

/**
 * A textured or solid quad in the world's xy plane facing +z, drawn this frame only (immediate mode, as og.render.text): submit it every tick it should show. Sprites and text are sorted by layer and drawn over the 3D scene.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SpriteDesc {
  /**
   * The texture from og.load.texture; null draws a solid quad of the colour.
   *
   * @default null
   */
  readonly texture?: Texture | null;
  /**
   * Draw from a layer of this texture array (og.load.textureArray) instead of `texture`: every sprite of one array is one draw whichever layer each shows, so a flipbook's frames packed as layers batch across frames. World and screen sprites alike, with flash, glow, blend and 2D lighting as a textured sprite. null uses `texture`.
   *
   * @default null
   */
  readonly array?: TextureArray | null;
  /**
   * With `array`: the layer drawn, 0 to the array's layers minus 1 (refused past it). `uv` is the region within the layer.
   *
   * @default 0
   */
  readonly arrayLayer?: number;
  /**
   * Where the anchor point is.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly position?: Vector3;
  /**
   * Its width in world units.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly width?: number;
  /**
   * Its height in world units.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly height?: number;
  /**
   * The point of the quad at `position`, as a fraction of its size from the bottom-left corner: (0.5, 0.5) is the centre.
   *
   * @default { x: 0.5, y: 0.5 }
   */
  readonly anchor?: Vector2;
  /**
   * Counter-clockwise rotation about the anchor.
   *
   * @default 0
   * @unit rad (radians)
   */
  readonly rotation?: number;
  /**
   * A linear colour multiplied with the texture.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  readonly color?: Color;
  /**
   * The alpha.
   *
   * @default 1
   * @range 0..1
   */
  readonly opacity?: number;
  /**
   * The texture region drawn, [u0, v0, u1, v1] with v down; null is the whole texture (a sheet's frame for animation).
   *
   * @default null
   */
  readonly uv?: [number, number, number, number] | null;
  /**
   * Draw order among sprites and text; higher draws on top.
   *
   * @default 0
   */
  readonly layer?: number;
  /**
   * A hit flash: the colour mixed towards flashColor by this much, keeping the alpha.
   *
   * @default 0
   * @range 0..1
   */
  readonly flash?: number;
  /**
   * The flash's linear colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  readonly flashColor?: Color;
  /**
   * Strength in the 2D glow buffer (PostEffects.glow); 0 none.
   *
   * @default 0
   * @range 0..
   */
  readonly glow?: number;
  /**
   * Alpha or additive.
   *
   * @default "alpha"
   */
  readonly blend?: SpriteBlend;
  /**
   * Block 2D light (Lighting2D) with the sprite's rectangle.
   *
   * @default false
   */
  readonly castShadow?: boolean;
  /**
   * A drop shadow this dark (0 none, 0.5 typical): the sprite's shape in black, flattened along the sun (the first og.DirectionalLight) onto the ground under it (dropShadowHeight, or the og.Terrain there), drawn before every sprite of its layer: pixel sprites standing in a 3D set cast shadows that agree with it. Where the ground already lies in the sun's cascaded shadow the drop shadow fades, so a shadow inside a shadow is no darker. None with the sun below the horizon; at most five times the sprite's height long under a low one. World sprites with a perspective camera over the xz ground; a 2D camera sees the ground edge on.
   *
   * @default 0
   * @range 0..1
   */
  readonly dropShadow?: number;
  /**
   * The height of the level ground the drop shadow lies on (a floor, a deck, a step). Where an og.Terrain under the sprite's anchor is higher, the shadow lies on the terrain instead, on the plane fitted to its slope under the shadow, or bent over a hollow or a ridge narrower than the shadow (the terrain's heights under a 4 x 2 grid of the shadow's cells). 0 with no terrain is the ground plane y = 0.
   *
   * @default 0
   * @unit m (metres)
   */
  readonly dropShadowHeight?: number;
  /**
   * As three.js Material.toneMapped: true draws the sprite into the lit scene, carried by exposure (auto exposure included), bloom and the tone curve like any surface. false keeps its colours as given: still placed in the world, depth tested and sorted with the other sprites, but shown unexposed and uncurved (clamped to 0 to 1), outside bloom and fog; the grade, vignette, grain and screen flash still apply. For world UI that must read the same at noon and at dusk: speech bubbles, name tags, markers. Alpha blending only: an additive sprite adds light and is always tone mapped.
   *
   * @default true
   */
  readonly toneMapped?: boolean;
  /**
   * Mirror the region left to right.
   *
   * @default false
   */
  readonly flipX?: boolean;
  /**
   * Mirror the region top to bottom.
   *
   * @default false
   */
  readonly flipY?: boolean;
  /**
   * Within its layer, draw by y: lower on the screen draws in front (top-down games).
   *
   * @default false
   */
  readonly sortY?: boolean;
  /**
   * Nine-slice: [left, top, right, bottom] insets in texture pixels of the region; the corners keep their size (sliceScale world units per pixel), edges and centre stretch. null draws it whole.
   *
   * @default null
   */
  readonly slice?: [number, number, number, number] | null;
  /**
   * Nine-slice: world units (CSS pixels on screen) per texture pixel of the insets.
   *
   * @default 0.0625
   * @range 0..
   */
  readonly sliceScale?: number;
  /**
   * Draw in screen space over the finished frame instead of the world: position x, y in CSS pixels from the top left with y down, width and height in CSS pixels, anchor from the top left, rotation clockwise; never tone mapped, blurred or depth tested (a HUD). flash, glow, blend and castShadow apply to world sprites only. With og.ui: under every UI element at a layer up to UI_LAYER (1000), over every one above it.
   *
   * @default false
   */
  readonly screen?: boolean;
  /**
   * Screen sprites: cut to [x, y, width, height] in CSS pixels; null draws it whole.
   *
   * @default null
   */
  readonly clip?: [number, number, number, number] | null;
}


/**
 * A sprite drawn every frame at the entity's MatrixWorld (its xy and z, its rotation about z; its scale multiplies width and height, so squash and stretch is a tween of og.Transform.scale). With an og.Flipbook on the entity, the texture (or an `array true` sheet's texture array layer), region and anchor come from the flipbook's current frame.
 * Sorted with the immediate sprites and text by layer, then (sortY) by y, then texture: one draw per run of one texture. Tile layers draw before the sprites of their layer.
 *
 * The value of the `Sprite` component (token `og.Sprite`).
 */
export interface Sprite {
  /** The texture (og.load.texture); 0 draws a solid quad of the colour. */
  texture: Texture;
  /**
   * Draw from a layer of this texture array (og.load.textureArray) instead of `texture`; 0 uses `texture`. Every sprite of one array is one draw whichever layer each shows. While the array's streaming group is not resident the sprite draws nothing.
   */
  array: TextureArray;
  /**
   * With `array`: the layer drawn (past the last layer, the last). uvMin and uvMax are the region within the layer; a nine-slice measures the layer.
   */
  arrayLayer: number;
  /**
   * Its width in world units (times Transform.scale.x).
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  width: number;
  /**
   * Its height in world units (times Transform.scale.y).
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  height: number;
  /**
   * The point of the quad at the entity's position, as a fraction from the bottom-left corner: (0.5, 0) stands it on its feet. A flipbook frame's pivot replaces it.
   *
   * @default { x: 0.5, y: 0.5 }
   */
  anchor: Vector2;
  /**
   * A linear colour multiplied with the texture.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  color: Color;
  /**
   * The alpha.
   *
   * @default 1
   * @range 0..1
   * @tunable
   */
  opacity: number;
  /** The texture region's top-left corner, v down. */
  uvMin: Vector2;
  /**
   * The texture region's bottom-right corner, v down.
   *
   * @default { x: 1, y: 1 }
   */
  uvMax: Vector2;
  /** Draw order among sprites, tiles and text; higher draws on top. */
  layer: number;
  /**
   * Within its layer, lower y draws in front: top-down characters, props and trees overlap by their feet.
   */
  sortY: boolean;
  /** Mirror left to right (a character facing left). */
  flipX: boolean;
  /** Mirror top to bottom. */
  flipY: boolean;
  /**
   * A hit flash: the colour mixed towards flashColor by this much (og.tween.flash fades it out).
   *
   * @range 0..1
   * @tunable
   */
  flash: number;
  /**
   * The flash's linear colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  flashColor: Color;
  /**
   * Strength in the 2D glow buffer (PostEffects.glow); 0 none.
   *
   * @range 0..
   */
  glow: number;
  /** Alpha or additive. */
  blend: SpriteBlend;
  /** Block 2D light (Lighting2D) with the sprite's rectangle. */
  castShadow: boolean;
  /**
   * Nine-slice: the left and right insets in texture pixels of the region (0 0 with sliceY 0 0 draws it whole). World panels, health bars, speech bubbles.
   */
  sliceX: Vector2;
  /** Nine-slice: the top and bottom insets in texture pixels. */
  sliceY: Vector2;
  /**
   * Nine-slice: world units per texture pixel of the insets.
   *
   * @default 0.0625
   * @range 0..
   */
  sliceScale: number;
  /**
   * A soft blob shadow on the ground under the sprite's anchor, this dark (0 none, 0.4 typical), drawn under every sprite of its layer.
   *
   * @range 0..1
   * @tunable
   */
  shadow: number;
  /**
   * The blob's width and height; 0 0 is 0.8 by 0.3 of the sprite's width.
   *
   * @range 0..
   * @unit m (metres)
   */
  shadowSize: Vector2;
  /**
   * The blob's centre relative to the anchor.
   *
   * @unit m (metres)
   */
  shadowOffset: Vector2;
  /**
   * A drop shadow this dark (0 none, 0.5 typical): the sprite's shape flattened along the sun onto the ground under it (dropShadowHeight, or the og.Terrain there), faded where that ground already lies in the sun's shadow, drawn before every sprite of its layer (SpriteDesc.dropShadow): pixel sprites standing in a 3D set.
   *
   * @range 0..1
   * @tunable
   */
  dropShadow: number;
  /**
   * The height of the level ground the drop shadow lies on; where an og.Terrain under the anchor is higher, the terrain (SpriteDesc.dropShadowHeight).
   *
   * @unit m (metres)
   */
  dropShadowHeight: number;
  /**
   * As three.js Material.toneMapped: false keeps the sprite's colours as given, outside exposure, bloom, fog and the tone curve, still placed and depth tested in the world (SpriteDesc.toneMapped): speech bubbles and markers that must not blow out at dusk. Alpha blending only.
   *
   * @default true
   */
  toneMapped: boolean;
  /**
   * Whether it is drawn.
   *
   * @default true
   */
  visible: boolean;
  /**
   * Three.js layers, as Mesh.layers: a world sprite is drawn by the cameras whose masks share a bit with it (a screen sprite draws on every camera's frame).
   *
   * @default 1
   */
  layers: number;
}

/**
 * Ghosts of the entity's og.Sprite: every `interval` ticks its pose (position, rotation, scale, frame, flips) is kept, and the last `count` are drawn behind it, fading from `opacity` to 0, tinted by color. A dash, a speed-up, a teleport. The history is world state (the og.sprite state block).
 *
 * The value of the `Afterimage` component (token `og.Afterimage`).
 */
export interface Afterimage {
  /**
   * How many ghosts.
   *
   * @default 4
   * @range 0..16
   */
  count: number;
  /**
   * Ticks between ghosts.
   *
   * @default 3
   * @range 1..
   * @unit ticks (fixed simulation steps)
   */
  interval: number;
  /**
   * The newest ghost's alpha.
   *
   * @default 0.5
   * @range 0..1
   * @tunable
   */
  opacity: number;
  /**
   * A linear tint (above 1 with additive, a glow).
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  color: Color;
  /** Add the ghosts instead of blending them. */
  additive: boolean;
  /**
   * Record new ghosts; false lets the existing ones fade out.
   *
   * @default true
   */
  active: boolean;
}

/**
 * A damage number or pickup text that pops at a point, rises, spreads and fades by itself (no per-tick submission). Deterministic: its drift comes from the tick and its order, and it is world state (the og.sprite state block).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface FloatingTextDesc {
  /**
   * The text, such as `-12` or `+1 key`.
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * Where it appears (above a hit, usually).
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly position?: Vector3;
  /**
   * The font; null uses the UI's body font.
   *
   * @default null
   */
  readonly font?: Font | null;
  /**
   * The em size in world units.
   *
   * @default 0.5
   * @range 0..
   * @unit m (metres)
   */
  readonly size?: number;
  /**
   * The linear fill colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  readonly color?: Color;
  /**
   * The outline's width as a fraction of the font's distance range (0 none).
   *
   * @default 0.6
   * @range 0..1
   */
  readonly outline?: number;
  /**
   * The outline's linear colour.
   *
   * @default { r: 0.02, g: 0.02, b: 0.03 }
   */
  readonly outlineColor?: Color;
  /**
   * How far it climbs over its life.
   *
   * @default 1
   * @unit m (metres)
   */
  readonly rise?: number;
  /**
   * The largest sideways drift, left or right.
   *
   * @default 0.3
   * @range 0..
   * @unit m (metres)
   */
  readonly spread?: number;
  /**
   * How long it shows.
   *
   * @default 0.8
   * @range 0.05..
   * @unit s (seconds of simulation time)
   */
  readonly lifetime?: number;
  /**
   * The last part of its life it fades over.
   *
   * @default 0.4
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly fade?: number;
  /**
   * How much bigger it starts, settling to its size in the first fifth of its life.
   *
   * @default 0.4
   * @range 0..
   */
  readonly pop?: number;
  /**
   * A critical hit: critScale times bigger, a stronger pop.
   *
   * @default false
   */
  readonly crit?: boolean;
  /**
   * The size of a crit.
   *
   * @default 1.6
   * @range 1..
   */
  readonly critScale?: number;
  /**
   * The sprite layer (above the action).
   *
   * @default 100
   */
  readonly layer?: number;
}



/**
 * A box centred at the origin, as three.js BoxGeometry(width, height, depth).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface BoxGeometryDesc {
  /**
   * Its extent along x.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly width?: number;
  /**
   * Its extent along y.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly height?: number;
  /**
   * Its extent along z.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly depth?: number;
  /**
   * How many times a texture repeats across it, along u and v: its UVs run 0 to uvRepeat instead of 0 to 1 (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). A ground grid every 10 m on a 1,600 m plane is uvRepeat 160 160.
   *
   * @default { x: 1, y: 1 }
   */
  readonly uvRepeat?: Vector2;
}

/**
 * A rectangle in the xy plane facing +z, centred at the origin, as three.js PlaneGeometry(width, height); turn it -pi/2 about x for a floor.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PlaneGeometryDesc {
  /**
   * Its extent along x.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly width?: number;
  /**
   * Its extent along y.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly height?: number;
  /**
   * How many times a texture repeats across it, along u and v: its UVs run 0 to uvRepeat instead of 0 to 1 (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). A ground grid every 10 m on a 1,600 m plane is uvRepeat 160 160.
   *
   * @default { x: 1, y: 1 }
   */
  readonly uvRepeat?: Vector2;
}

/**
 * A UV sphere centred at the origin, as three.js SphereGeometry(radius, widthSegments, heightSegments).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SphereGeometryDesc {
  /**
   * Its radius.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly radius?: number;
  /**
   * Segments around the equator.
   *
   * @default 32
   * @range 3..
   */
  readonly widthSegments?: number;
  /**
   * Rings from pole to pole.
   *
   * @default 16
   * @range 2..
   */
  readonly heightSegments?: number;
  /**
   * How many times a texture repeats across it, along u and v: its UVs run 0 to uvRepeat instead of 0 to 1 (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). A ground grid every 10 m on a 1,600 m plane is uvRepeat 160 160.
   *
   * @default { x: 1, y: 1 }
   */
  readonly uvRepeat?: Vector2;
}




/**
 * The most point and spot lights one frame draws, among those in view; past it the ones brightest at the camera are kept.
 */
export declare const MAX_LIGHTS: 256;

/**
 * The most point and spot lights that shade one screen cluster (a 1/16 by 1/9 tile of the view and one of 24 depth slices); past it the ones brightest at the camera are kept there.
 */
export declare const MAX_CLUSTER_LIGHTS: 64;

/**
 * The UI's layer among screen sprites and screen text (SpriteDesc and TextDesc with screen): those at or below it draw under every og.ui element, those above it over every one, modals, toasts and the screen transition included (a HUD element made of sprites sitting on a UI panel: draw it only while its screen shows), in their layers' order either way.
 */
export declare const UI_LAYER: 1000;

/**
 * What the renderer can do on this device. Backend limits are reported here, never by missing members.
 */
export interface RenderCapabilities {
  /** The backend. */
  readonly tier: RenderTier;
  /** Whether compute passes run (GPU culling, GPU particles). */
  readonly compute: boolean;
  /** The antialiasing the device resolved RenderOptions.antialias to. */
  readonly antialias: Antialias;
  /** How many point and spot lights one frame draws (MAX_LIGHTS on every tier). */
  readonly maxLights: number;
  /**
   * How many point and spot lights cast shadows in one frame (8 on the full tier, 4 on WebGL2); past it, the shadowed lights brightest at the camera keep theirs.
   */
  readonly maxShadowedLights: number;
  /**
   * How point lights' shadows are stored on this device: cube or paraboloid (what RenderOptions.pointShadows resolved to).
   */
  readonly pointShadows: PointShadows;
  /** The render target width in device pixels. */
  readonly width: number;
  /** The render target height in device pixels. */
  readonly height: number;
  /**
   * The Stylise and MaterialStyle primitives this renderer draws, by group: pixel, posterize, palette, dither, halftone, hatching, paint, ink, inkDepth, depthBands, retro3d, surfaceSpace (paintCanvasSpace, hatchingSpace and halftoneSpace surface) and objectSpace (the same object) of Stylise; toon, shadowTint, rim, outline, outlineDistance and flatShading of MaterialStyle. A look that uses one this list lacks still applies; that primitive is not drawn.
   */
  readonly stylise: readonly string[];
  /**
   * Whether PostEffects.ssr draws: true on the full tier, false on WebGL2, where smooth surfaces reflect the reflection probes and the environment instead.
   */
  readonly ssr: boolean;
  /**
   * How many og.ReflectionProbe components one frame samples (MAX_REFLECTION_PROBES on every tier).
   */
  readonly reflectionProbes: number;
  /**
   * Whether a render target made with `depth` gets its view's depth drawn: true on every tier with a renderer (WebGPU, Vulkan, Metal and WebGL2 alike), false without one. The depth comes from the view's own depth prepass of its opaque meshes into a linear-depth colour target, never from its depth buffer, which WebGL2 cannot read in a shader.
   */
  readonly renderTargetDepth: boolean;
  /** The largest side of a render target (the device's largest 2D texture). */
  readonly maxRenderTargetSize: number;
  /**
   * The most custom post passes a frame runs: 8 on every tier, PostEffects.custom1 to custom8 (numbered fields, as a reflected record has no list field).
   */
  readonly maxCustomPosts: number;
  /**
   * How Fog.volumetric is drawn on this device: a froxel grid (the full tier) or a raymarch along each pixel's view ray (WebGL2), each lit by the sun through its shadows and by the point, spot and rect lights around each sample (up to volumetricLights of them) through their shadows and cookies.
   */
  readonly volumetricFog: VolumetricFog;
  /**
   * The most point, spot and rect lights that light one sample of Fog.volumetric: those of its screen cluster, MAX_CLUSTER_LIGHTS (64) in the froxel grid, 8 in WebGL2's raymarch, the brightest at the camera first.
   */
  readonly volumetricLights: number;
  /**
   * The most light cookie layers one frame draws (24 on every tier): a SpotLight.cookie takes one, a PointLight.cookie six (its cube's faces), and a texture several lights of one kind share takes its layers once. Cookies past it, in the order of their lights' entities, are not drawn: those lights shine without them, and the log names them in a warning.
   */
  readonly maxLightCookies: number;
}

/**
 * The most og.ReflectionProbe components one frame samples, nearest the camera first; the rest are skipped.
 */
export declare const MAX_REFLECTION_PROBES: 8;

/**
 * When a reflection probe's cube is rendered.
 * - `"baked"`: Once: read from the project's cook cache when `og bake probes` wrote it there, otherwise rendered on its first frame and kept. Static rooms.
 * - `"realtime"`: Again every `interval` ticks, so moving things show in it (a cost per update of six faces).
 */
export type ReflectionProbeMode = "baked" | "realtime";

/**
 * A ReflectionProbeMode field: the variant's index.
 *
 * A `ReflectionProbeMode` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function ReflectionProbeMode(value?: ReflectionProbeMode, options?: FieldOptions): FieldSpec<"ReflectionProbeMode">;

/**
 * A local reflection: a cube of what is around the entity's position, sampled by every lit surface inside its box with box projection (the reflected ray meets the box, so a room's walls reflect where they are, not at infinity) and blended with the environment map over blendDistance at the box's faces. Interiors that the sky must not light: a chrome sphere in a room shows the room. Overlapping probes: the smallest wins. The cube is rendered from the probe's position without it (nothing is drawn twice) and without the probes' own reflections.
 *
 * The value of the `ReflectionProbe` component (token `og.ReflectionProbe`).
 */
export interface ReflectionProbe {
  /**
   * The box's full extents in world units, centred on the entity (its rotation is ignored: the box is axis-aligned).
   *
   * @default { x: 10, y: 10, z: 10 }
   */
  size: Vector3;
  /**
   * The side of each cube face in texels; it is rounded to a power of two.
   *
   * @default 128
   * @range 16..1024
   */
  resolution: number;
  /**
   * baked (rendered once, or loaded from `og bake probes`) or realtime (every interval ticks).
   */
  mode: ReflectionProbeMode;
  /**
   * With realtime: ticks between updates (1 is every frame).
   *
   * @default 30
   * @range 1..
   * @unit ticks (fixed simulation steps)
   */
  interval: number;
  /**
   * A multiplier on the probe's light.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  intensity: number;
  /**
   * The distance inside the box's faces over which the probe fades into the environment map.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  blendDistance: number;
  /**
   * Whether it is sampled.
   *
   * @default true
   */
  visible: boolean;
}



/**
 * A material shader from og.load.shader (an `.ogt` of `type shader`): WGSL hooks of the lit shader, or with `mode full` its own vertex and fragment entries, and the parameters they read.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Shader = number & { readonly [handleType]: "Shader" };








/**
 * Geometry from attribute arrays, as three.js BufferGeometry with its attributes set: one item per vertex in each array, triangles three indices each (or, with no indices, every three vertices).
 * Missing normals are computed, as three.js computeVertexNormals (each vertex the area-weighted sum of its triangles' normals: shared vertices shade smooth, an unindexed geometry flat); missing tangents are generated when a normal map needs them. An unindexed geometry whose vertex count is not a multiple of three has no triangles: it draws nothing as an og.Mesh and is a vertex set for og.Lines and og.Points.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface BufferGeometryDesc {
  /** The vertex positions, x y z per vertex (three.js `position`); at least one vertex. */
  readonly positions?: Float32Array;
  /**
   * Unit normals, x y z per vertex; null computes them.
   *
   * @default null
   */
  readonly normals?: Float32Array | null;
  /**
   * Texture coordinates, u v per vertex, v down the image (glTF's, as og-render's); null is 0 0.
   *
   * @default null
   */
  readonly uvs?: Float32Array | null;
  /**
   * Linear vertex colours, r g b or r g b a per vertex, multiplied into the material's colour (three.js `color` with vertexColors); null is white.
   *
   * @default null
   */
  readonly colors?: Float32Array | null;
  /**
   * Tangents, x y z w per vertex (glTF's: w the bitangent's sign); null generates them.
   *
   * @default null
   */
  readonly tangents?: Float32Array | null;
  /**
   * Triangles, three vertex indices each, counter-clockwise seen from the front; null takes every three vertices as one.
   *
   * @default null
   */
  readonly indices?: Uint32Array | null;
  /**
   * Whether og.render.updateGeometry may rewrite it (a mesh made each tick: water, a deforming terrain, a trail of geometry). A static geometry is refused there.
   *
   * @default false
   */
  readonly dynamic?: boolean;
  /**
   * Register it as a physics mesh too, so an og.Collider with it as `mesh` (SHAPE_MESH or SHAPE_CONVEX) collides with what is drawn; equivalent to og.physics.setMesh with its positions and indices. An update re-registers it.
   *
   * @default true
   */
  readonly physics?: boolean;
}

/** What a geometry holds (og.render.geometryInfo). */
export interface GeometryInfo {
  /** Vertices. */
  readonly vertices: number;
  /** Triangles. */
  readonly triangles: number;
  /**
   * The bounds' least corner in the geometry's own space, as three.js boundingBox.min.
   *
   * @unit m (metres)
   */
  readonly min: Vector3;
  /**
   * The bounds' greatest corner.
   *
   * @unit m (metres)
   */
  readonly max: Vector3;
  /** Whether og.render.updateGeometry may rewrite it. */
  readonly dynamic: boolean;
}




/**
 * A cylinder (or a truncated cone) along +Y centred at the origin, as three.js CylinderGeometry(radiusTop, radiusBottom, height, radialSegments, heightSegments, openEnded, thetaStart, thetaLength).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CylinderGeometryDesc {
  /**
   * The radius at the top.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly radiusTop?: number;
  /**
   * The radius at the bottom.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly radiusBottom?: number;
  /**
   * Its extent along y.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly height?: number;
  /**
   * Segments around.
   *
   * @default 32
   * @range 3..
   */
  readonly radialSegments?: number;
  /**
   * Rows along the height.
   *
   * @default 1
   * @range 1..
   */
  readonly heightSegments?: number;
  /**
   * Leave the ends open (no caps).
   *
   * @default false
   */
  readonly openEnded?: boolean;
  /**
   * Where the side starts around y, from +Z towards +X.
   *
   * @default 0
   * @unit rad (radians)
   */
  readonly thetaStart?: number;
  /**
   * How far around the side goes; 2 pi is whole.
   *
   * @default 6.283185307179586
   * @unit rad (radians)
   */
  readonly thetaLength?: number;
}

/**
 * A cone along +Y centred at the origin, its tip up, as three.js ConeGeometry(radius, height, radialSegments, heightSegments, openEnded).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ConeGeometryDesc {
  /**
   * The base's radius.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly radius?: number;
  /**
   * Its extent along y.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly height?: number;
  /**
   * Segments around.
   *
   * @default 32
   * @range 3..
   */
  readonly radialSegments?: number;
  /**
   * Rows along the height.
   *
   * @default 1
   * @range 1..
   */
  readonly heightSegments?: number;
  /**
   * Leave the base open.
   *
   * @default false
   */
  readonly openEnded?: boolean;
}

/**
 * A capsule along +Y centred at the origin, as three.js CapsuleGeometry(radius, height, capSegments, radialSegments, heightSegments): a cylinder of `height` between two hemispheres, `height + 2 radius` long in all (an og.Collider capsule of halfHeight height / 2 and that radius matches it).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CapsuleGeometryDesc {
  /**
   * The radius.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly radius?: number;
  /**
   * The straight middle's length.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly height?: number;
  /**
   * Rows in each hemisphere.
   *
   * @default 4
   * @range 1..
   */
  readonly capSegments?: number;
  /**
   * Segments around.
   *
   * @default 8
   * @range 3..
   */
  readonly radialSegments?: number;
  /**
   * Rows along the middle.
   *
   * @default 1
   * @range 1..
   */
  readonly heightSegments?: number;
}

/**
 * A torus in the xy plane around +Z, as three.js TorusGeometry(radius, tube, radialSegments, tubularSegments, arc).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TorusGeometryDesc {
  /**
   * From the centre to the tube's centre.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly radius?: number;
  /**
   * The tube's radius.
   *
   * @default 0.4
   * @range 0..
   * @unit m (metres)
   */
  readonly tube?: number;
  /**
   * Segments around the tube.
   *
   * @default 12
   * @range 2..
   */
  readonly radialSegments?: number;
  /**
   * Segments around the ring.
   *
   * @default 48
   * @range 3..
   */
  readonly tubularSegments?: number;
  /**
   * How far around the ring goes.
   *
   * @default 6.283185307179586
   * @unit rad (radians)
   */
  readonly arc?: number;
}

/**
 * A flat ring in the xy plane facing +Z, as three.js RingGeometry(innerRadius, outerRadius, thetaSegments, phiSegments, thetaStart, thetaLength).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RingGeometryDesc {
  /**
   * The hole's radius.
   *
   * @default 0.5
   * @range 0..
   * @unit m (metres)
   */
  readonly innerRadius?: number;
  /**
   * The ring's radius.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly outerRadius?: number;
  /**
   * Segments around.
   *
   * @default 32
   * @range 3..
   */
  readonly thetaSegments?: number;
  /**
   * Rings from the inside out.
   *
   * @default 1
   * @range 1..
   */
  readonly phiSegments?: number;
  /**
   * Where it starts, from +X towards +Y.
   *
   * @default 0
   * @unit rad (radians)
   */
  readonly thetaStart?: number;
  /**
   * How far around it goes.
   *
   * @default 6.283185307179586
   * @unit rad (radians)
   */
  readonly thetaLength?: number;
}

/**
 * A disc (or a sector) in the xy plane facing +Z, as three.js CircleGeometry(radius, segments, thetaStart, thetaLength): a 2D polygon's simplest shape.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CircleGeometryDesc {
  /**
   * The radius.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly radius?: number;
  /**
   * Segments around.
   *
   * @default 32
   * @range 3..
   */
  readonly segments?: number;
  /**
   * Where it starts, from +X towards +Y.
   *
   * @default 0
   * @unit rad (radians)
   */
  readonly thetaStart?: number;
  /**
   * How far around it goes.
   *
   * @default 6.283185307179586
   * @unit rad (radians)
   */
  readonly thetaLength?: number;
}

/**
 * A tube along a curve through points, as three.js TubeGeometry(new CatmullRomCurve3(points, closed, 'centripetal'), tubularSegments, radius, radialSegments, closed): u runs along it, v around. The points may come from anywhere: a path's samples, a rope's points, a spline a tool made.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TubeGeometryDesc {
  /**
   * The points the curve passes through, in order (at least two).
   *
   * @default []
   * @unit m (metres)
   */
  readonly points?: readonly Vector3[];
  /**
   * Segments along the tube.
   *
   * @default 64
   * @range 1..
   */
  readonly tubularSegments?: number;
  /**
   * The tube's radius.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly radius?: number;
  /**
   * Segments around it.
   *
   * @default 8
   * @range 3..
   */
  readonly radialSegments?: number;
  /**
   * Join the last point back to the first.
   *
   * @default false
   */
  readonly closed?: boolean;
}








/**
 * How a line set's points join, as three.js's three line objects.
 * - `"strip"`: One polyline through every point, as three.js Line.
 * - `"segments"`: Pairs of points, each its own segment, as three.js LineSegments.
 * - `"loop"`: A polyline closed back to its first point, as three.js LineLoop.
 */
export type LineMode = "strip" | "segments" | "loop";

/**
 * A LineMode field: the variant's index.
 *
 * A `LineMode` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function LineMode(value?: LineMode, options?: FieldOptions): FieldSpec<"LineMode">;

/**
 * Lines through points, drawn this frame only (immediate mode, as og.render.sprite): submit them every tick they should show. three.js Line, LineSegments and LineLoop with a LineBasicMaterial or LineDashedMaterial, but any width on every tier (each segment is a quad in screen space with a feathered edge, so WebGL2's one-pixel lines do not apply).
 * World lines are drawn in the scene after the transparent meshes and particles, exposed, bloomed and tone mapped with it; screen lines over the finished frame, after every post effect and under the UI.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LinesDesc {
  /**
   * The points, in order: world positions, or with screen CSS pixels from the top left of the play surface with y down (z ignored).
   *
   * @default []
   * @unit m (metres)
   */
  readonly points?: readonly Vector3[];
  /**
   * Per point linear colours, blended along each segment (three.js vertexColors); empty is `color` for all.
   *
   * @default []
   */
  readonly colors?: readonly Color[];
  /**
   * The linear colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  readonly color?: Color;
  /**
   * The alpha.
   *
   * @default 1
   * @range 0..1
   */
  readonly opacity?: number;
  /**
   * How the points join.
   *
   * @default "strip"
   */
  readonly mode?: LineMode;
  /**
   * The width: CSS pixels, or world units with worldUnits.
   *
   * @default 1
   * @range 0..
   */
  readonly width?: number;
  /**
   * Measure width in world units (a line that thins with distance) instead of CSS pixels (as wide at any distance).
   *
   * @default false
   */
  readonly worldUnits?: boolean;
  /**
   * Hidden behind nearer geometry, as three.js Material.depthTest; false draws it over the scene. Screen lines are never depth tested.
   *
   * @default true
   */
  readonly depthTest?: boolean;
  /**
   * Draw in screen space over the finished frame: points in CSS pixels, width in CSS pixels.
   *
   * @default false
   */
  readonly screen?: boolean;
  /**
   * Dashes, as three.js LineDashedMaterial: dashSize on, gapSize off, along the line from its first point.
   *
   * @default false
   */
  readonly dashed?: boolean;
  /**
   * A dash's length: world units (CSS pixels for screen lines).
   *
   * @default 3
   * @range 0..
   */
  readonly dashSize?: number;
  /**
   * A gap's length.
   *
   * @default 1
   * @range 0..
   */
  readonly gapSize?: number;
}


/**
 * A point cloud drawn this frame only (immediate mode), as three.js Points with a PointsMaterial: a square per point facing the camera. Submit it every tick it should show (og.Points keeps a static set).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PointsDesc {
  /**
   * The positions: world, or with screen CSS pixels from the top left, y down.
   *
   * @default []
   * @unit m (metres)
   */
  readonly points?: readonly Vector3[];
  /**
   * Per point linear colours; empty is `color` for all.
   *
   * @default []
   */
  readonly colors?: readonly Color[];
  /**
   * The linear colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  readonly color?: Color;
  /**
   * The alpha.
   *
   * @default 1
   * @range 0..1
   */
  readonly opacity?: number;
  /**
   * The side of each square: world units with sizeAttenuation, else CSS pixels.
   *
   * @default 1
   * @range 0..
   */
  readonly size?: number;
  /**
   * Size in world units, smaller with distance, as three.js PointsMaterial.sizeAttenuation; false keeps every point `size` CSS pixels.
   *
   * @default true
   */
  readonly sizeAttenuation?: boolean;
  /**
   * Hidden behind nearer geometry; false draws them over the scene.
   *
   * @default true
   */
  readonly depthTest?: boolean;
  /**
   * Draw in screen space over the finished frame, positions and size in CSS pixels.
   *
   * @default false
   */
  readonly screen?: boolean;
}


/**
 * Draws a geometry's vertices as lines at the entity's MatrixWorld, every frame (three.js Line, LineSegments or LineLoop over a BufferGeometry): a static set of lines kept as data, a level's outline, a track, a constellation. The vertices are taken in order (the geometry's indices are not read), with its vertex colours times color.
 *
 * The value of the `Lines` component (token `og.Lines`).
 */
export interface Lines {
  /** The vertices (og.render.bufferGeometry: an unindexed one of any count). */
  geometry: Geometry;
  /** How the vertices join. */
  mode: LineMode;
  /**
   * The linear colour, times the vertex colours.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  color: Color;
  /**
   * The alpha.
   *
   * @default 1
   * @range 0..1
   * @tunable
   */
  opacity: number;
  /**
   * CSS pixels, or world units with worldUnits.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  width: number;
  /** Width in world units. */
  worldUnits: boolean;
  /**
   * Hidden behind nearer geometry.
   *
   * @default true
   */
  depthTest: boolean;
  /**
   * Dashes' length in world units along the line; 0 is solid (three.js LineDashedMaterial.dashSize).
   *
   * @range 0..
   */
  dashSize: number;
  /**
   * Gaps' length between dashes.
   *
   * @default 1
   * @range 0..
   */
  gapSize: number;
  /**
   * Whether it is drawn.
   *
   * @default true
   */
  visible: boolean;
  /**
   * Three.js layers, as Mesh.layers: drawn by every camera whose mask shares a bit with it (og.Camera, og.CameraView, og.Mirror).
   *
   * @default 1
   */
  layers: number;
}

/**
 * Draws a geometry's vertices as a point cloud at the entity's MatrixWorld, every frame (three.js Points over a BufferGeometry): star fields, scanned surfaces, sparse markers kept as data. Vertex colours times color.
 *
 * The value of the `Points` component (token `og.Points`).
 */
export interface Points {
  /** The vertices. */
  geometry: Geometry;
  /**
   * The linear colour, times the vertex colours.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  color: Color;
  /**
   * The alpha.
   *
   * @default 1
   * @range 0..1
   * @tunable
   */
  opacity: number;
  /**
   * World units with sizeAttenuation, else CSS pixels.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  size: number;
  /**
   * Size in world units, smaller with distance.
   *
   * @default true
   */
  sizeAttenuation: boolean;
  /**
   * Hidden behind nearer geometry.
   *
   * @default true
   */
  depthTest: boolean;
  /**
   * Whether it is drawn.
   *
   * @default true
   */
  visible: boolean;
  /**
   * Three.js layers, as Mesh.layers: drawn by every camera whose mask shares a bit with it (og.Camera, og.CameraView, og.Mirror).
   *
   * @default 1
   */
  layers: number;
}

/**
 * Which meshes og.render.raycast tests, as three.js Raycaster's near, far and layers.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RaycastGeometryDesc {
  /**
   * How far the ray reaches (three.js Raycaster.far); 0 is unbounded.
   *
   * @default 0
   * @range 0..
   * @unit m (metres)
   */
  readonly maxDistance?: number;
  /**
   * Answer only the nearest hit (cheaper: the BVH stops early).
   *
   * @default false
   */
  readonly first?: boolean;
  /**
   * A bit mask of the layers tested, as three.js Raycaster.layers: an og.Mesh or og.Model is tested when its own layers (Mesh.layers, Model.layers, 1 by default) share a bit with it, so the default tests every mesh on layer 0.
   *
   * @default 1
   */
  readonly layers?: number;
  /**
   * Test meshes whose visible is false too.
   *
   * @default false
   */
  readonly invisible?: boolean;
}

/** Where a ray meets a drawn triangle, as three.js Raycaster's intersection. */
export interface GeometryHit {
  /** The og.Mesh's or og.Model's entity. */
  readonly entity: Entity;
  /** The geometry hit. */
  readonly geometry: Geometry;
  /**
   * From the ray's origin along its normalised direction.
   *
   * @unit m (metres)
   */
  readonly distance: number;
  /**
   * Where, in world space.
   *
   * @unit m (metres)
   */
  readonly point: Vector3;
  /**
   * The triangle's face normal in world space (its front side, counter-clockwise), unit length.
   */
  readonly normal: Vector3;
  /** The texture coordinates there, interpolated. */
  readonly uv: Vector2;
  /** The triangle's index in the geometry, as three.js faceIndex. */
  readonly faceIndex: number;
}


/**
 * How a render target's colour is stored.
 * - `"ldr"`: 8-bit sRGB: the view's finished image (exposed with its camera's exposure and tone mapped with RenderSettings' curve), as a loaded colour texture: a monitor, a minimap, a portrait (show it with an unlit material's map or a sprite).
 * - `"hdr"`: 16-bit float, linear: the view's radiance before exposure and tone mapping, exposed once with the frame that shows it: a mirror or a portal (show it as a material's emissiveMap with color black).
 */
export type RenderTargetFormat = "ldr" | "hdr";

/**
 * A RenderTargetFormat field: the variant's index.
 *
 * A `RenderTargetFormat` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function RenderTargetFormat(value?: RenderTargetFormat, options?: FieldOptions): FieldSpec<"RenderTargetFormat">;

/**
 * A texture a camera draws into (og.render.renderTarget), as three.js WebGLRenderTarget.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type RenderTarget = number & { readonly [handleType]: "RenderTarget" };

/**
 * A RenderTarget handle field (0 is none).
 *
 * A `RenderTarget` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function RenderTarget(value?: RenderTarget, options?: FieldOptions): FieldSpec<"RenderTarget">;

/**
 * A render target's size and storage, as three.js WebGLRenderTarget's width, height, type and depthTexture.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RenderTargetDesc {
  /**
   * Its width in texels.
   *
   * @default 512
   * @range 1..
   */
  readonly width?: number;
  /**
   * Its height in texels.
   *
   * @default 512
   * @range 1..
   */
  readonly height?: number;
  /**
   * How its colour is stored.
   *
   * @default "ldr"
   */
  readonly format?: RenderTargetFormat;
  /**
   * Keep the view's depth too, as a second texture (RenderTargetInfo.depthTexture): its linear view depth from near (0) to far (1) in red, green and blue, of the opaque meshes and models, alpha-cut ones cut (transparent meshes, sprites and particles write none), drawn on every tier (RenderCapabilities.renderTargetDepth). It costs a second draw of the view's opaque meshes.
   *
   * @default false
   */
  readonly depth?: boolean;
}

/** What og.render.renderTarget made. */
export interface RenderTargetInfo {
  /** The target, for CameraView.target and Mirror.target. */
  readonly target: RenderTarget;
  /**
   * Its colour: any material map, og.Sprite, og.render.sprite or og.ui image shows it, as drawn by the last camera that drew into it (transparent black before).
   */
  readonly texture: Texture;
  /** Its depth (RenderTargetDesc.depth); 0 without. */
  readonly depthTexture: Texture;
  /** Its width in texels. */
  readonly width: number;
  /** Its height in texels. */
  readonly height: number;
}


/**
 * A secondary camera, as a second three.js camera rendered with setRenderTarget or setViewport: it draws the scene from the entity's MatrixWorld (looking down its local -Z, as og.Camera) into a render target (target), or, without one, into a rect of the frame (viewport: split-screen, picture-in-picture) over the finished image and under the UI.
 * It draws what the main camera draws, from its own eye and on its own layers: the meshes and models (skinned and with any material: hooks and full shaders run their vertex and fragment entries), opaque then transparent back to front, the terrain and the sky, the particle systems, og.Lines and og.Points, world sprites (og.Sprite, og.render.sprite) and world text, lit by the frame's lights and shadowed by its shadow maps. Each view is a draw of everything its layers admit: keep a view cheap with its layers and `every`. Screen sprites, the UI and debug draw stay on the main frame. Of the post chain, an ldr target and a viewport carry the view's exposure (its camera's) and the frame's tone curve; an hdr target carries linear radiance before exposure, exposed by the frame that shows it. Neither runs the frame's bloom, AO, SSR, depth of field, colour grading or custom post passes, whose inputs are the main camera's. Sprites in a view are not 2D-lit and do not glow; trails and 2D fluids draw on the main frame only. Views draw lowest order first, all before the frame.
 *
 * The value of the `CameraView` component (token `og.CameraView`).
 */
export interface CameraView {
  /** The target it draws into; 0 draws into the viewport rect of the frame. */
  target: RenderTarget;
  /**
   * The viewport's left edge, as a fraction of the frame's width (without a target).
   *
   * @range 0..1
   */
  viewportX: number;
  /**
   * The viewport's top edge, as a fraction of the frame's height.
   *
   * @range 0..1
   */
  viewportY: number;
  /**
   * The viewport's width, as a fraction of the frame's.
   *
   * @default 1
   * @range 0..1
   */
  viewportWidth: number;
  /**
   * The viewport's height, as a fraction of the frame's.
   *
   * @default 1
   * @range 0..1
   */
  viewportHeight: number;
  /**
   * The vertical field of view, as og.Camera.fov.
   *
   * @default 50
   * @range 1..179
   * @unit deg (degrees)
   * @tunable
   */
  fov: number;
  /**
   * The near clipping plane.
   *
   * @default 0.1
   * @range 0.001..
   * @unit m (metres)
   */
  near: number;
  /**
   * The far clipping plane.
   *
   * @default 2000
   * @range 0.01..
   * @unit m (metres)
   */
  far: number;
  /**
   * The zoom factor, as og.Camera.zoom.
   *
   * @default 1
   * @range 0.01..100
   */
  zoom: number;
  /** An orthographic projection of `height` world units. */
  orthographic: boolean;
  /**
   * The visible world height of the orthographic projection.
   *
   * @default 10
   * @range 0.01..
   * @unit m (metres)
   */
  height: number;
  /**
   * Its exposure before tone mapping (an ldr target and a viewport).
   *
   * @default 1
   * @range 0..16
   * @tunable
   */
  exposure: number;
  /**
   * Three.js layers, as og.Camera.layers: it draws the objects whose layers share a bit with it.
   *
   * @default 1
   */
  layers: number;
  /** Views draw in ascending order (ties in entity order), all before the frame. */
  order: number;
  /** Clear to clearColor instead of drawing the scene's background (sky or colour). */
  clear: boolean;
  /**
   * The linear colour it clears to with clear.
   *
   * @default { r: 0, g: 0, b: 0 }
   */
  clearColor: Color;
  /**
   * Draw once every this many ticks (when the tick is a multiple of it); the target keeps its image between. 0 and 1 draw every frame: a security monitor at 4 costs a quarter.
   *
   * @default 1
   */
  every: number;
  /**
   * Whether it draws.
   *
   * @default true
   */
  active: boolean;
}

/**
 * A planar mirror: the main camera reflected in the entity's xy plane (its local +Z the mirror's facing side), drawn into target fitted to a width by height rect centred on the entity, as a mirrored og.CameraView. Give the mirror's mesh (an og.render.planeGeometry of the same size) a material that reads the target: an hdr target as emissiveMap with color black, or an ldr target as an unlit map. Nothing behind the glass is drawn; seen from behind, it draws nothing.
 *
 * The value of the `Mirror` component (token `og.Mirror`).
 */
export interface Mirror {
  /** The target it draws into. */
  target: RenderTarget;
  /**
   * The mirror's width along the entity's local +X.
   *
   * @default 1
   * @range 0.001..
   * @unit m (metres)
   */
  width: number;
  /**
   * Its height along local +Y.
   *
   * @default 1
   * @range 0.001..
   * @unit m (metres)
   */
  height: number;
  /**
   * Three.js layers it reflects, as og.Camera.layers.
   *
   * @default 1
   */
  layers: number;
  /**
   * Draw once every this many ticks, as CameraView.every.
   *
   * @default 1
   */
  every: number;
  /**
   * Whether it draws.
   *
   * @default true
   */
  active: boolean;
}

/**
 * A custom post pass from og.load.post (an `.ogt` of `type post`): a WGSL `post` function over the lit image and the parameters it reads.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Post = number & { readonly [handleType]: "Post" };

/**
 * A Post handle field (0 is none).
 *
 * A `Post` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Post(value?: Post, options?: FieldOptions): FieldSpec<"Post">;


/** A custom post pass the device failed to compile. */
export interface PostError {
  /** The pass. */
  readonly post: Post;
  /** The device's message. */
  readonly message: string;
}


/**
 * How the device draws Fog.volumetric (RenderCapabilities.volumetricFog).
 * - `"froxel"`: A grid of view-aligned cells over the camera's frustum (160 by 90 by 64, slices deepening with the square of their index) lit in compute by the sun through its shadow cascades and by the clustered point, spot and rect lights through their shadows and cookies (each cell the mean of eight samples over its depth), then integrated front to back: the full tier.
 * - `"raymarch"`: Each pixel's view ray marched through the fog in 24 jittered steps, each lit by the sun through its cascades and by the first 8 point, spot and rect lights of its screen cluster through their shadows and cookies (RenderCapabilities.volumetricLights): WebGL2, which has no compute.
 */
export type VolumetricFog = "froxel" | "raymarch";

/**
 * A rectangle of light, as three.js RectAreaLight: width by height metres centred on the entity in its local xy plane, facing along its local -Z (aim it with transform.lookAt, as a spot), emitting color * intensity from its front, or from both faces with twoSided. Shaded with linearly transformed cosines (Heitz et al. 2016, three.js's tables), so its highlight on a glossy floor is the rectangle's shape and its light on a matte wall falls off as a real panel's does. With castShadow it casts a soft shadow in the frame; og bake lightmaps traces its exact soft shadows into the bounce light either way.
 * Area lights are clustered with the point and spot lights and share their MAX_LIGHTS budget; each reaches as far as its brightness carries (where it would light a white surface facing it at one hundredth of one), and fades smoothly to zero there.
 *
 * The value of the `RectLight` component (token `og.RectLight`).
 */
export interface RectLight {
  /**
   * The light's linear colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  color: Color;
  /**
   * The emitted radiance (luminance), as three.js RectAreaLight.intensity.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  intensity: number;
  /**
   * Its size along the entity's local x, as RectAreaLight.width.
   *
   * @default 10
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  width: number;
  /**
   * Its size along the entity's local y, as RectAreaLight.height.
   *
   * @default 10
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  height: number;
  /** Emit from the back face as well (a hanging panel lit both ways). */
  twoSided: boolean;
  /**
   * Cast a soft shadow: a shadow map from the rectangle's centre (two when twoSided, counted in the shadowed lights' budget) read with percentage-closer soft shadows, sharp where a caster touches its receiver and widening with the rectangle's size away from it. The view is from the centre, so a caster close to a large panel is shadowed as from its centre.
   */
  castShadow: boolean;
}

/**
 * A box of baked light probes (an irradiance volume): og bake lightmaps traces the light arriving from the scene (bounced off its static models, from the sky, from emissive surfaces) at a grid of points spacing metres apart inside it, as first-order spherical harmonics. Every mesh, model, skinned character and particle-lit surface drawn inside it that has no lightmap of its own takes its ambient light from the probes instead of the sky's, blended at the box's edge over one spacing: a ball rolling past a red wall turns red on that side.
 * The box is size metres centred on the entity, axis-aligned in world space (its rotation and scale are ignored). Its probes are the scene's: unbaked (no bake since the volume was added, moved or resized) it lights nothing and og doctor --project names the scene.
 *
 * The value of the `LightVolume` component (token `og.LightVolume`).
 */
export interface LightVolume {
  /**
   * The box's extent along world x, y and z.
   *
   * @default { x: 4, y: 4, z: 4 }
   * @unit m (metres)
   */
  size: Vector3;
  /**
   * The most distance between neighbouring probes (the box is divided evenly, with probes on its faces; at most 64 a side).
   *
   * @default 1
   * @range 0.1..16
   * @unit m (metres)
   */
  spacing: number;
}

/** One instantiated scene's baked lighting, as og.render.bakedLighting reports it. */
export interface BakedScene {
  /** The scene's asset id, such as `scenes/cornell.ogt`. */
  readonly scene: string;
  /** Its lightmap and probes are loaded and drawn. */
  readonly baked: boolean;
  /**
   * Why not, when not: the scene has no bake (`og bake lightmaps` writes `<scene>.lightmap.ogt`), its bake is stale (a static model or the scene changed since; the reason names which), or RenderSettings.bakedLighting is off. Empty when baked.
   */
  readonly reason: string;
  /** Static model instances drawn with the lightmap. */
  readonly models: number;
  /** Lightmap pages (RGBA16F texture array layers). */
  readonly pages: number;
  /** The pages' side in texels. */
  readonly texels: number;
  /** LightVolume grids its dynamic objects sample. */
  readonly volumes: number;
  /** Probes in those volumes. */
  readonly probes: number;
}


// ── load ────────────────────────────────────────────────────────
// Asset loading from cooked output: `og cook` turns assets/ into cooked/ (.ogb blobs and the manifest cooked/.cook.ogt), and the runtime reads only cooked/.
// Paths are asset ids: the path under assets/ (`models/ship.glb`, `scenes/main.ogt`); an `assets/` prefix is accepted. Every blob is checked against the manifest's type, schema version and cook stamp.
// The game's bundle (`cooked`) is opened before the first tick and GameDesc.scene is spawned then. Loads from an open bundle complete at once on both hosts (the web host fetches the bundle's files before the engine starts); only og.load.bundle of another bundle can be pending, and it completes at a tick boundary.

/**
 * A loaded bundle, scene or glTF asset.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Asset = number & { readonly [handleType]: "Asset" };

/**
 * A cooked MSDF font: its glyph table and one or more atlas pages.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Font = number & { readonly [handleType]: "Font" };

/**
 * A cooked sound: QOA-encoded PCM the audio module plays.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Sound = number & { readonly [handleType]: "Sound" };

/**
 * A cooked UI style: its values, atlas, fonts, icons and sounds loaded with it, for og.ui.configure.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Style = number & { readonly [handleType]: "Style" };

/**
 * A cooked texture array: same-sized layers uploaded as one 2D-array texture (og.load.textureArray).
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type TextureArray = number & { readonly [handleType]: "TextureArray" };

/**
 * A TextureArray handle field (0 is none).
 *
 * A `TextureArray` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function TextureArray(value?: TextureArray, options?: FieldOptions): FieldSpec<"TextureArray">;

/**
 * An Asset handle field.
 *
 * A `Asset` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Asset(value?: Asset, options?: FieldOptions): FieldSpec<"Asset">;

/**
 * A Sound handle field.
 *
 * A `Sound` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Sound(value?: Sound, options?: FieldOptions): FieldSpec<"Sound">;

/**
 * Where a load is.
 * - `"pending"`: Still loading.
 * - `"ready"`: Loaded; it can be instantiated.
 * - `"failed"`: Failed; og.load.error gives the reason.
 */
export type LoadStatus = "pending" | "ready" | "failed";

/**
 * How to load a glTF, as three.js GLTFLoader options where they apply.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GltfDesc {
  /**
   * Which glTF scene to instantiate.
   *
   * @default 0
   */
  readonly scene?: number;
  /**
   * Whether the models instantiated from it cast shadows: world.instantiate of the handle and a scene's `model` line give their og.Model this castShadow. On when omitted, so a model loaded without a desc casts as it always has; false turns it off for the model (og.Model.castShadow turns it off for one instance). The flags belong to the asset: another og.load.gltf of the same path answers the same handle and sets them again, for the models instantiated after it.
   *
   * @default true
   */
  readonly castShadow?: boolean;
  /**
   * Whether the models instantiated from it receive shadows (og.Model.receiveShadow, as castShadow): on when omitted; false lights them as if unshadowed by every light.
   *
   * @default true
   */
  readonly receiveShadow?: boolean;
}

/** What a loaded glTF holds. */
export interface GltfInfo {
  /** How many scenes it has. */
  readonly scenes: number;
  /** Its geometries, in glTF mesh-primitive order. */
  readonly geometries: readonly Geometry[];
  /** Its materials, in glTF order, then the default material. */
  readonly materials: readonly Material[];
  /**
   * Its clip names, in glTF order, from the model's `#anim` blob (empty for a model without clips): a skinned glTF's animations, an animated `.vox`'s `frames` or `voxClip` names. og.load.animation answers the same clips with their ClipAsset handles and durations, and og.anim.clip finds one by name.
   */
  readonly animations: readonly string[];
}

/** An open cooked bundle. */
export interface BundleInfo {
  /** The cooked directory it was opened from. */
  readonly path: string;
  /** Source assets it holds. */
  readonly assets: number;
  /** Cooked outputs (blobs) it holds. */
  readonly outputs: number;
}

/** A loaded font's metrics, in em units unless stated. */
export interface FontInfo {
  /** Glyphs cooked into it. */
  readonly glyphs: number;
  /** Atlas pages; the renderer draws one page per pass with one shared layout. */
  readonly pages: number;
  /** Atlas pixels per em. */
  readonly emPx: number;
  /** The distance range in atlas pixels. */
  readonly pxRange: number;
  /** Ascender above the baseline. */
  readonly ascender: number;
  /** Descender below the baseline (negative). */
  readonly descender: number;
  /** Ascender minus descender plus the line gap. */
  readonly lineHeight: number;
  /**
   * The icons of an icon set (an `.ogt` of `type icons`, SVGs cooked like a font for UiDesc.icons.font); 0 for a text font.
   */
  readonly icons: number;
}

/** A loaded sound. */
export interface SoundInfo {
  /**
   * Samples per second.
   *
   * @unit hz (ticks or cycles per second)
   */
  readonly rate: number;
  /** Interleaved channels. */
  readonly channels: number;
  /** Samples per channel. */
  readonly frames: number;
  /**
   * Its length.
   *
   * @unit s (seconds of simulation time)
   */
  readonly seconds: number;
  /**
   * The chunks it was cooked in (`sound { stream true }` in its import settings, or a QOA file above 1 MiB), the first included; 0 when it was cooked whole. A chunked sound is never resident whole: the engine reads its chunks from about half a second behind each playhead to four seconds ahead (on the web, fetched like a streaming group's pack) and drops the rest.
   */
  readonly chunks: number;
}

/** What og.load.textureArray uploaded. */
export interface TextureArrayInfo {
  /** The array. */
  readonly array: TextureArray;
  /** Each layer's width in texels. */
  readonly width: number;
  /** Each layer's height in texels. */
  readonly height: number;
  /** Its layers, in the order the file lists them. */
  readonly layers: number;
  /** Mip levels of each layer. */
  readonly levels: number;
  /**
   * The encoding on the GPU: `rgba8`, or the block format the device takes for its kind (`bc7`, `bc5`, `astc4x4`, `etc2`, `eacrg11`).
   */
  readonly format: string;
  /**
   * Bytes it takes on the GPU, every level of every layer (counted in the frame's texture bytes).
   */
  readonly bytes: number;
}

/**
 * Draws a loaded glTF's default scene at the entity's world transform: every mesh node with its glTF material. Scene entities with a `model` line get one; world.instantiate of a glTF asset spawns one.
 * The glTF's materials are shared by every instance of it; `tint` and `flash` are this instance's own, carried with its transform to the lit shaders (one draw for every instance, whatever each shows), so one of a crowd can flash when struck.
 *
 * The value of the `Model` component (token `og.Model`).
 */
export interface Model {
  /** The glTF asset. */
  asset: Asset;
  /**
   * A linear colour, 0 to 1 a channel, multiplied with every material's base colour on this instance only (white: as the glTF).
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  tint: Color;
  /**
   * A hit flash on this instance only: the lit colour mixed towards flashColor by this much (og.tween.flash fades it out).
   *
   * @range 0..1
   * @tunable
   */
  flash: number;
  /**
   * The flash's linear colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  flashColor: Color;
  /**
   * Every mesh of the model casts shadows, as Mesh.castShadow: drawn into the sun's cascades and the shadowed lights' maps. On by default, unlike Mesh's (a loaded model casts, as it always has); false takes this instance out of every shadow pass. world.instantiate of a glTF and a scene's `model` line set it from the load's GltfDesc.castShadow, unless the scene gives the entity a `Model` table (whose castShadow is on unless it says false).
   *
   * @default true
   */
  castShadow: boolean;
  /**
   * Every mesh of the model receives shadows, as Mesh.receiveShadow; false lights this instance as if unshadowed by every light. On by default. Set as castShadow is, from GltfDesc.receiveShadow or the scene's `Model` table.
   *
   * @default true
   */
  receiveShadow: boolean;
  /**
   * Whether it is drawn, as Mesh.visible: false skips every draw of this instance (its meshes, its shadows) while it keeps its transform, its animation and its physics, so a ghost car or a pickup waiting to respawn is hidden without moving it away.
   *
   * @default true
   */
  visible: boolean;
  /**
   * This instance's level of detail bias, multiplied with RenderSettings.lodBias: the cook simplifies every mesh into levels (a model's import settings, `model { lods 1 0.5 0.25 }` by default, the triangle fraction of each level; a glTF with MSFT_lod keeps its own levels), and the renderer draws the level its projected size calls for, with hysteresis so it does not flicker at the boundary. Above 1 keeps detail farther (a hero), below 1 drops it sooner (a crowd); 0 always draws the full mesh. dump_entity shows the level drawn.
   *
   * @default 1
   * @range 0..16
   * @tunable
   */
  lodBias: number;
  /**
   * Three.js layers of every mesh of the model, as Mesh.layers.
   *
   * @default 1
   */
  layers: number;
  /**
   * Part of its scene's baked lighting: og bake lightmaps gives a static model lightmap UVs (xatlas, over its world-space triangles) and traces the light arriving at its surfaces from the scene into the scene's lightmap (`<scene>.lightmap.ogt` and `.ktx2` beside the scene). Once baked, the model draws with it (its ambient light from the lightmap instead of the sky; its lights' direct light, shadows and highlights still drawn every frame) for as long as the bake matches the scene. A scene entity says `static true` beside its `model` line; an unbaked static model draws as any model, and og doctor --project names the scene.
   */
  static: boolean;
}
































/**
 * A load completed.
 *
 * The payload of the `AssetReady` event (token `og.AssetReady`).
 */
export interface AssetReady {
  /** The handle. */
  readonly asset: Asset;
  /** The path it was loaded from. */
  readonly path: string;
}

/**
 * A load failed.
 *
 * The payload of the `AssetFailed` event (token `og.AssetFailed`).
 */
export interface AssetFailed {
  /** The handle. */
  readonly asset: Asset;
  /** The path it was loaded from. */
  readonly path: string;
  /** Why, naming the fix. */
  readonly reason: string;
}

// ── physics ─────────────────────────────────────────────────────
// Rigid bodies over rapier (rapier3d in the 3D profile, rapier2d in 2D, both with enhanced determinism): game code owns the components, the engine's og.physics system steps them after the game's systems and writes the dynamic bodies back.
// The profile is the project's (`profile "2d"` or `"3d"` in project.ogt). In 2D, bodies move in the xy plane and rotate about z; a transform keeps its z. A collider without a RigidBody is static at its transform.
// The rapier world persists between ticks and is world state: snapshots, restore and the world hash cover the whole simulation. Events sent in tick N are read in tick N + 1.

/**
 * RigidBody.kind: moved by the solver; its Transform and Velocity are written back every tick.
 */
export declare const BODY_DYNAMIC: 0;

/** RigidBody.kind: never moves; collides with dynamic bodies. */
export declare const BODY_STATIC: 1;

/**
 * RigidBody.kind: not moved by the solver. With an og.Velocity the step moves it by it each tick and writes its Transform back (a lift, a conveyor, a turning gear); a Transform game code writes moves it there first (a teleport). Without one it stays where its Transform puts it. Either way it pushes dynamic bodies and carries the controllers standing on it.
 */
export declare const BODY_KINEMATIC: 2;

/** RigidBody.lock bit: no translation along x. */
export declare const LOCK_TRANSLATION_X: 1;

/** RigidBody.lock bit: no translation along y. */
export declare const LOCK_TRANSLATION_Y: 2;

/** RigidBody.lock bit: no translation along z. */
export declare const LOCK_TRANSLATION_Z: 4;

/** RigidBody.lock bit: no rotation about x. */
export declare const LOCK_ROTATION_X: 8;

/** RigidBody.lock bit: no rotation about y. */
export declare const LOCK_ROTATION_Y: 16;

/** RigidBody.lock bit: no rotation about z (the 2D profile's only rotation). */
export declare const LOCK_ROTATION_Z: 32;

/** Collider.shape: a box of `size` (both profiles). */
export declare const SHAPE_BOX: 0;

/** Collider.shape: a sphere of `radius` (a circle in 2D). */
export declare const SHAPE_SPHERE: 1;

/**
 * Collider.shape: a capsule along y of `radius`, with `height` between the cap centres (both profiles).
 */
export declare const SHAPE_CAPSULE: 2;

/** Collider.shape: a cylinder along y of `radius` and `height` (3D only). */
export declare const SHAPE_CYLINDER: 3;

/**
 * Collider.shape: the convex hull of the vertices of `mesh` (3D), or of their xy (a convex polygon, 2D).
 */
export declare const SHAPE_CONVEX: 4;

/** Collider.shape: the triangles of `mesh` (3D only; for static and kinematic bodies). */
export declare const SHAPE_TRIMESH: 5;

/**
 * Collider.shape: the height field registered as `mesh`, centred on the entity (3D only; a terrain: og.terrain.load's TerrainInfo.geometry).
 */
export declare const SHAPE_HEIGHTFIELD: 6;

/** Physics.profile: rapier2d, bodies in the xy plane rotating about z. */
export declare const PROFILE_2D: 2;

/** Physics.profile: rapier3d. */
export declare const PROFILE_3D: 3;

/**
 * A body the physics step simulates. It needs an og.Transform and usually has an og.Collider; a dynamic body without an og.Velocity gets one.
 * Changing a RigidBody, a Collider or Transform.scale rebuilds the body (it loses its contacts' warm starts); a Transform or Velocity that game code changed moves it (a teleport).
 *
 * The value of the `RigidBody` component (token `og.RigidBody`).
 */
export interface RigidBody {
  /**
   * BODY_DYNAMIC (0, simulated and written back), BODY_STATIC (1, never moves) or BODY_KINEMATIC (2, moved by its Velocity each tick, or by game code through its Transform).
   *
   * @range 0..2
   * @label Kind
   */
  kind: number;
  /**
   * Mass; 0 derives it from the collider's density and volume.
   *
   * @range 0..
   * @unit kg (kilograms)
   * @tunable
   */
  mass: number;
  /**
   * How fast linear velocity decays (0: none).
   *
   * @range 0..100
   * @unit perSecond (a rate per second: the fraction of a value that decays in one second)
   * @tunable
   */
  linearDamping: number;
  /**
   * How fast angular velocity decays (0: none).
   *
   * @range 0..100
   * @unit perSecond (a rate per second: the fraction of a value that decays in one second)
   * @tunable
   */
  angularDamping: number;
  /**
   * Multiplies Physics.gravity for this body: 0 floats, 1 is normal.
   *
   * @default 1
   * @range -10..10
   * @tunable
   */
  gravityScale: number;
  /**
   * Locked axes, a sum of the LOCK_* bits: 1, 2, 4 no translation along x, y, z; 8, 16, 32 no rotation about x, y, z (2D uses 1, 2 and 32).
   *
   * @range 0..63
   */
  lock: number;
  /**
   * 1 enables continuous collision detection, for fast small bodies that must not tunnel; 0 off.
   *
   * @range 0..1
   */
  ccd: number;
  /**
   * Moves the centre of mass from where the collider puts it (its shape's centre), in the entity's own frame and scaled by og.Transform.scale like Collider.offset; the inertia stays the shape's. A car's lowered below its wheel mounts keeps it on its wheels at full lock; a weight at a toy's base rights it. Needs an og.Collider.
   *
   * @unit m (metres)
   * @tunable
   */
  centerOfMass: Vector3;
}

/**
 * The shape an entity collides with, attached to its RigidBody (or, without one, a static collider at its Transform). Sizes are scaled by og.Transform.scale.
 *
 * The value of the `Collider` component (token `og.Collider`).
 */
export interface Collider {
  /**
   * One of the SHAPE_* codes: 0 box, 1 sphere (a circle in 2D), 2 capsule, 3 cylinder (3D), 4 convex hull of `mesh` (a polygon in 2D), 5 triangle mesh of `mesh` (3D), 6 the height field `mesh` (3D: a terrain).
   *
   * @range 0..6
   * @label Shape
   */
  shape: number;
  /**
   * Full extents of a box, as three.js BoxGeometry(width, height, depth).
   *
   * @default { x: 1, y: 1, z: 1 }
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  size: Vector3;
  /**
   * Radius of a sphere, circle, capsule or cylinder.
   *
   * @default 0.5
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  radius: number;
  /**
   * A capsule's distance between its cap centres (three.js CapsuleGeometry length); a cylinder's height.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  height: number;
  /**
   * The geometry whose vertices a convex or triangle-mesh shape uses: a loaded glTF's geometry (registered automatically) or one given to og.physics.setMesh; for SHAPE_HEIGHTFIELD, a terrain's height field (og.terrain.load's TerrainInfo.geometry).
   */
  mesh: Geometry;
  /**
   * Position of the shape relative to the entity.
   *
   * @unit m (metres)
   */
  offset: Vector3;
  /** Rotation of the shape relative to the entity. */
  rotation: Quaternion;
  /**
   * Coulomb friction (0 is ice, 1 rubber); a pair uses the mean of both colliders.
   *
   * @default 0.5
   * @range 0..2
   * @tunable
   */
  friction: number;
  /**
   * Bounciness (0 no bounce, 1 elastic); a pair uses the mean of both colliders.
   *
   * @range 0..1
   * @tunable
   */
  restitution: number;
  /**
   * Density, used when RigidBody.mass is 0.
   *
   * @default 1
   * @range 0..
   * @unit kgPerM3 (kilograms per cubic metre (per square metre in the 2D profile))
   * @tunable
   */
  density: number;
  /**
   * 1 makes this a sensor: it reports SensorEntered and SensorExited and pushes nothing; 0 a solid.
   *
   * @range 0..1
   */
  sensor: number;
  /**
   * The collision layers this collider is on, as bits.
   *
   * @default 1
   */
  layers: number;
  /**
   * The layers it collides with, as bits. Two colliders interact when each one's layers meet the other's mask.
   *
   * @default 4294967295
   */
  mask: number;
}

/**
 * Linear and angular velocity. Written for dynamic bodies every tick (added to them by the step when missing); a kinematic body moves by it each tick (never added, never written). Set it to launch or stop a body; push one with og.physics.applyForce, applyImpulse, applyTorque or applyTorqueImpulse.
 *
 * The value of the `Velocity` component (token `og.Velocity`).
 */
export interface Velocity {
  /**
   * The linear velocity (2D uses x and y).
   *
   * @unit mps (metres per second)
   * @tunable
   */
  linear: Vector3;
  /**
   * The angular velocity about each axis (2D uses z).
   *
   * @unit radps (radians per second)
   * @tunable
   */
  angular: Vector3;
}

/**
 * The physics settings of the world. The profile is the project's; the same components serve both profiles.
 *
 * The `Physics` resource (token `og.Physics`).
 */
export interface Physics {
  /**
   * PROFILE_2D (2: rapier2d, the xy plane, rotation about z) or PROFILE_3D (3: rapier3d), from project.ogt. Set the other one at run time to change profile: the next step builds that profile's simulation from the components (contacts, joints' warm starts, ropes, cloth, fluids and ragdoll bodies start again), so a 3D game can play a 2D scene and back.
   *
   * @default 3
   * @range 2..3
   * @label Profile
   */
  profile: number;
  /**
   * The acceleration of every dynamic body, times its gravityScale; 2D uses x and y.
   *
   * @default { x: 0, y: -9.81, z: 0 }
   * @unit mps2 (metres per second squared)
   * @tunable
   */
  gravity: Vector3;
  /**
   * Physics steps per tick, each of dt / substeps: more is stiffer stacks and less tunnelling, at more cost.
   *
   * @default 1
   * @range 1..16
   * @tunable
   */
  substeps: number;
  /**
   * Solver iterations per step.
   *
   * @default 4
   * @range 1..32
   * @tunable
   */
  solverIterations: number;
  /**
   * A dynamic body touching something, supported with at most 2 mm of overlap and slower than this before and after a step, keeps its pose and loses its residual velocity (the same number in m/s and rad/s); 0 disables it.
   *
   * @default 0.02
   * @range 0..1
   * @unit mps (metres per second)
   * @tunable
   */
  restSpeed: number;
}

/**
 * Two solid colliders started touching during the tick.
 *
 * The payload of the `CollisionStarted` event (token `og.CollisionStarted`).
 */
export interface CollisionStarted {
  /** The entity with the lower id. */
  readonly a: Entity;
  /** The entity with the higher id. */
  readonly b: Entity;
  /** The deepest contact point in world space (z is 0 in 2D). */
  readonly point: Vector3;
  /** The contact normal, from a towards b. */
  readonly normal: Vector3;
}

/**
 * Two solid colliders stopped touching, or one of them went away.
 *
 * The payload of the `CollisionEnded` event (token `og.CollisionEnded`).
 */
export interface CollisionEnded {
  /** The entity with the lower id. */
  readonly a: Entity;
  /** The entity with the higher id. */
  readonly b: Entity;
}

/**
 * A collider started overlapping a sensor.
 *
 * The payload of the `SensorEntered` event (token `og.SensorEntered`).
 */
export interface SensorEntered {
  /** The entity whose collider is the sensor. */
  readonly sensor: Entity;
  /** The entity that entered it. */
  readonly other: Entity;
}

/**
 * A collider stopped overlapping a sensor, or one of them went away.
 *
 * The payload of the `SensorExited` event (token `og.SensorExited`).
 */
export interface SensorExited {
  /** The entity whose collider is the sensor. */
  readonly sensor: Entity;
  /** The entity that left it. */
  readonly other: Entity;
}

/**
 * One pair in contact at the end of a step, sent every tick for every touching pair: how a game asks what is touching what right now.
 *
 * The payload of the `Contact` event (token `og.Contact`).
 */
export interface Contact {
  /** The entity with the lower id. */
  readonly a: Entity;
  /** The entity with the higher id. */
  readonly b: Entity;
  /**
   * 0 for two solids; an overlap with a sensor otherwise: 1 a is the sensor, 2 b is, 3 both are.
   */
  readonly sensor: number;
}

/** `Joint.kind`: the two bodies move as one. */
export declare const JOINT_FIXED: 0;

/** `Joint.kind`: a hinge about `axis` (2D: about z), with optional limits and a motor. */
export declare const JOINT_REVOLUTE: 1;

/** `Joint.kind`: a slider along `axis`, with optional limits and a motor. */
export declare const JOINT_PRISMATIC: 2;

/** `Joint.kind`: a damped spring pulling the anchors to `restLength` apart. */
export declare const JOINT_SPRING: 3;

/** `Joint.kind`: a rope: the anchors stay at most `length` apart, free inside it. */
export declare const JOINT_ROPE: 4;

/** `Joint.kind`: a ball and socket (3D; the 2D profile treats it as a hinge). */
export declare const JOINT_BALL: 5;

/** `Joint.motor`: no motor. */
export declare const MOTOR_NONE: 0;

/** `Joint.motor`: drive towards `motorTarget` speed (rad/s or m/s). */
export declare const MOTOR_VELOCITY: 1;

/** `Joint.motor`: spring towards `motorTarget` angle or position (a servo). */
export declare const MOTOR_POSITION: 2;

/**
 * A joint between two bodies (or a body and a point of the world). The component may sit on one of the bodies (`body` 0) or on an entity of its own, so one body can have several joints. Anchors are in each body's frame (rotated and moved with it, not scaled); without `other` the second anchor is a world point. A joint whose bodies are not both `og.RigidBody` entities does nothing until they are.
 *
 * The value of the `Joint` component (token `og.Joint`).
 */
export interface Joint {
  /**
   * JOINT_FIXED 0, JOINT_REVOLUTE 1 (hinge), JOINT_PRISMATIC 2 (slider), JOINT_SPRING 3, JOINT_ROPE 4, JOINT_BALL 5.
   *
   * @range 0..5
   * @label Kind
   */
  kind: number;
  /** The first body; 0 is this entity. */
  body: Entity;
  /** The second body; 0 joins the first one to the world at `otherAnchor`. */
  other: Entity;
  /**
   * Where the joint sits on the first body, in its frame.
   *
   * @unit m (metres)
   */
  anchor: Vector3;
  /**
   * Where it sits on the second body, in its frame (a world point without `other`).
   *
   * @unit m (metres)
   */
  otherAnchor: Vector3;
  /**
   * The hinge or slide axis in the first body's frame (a hinge in 2D turns about z).
   *
   * @default { x: 1, y: 0, z: 0 }
   */
  axis: Vector3;
  /** Whether `min`..`max` bound the hinge angle (rad) or the slide (m). */
  limits: boolean;
  /** The lower limit: radians for a hinge, metres for a slider. */
  min: number;
  /** The upper limit. */
  max: number;
  /**
   * MOTOR_NONE 0, MOTOR_VELOCITY 1 (drive at motorTarget speed), MOTOR_POSITION 2 (spring to motorTarget).
   *
   * @range 0..2
   */
  motor: number;
  /**
   * The motor's target speed (rad/s, m/s) or angle or position (rad, m).
   *
   * @tunable
   */
  motorTarget: number;
  /**
   * A position motor's stiffness; a velocity motor's strength.
   *
   * @default 50
   * @range 0..
   * @tunable
   */
  motorStiffness: number;
  /**
   * A position motor's damping.
   *
   * @default 5
   * @range 0..
   * @tunable
   */
  motorDamping: number;
  /**
   * The most force (or torque) the motor applies; 0 is unbounded.
   *
   * @range 0..
   * @tunable
   */
  motorMaxForce: number;
  /**
   * A spring's rest length.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  restLength: number;
  /**
   * A spring's stiffness (N/m).
   *
   * @default 40
   * @range 0..
   * @tunable
   */
  stiffness: number;
  /**
   * A spring's damping (N·s/m).
   *
   * @default 2
   * @range 0..
   * @tunable
   */
  damping: number;
  /**
   * A rope's longest distance between the anchors.
   *
   * @default 2
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  length: number;
  /** Whether the two bodies still collide with each other. */
  collide: boolean;
  /**
   * Written: the hinge angle (rad) or the slide (m) after the step; the anchors' distance for springs and ropes.
   *
   * @readonly written by the engine
   */
  readonly value: number;
}

/**
 * A kinematic character (3D, or top-down 2D): the entity's capsule (or any og.Collider) moved by `move` each tick, sliding along walls, climbing slopes up to `maxSlope` and steps up to `stepHeight`, snapped to the ground, carried by the moving platform it stands on and falling by gravity. It keeps `skin` off every surface, standing that far over the ground; a step is climbed when the ground beyond it is at most `stepHeight` over the ground it stands on. What blocks it is every solid collider its og.Collider's `layers` and `mask` interact with (each one's layers meeting the other's mask), as for bodies. The entity needs an og.Transform and an og.Collider; the step makes it a kinematic body and pushes the dynamic bodies it walks into (`pushBodies`) with at most `pushStrength`, so a light one is shoved and a heavy one holds.
 *
 * The value of the `CharacterController` component (token `og.CharacterController`).
 */
export interface CharacterController {
  /**
   * The wanted velocity across the ground (m/s, world space; y is ignored unless gravityScale is 0). Set it every tick.
   *
   * @unit mps (metres per second)
   */
  move: Vector3;
  /**
   * Held to jump: a jump starts on the tick it becomes true while grounded (or in coyote time).
   */
  jump: boolean;
  /**
   * The upward speed of a jump.
   *
   * @default 6
   * @range 0..
   * @unit mps (metres per second)
   * @tunable
   */
  jumpSpeed: number;
  /**
   * Multiplies og.Physics.gravity; 0 for top-down movement in any direction.
   *
   * @default 1
   * @range 0..10
   * @tunable
   */
  gravityScale: number;
  /**
   * The steepest slope it walks up.
   *
   * @default 45
   * @range 0..89
   * @unit deg (degrees)
   * @tunable
   */
  maxSlope: number;
  /**
   * The highest step it climbs without jumping (0: none).
   *
   * @default 0.3
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  stepHeight: number;
  /**
   * How far below it the ground still holds it (stairs down, slopes): 0 lets it leave every edge.
   *
   * @default 0.2
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  snapDistance: number;
  /**
   * The gap kept between it and every surface.
   *
   * @default 0.02
   * @range 0.001..
   * @unit m (metres)
   */
  skin: number;
  /**
   * After leaving an edge, a jump still starts within this time.
   *
   * @default 0.1
   * @range 0..
   * @unit s (seconds of simulation time)
   * @tunable
   */
  coyoteTime: number;
  /**
   * The fastest fall.
   *
   * @default 40
   * @range 0..
   * @unit mps (metres per second)
   * @tunable
   */
  maxFallSpeed: number;
  /**
   * Push dynamic bodies it walks into, with at most `pushStrength`.
   *
   * @default true
   */
  pushBodies: boolean;
  /**
   * The most force it pushes with: each tick the bodies it walks into share an impulse of at most this times the tick's length (mass times the speed it asks of them, capped), so a body of a few kilograms is shoved at its speed while one whose friction holds more (a tonne on the ground) does not move. 0 leaves every body where it is, as `pushBodies` false does.
   *
   * @default 1000
   * @range 0..
   * @unit newton (newtons: the force that gives a kilogram one metre per second squared)
   * @tunable
   */
  pushStrength: number;
  /**
   * Written: standing on walkable ground after the step.
   *
   * @readonly written by the engine
   */
  readonly grounded: boolean;
  /**
   * Written: sliding down a slope steeper than maxSlope.
   *
   * @readonly written by the engine
   */
  readonly sliding: boolean;
  /**
   * Written: the velocity it moved at this tick (the platform's carry included).
   *
   * @unit mps (metres per second)
   * @readonly written by the engine
   */
  readonly velocity: Vector3;
  /**
   * Written: the ground's normal (zero in the air).
   *
   * @readonly written by the engine
   */
  readonly groundNormal: Vector3;
  /**
   * Written: seconds left in which a jump may still start (coyote time).
   *
   * @readonly written by the engine
   */
  readonly coyote: number;
  /**
   * Written: `jump` as the last step saw it (a jump starts on its rising edge).
   *
   * @readonly written by the engine
   */
  readonly jumpWasHeld: boolean;
  /**
   * Written: the entity it stands on (0 in the air); a moving one carries it.
   *
   * @readonly written by the engine
   */
  readonly ground: Entity;
}

/**
 * A 2D platformer character (the 2D profile): run, jump and fall with designer units (jump height and time to apex, not gravity), coyote time, jump buffering, variable jump height (releasing jump while rising cuts the jump), apex hang, air jumps, wall slide and wall jump, one-way platforms (colliders on `oneWayLayers` are solid from above only; hold `down` to drop through), moving platforms (it rides what it stands on) and corner correction (a jump that clips a ceiling corner by less than `cornerCorrection` slides past it). It keeps `skin` off every surface; what blocks it is every solid collider its og.Collider's `layers` and `mask` interact with (each one's layers meeting the other's mask), as for bodies. The entity needs an og.Transform and an og.Collider (a box or capsule); it becomes a kinematic body and pushes the dynamic bodies it runs into (`pushBodies`) with at most `pushStrength`, so a crate's mass tells: a light one is shoved at the run speed, a heavy one holds (set its og.RigidBody `mass`).
 *
 * The value of the `PlatformerController` component (token `og.PlatformerController`).
 */
export interface PlatformerController {
  /**
   * The run input, -1 (left) to 1 (right). Set it every tick.
   *
   * @range -1..1
   */
  moveX: number;
  /** Held to jump: a press is buffered for jumpBuffer; releasing while rising cuts the jump. */
  jump: boolean;
  /** Held to drop through a one-way platform. */
  down: boolean;
  /**
   * A velocity change to take on the next step (a knock-back, a spring, a recoil): added to `velocity`, then set back to zero. One with an upward y leaves the ground; the run's acceleration and gravity act on the result from then on, so a knock-back is `{ x: away * 6, y: 7 }` with `moveX` 0 for its few ticks.
   *
   * @unit mps (metres per second)
   */
  impulse: Vector2;
  /**
   * The top running speed.
   *
   * @default 8
   * @range 0..
   * @unit mps (metres per second)
   * @tunable
   */
  maxSpeed: number;
  /**
   * How fast it reaches maxSpeed on the ground.
   *
   * @default 70
   * @range 0..
   * @unit mps2 (metres per second squared)
   * @tunable
   */
  acceleration: number;
  /**
   * How fast it stops on the ground without input.
   *
   * @default 80
   * @range 0..
   * @unit mps2 (metres per second squared)
   * @tunable
   */
  deceleration: number;
  /**
   * Acceleration in the air.
   *
   * @default 45
   * @range 0..
   * @unit mps2 (metres per second squared)
   * @tunable
   */
  airAcceleration: number;
  /**
   * Deceleration in the air without input.
   *
   * @default 25
   * @range 0..
   * @unit mps2 (metres per second squared)
   * @tunable
   */
  airDeceleration: number;
  /**
   * How high a full jump goes.
   *
   * @default 3.2
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  jumpHeight: number;
  /**
   * How long a full jump takes to reach its height (sets gravity and the jump speed).
   *
   * @default 0.4
   * @range 0.05..
   * @unit s (seconds of simulation time)
   * @tunable
   */
  timeToApex: number;
  /**
   * Gravity multiplier while falling (snappier landings above 1).
   *
   * @default 1.8
   * @range 0..
   * @tunable
   */
  fallGravity: number;
  /**
   * Gravity multiplier while rising with jump released (variable jump height; 1 disables it).
   *
   * @default 2.5
   * @range 1..
   * @tunable
   */
  jumpCut: number;
  /**
   * Gravity multiplier near the apex while jump is held (hang time; 1 disables it).
   *
   * @default 0.6
   * @range 0..1
   * @tunable
   */
  apexGravity: number;
  /**
   * The vertical speed below which the apex multiplier applies.
   *
   * @default 1.5
   * @range 0..
   * @unit mps (metres per second)
   * @tunable
   */
  apexThreshold: number;
  /**
   * The fastest fall.
   *
   * @default 20
   * @range 0..
   * @unit mps (metres per second)
   * @tunable
   */
  maxFallSpeed: number;
  /**
   * After running off an edge, a jump still starts within this time.
   *
   * @default 0.1
   * @range 0..
   * @unit s (seconds of simulation time)
   * @tunable
   */
  coyoteTime: number;
  /**
   * A jump pressed this long before landing starts on landing.
   *
   * @default 0.12
   * @range 0..
   * @unit s (seconds of simulation time)
   * @tunable
   */
  jumpBuffer: number;
  /**
   * Extra jumps in the air (1: a double jump).
   *
   * @range 0..8
   */
  airJumps: number;
  /**
   * Slide down walls it pushes against (and jump off them).
   *
   * @default true
   */
  wallJumps: boolean;
  /**
   * The fastest slide down a wall.
   *
   * @default 3
   * @range 0..
   * @unit mps (metres per second)
   * @tunable
   */
  wallSlideSpeed: number;
  /**
   * A wall jump's velocity: x away from the wall, y up.
   *
   * @default { x: 8, y: 12 }
   * @unit mps (metres per second)
   * @tunable
   */
  wallJump: Vector2;
  /**
   * After a wall jump, the run input is ignored for this long (so it leaves the wall).
   *
   * @default 0.14
   * @range 0..
   * @unit s (seconds of simulation time)
   * @tunable
   */
  wallJumpLock: number;
  /** The collision layers that are one-way platforms (solid from above), as bits. */
  oneWayLayers: number;
  /**
   * How far a rising jump is nudged sideways past a ceiling corner (0: never).
   *
   * @default 0.15
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  cornerCorrection: number;
  /**
   * The steepest slope it runs up.
   *
   * @default 50
   * @range 0..89
   * @unit deg (degrees)
   * @tunable
   */
  maxSlope: number;
  /**
   * How far below it the ground still holds it (slopes, small drops).
   *
   * @default 0.1
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  snapDistance: number;
  /**
   * The gap kept between it and every surface.
   *
   * @default 0.01
   * @range 0.001..
   * @unit m (metres)
   */
  skin: number;
  /**
   * Push dynamic bodies it runs into, with at most `pushStrength`.
   *
   * @default true
   */
  pushBodies: boolean;
  /**
   * The most force it pushes with: each tick the bodies it runs into share an impulse of at most this times the tick's length (mass times the speed it asks of them, capped), so a body of a few kilograms is shoved at its speed while one whose friction holds more (a tonne on the ground) does not move. 0 leaves every body where it is, as `pushBodies` false does.
   *
   * @default 1000
   * @range 0..
   * @unit newton (newtons: the force that gives a kilogram one metre per second squared)
   * @tunable
   */
  pushStrength: number;
  /**
   * Written: standing on ground after the step.
   *
   * @readonly written by the engine
   */
  readonly grounded: boolean;
  /**
   * Written: -1 sliding on a wall to its left, 1 to its right, 0 none.
   *
   * @range -1..1
   * @readonly written by the engine
   */
  readonly onWall: number;
  /**
   * Written: -1 facing left, 1 right (the last run direction).
   *
   * @default 1
   * @range -1..1
   * @readonly written by the engine
   */
  readonly facing: number;
  /**
   * Written: the velocity after the step (the platform's carry excluded).
   *
   * @unit mps (metres per second)
   * @readonly written by the engine
   */
  readonly velocity: Vector3;
  /**
   * Written: rising from a jump (until the apex or a release cut).
   *
   * @readonly written by the engine
   */
  readonly jumping: boolean;
  /**
   * Written: air jumps left before landing.
   *
   * @readonly written by the engine
   */
  readonly airJumpsLeft: number;
  /**
   * Written: seconds of coyote time left.
   *
   * @readonly written by the engine
   */
  readonly coyote: number;
  /**
   * Written: seconds a buffered jump press stays valid.
   *
   * @readonly written by the engine
   */
  readonly buffer: number;
  /**
   * Written: seconds the run input stays locked after a wall jump.
   *
   * @readonly written by the engine
   */
  readonly wallLock: number;
  /**
   * Written: seconds one-way platforms stay passable after dropping through.
   *
   * @readonly written by the engine
   */
  readonly dropTime: number;
  /**
   * Written: `jump` as the last step saw it.
   *
   * @readonly written by the engine
   */
  readonly jumpWasHeld: boolean;
  /**
   * Written: the entity it stands on (0 in the air); a moving one carries it.
   *
   * @readonly written by the engine
   */
  readonly ground: Entity;
}

/** `CharacterEvent.kind`: a jump from the ground (or in coyote time). */
export declare const CHARACTER_JUMPED: 1;

/** `CharacterEvent.kind`: touched the ground after being in the air. */
export declare const CHARACTER_LANDED: 2;

/** `CharacterEvent.kind`: a jump off a wall. */
export declare const CHARACTER_WALL_JUMPED: 3;

/** `CharacterEvent.kind`: a jump in the air (PlatformerController.airJumps). */
export declare const CHARACTER_AIR_JUMPED: 4;

/** `CharacterEvent.kind`: left the ground without jumping (ran off an edge). */
export declare const CHARACTER_LEFT_GROUND: 5;

/** `CharacterEvent.kind`: hit a ceiling while rising. */
export declare const CHARACTER_BUMPED: 6;

/**
 * Something a CharacterController or PlatformerController did this step: the hooks for dust, squash, sounds and camera kicks. Read the next tick: the controllers move in the physics step, after the game's systems, so a jump pressed in tick T moves the character in T and its CHARACTER_JUMPED is read in T + 1 (og-feel §12 has the same-tick alternative).
 *
 * The payload of the `CharacterEvent` event (token `og.CharacterEvent`).
 */
export interface CharacterEvent {
  /** The character. */
  readonly entity: Entity;
  /**
   * CHARACTER_JUMPED 1, LANDED 2, WALL_JUMPED 3, AIR_JUMPED 4, LEFT_GROUND 5, BUMPED 6.
   *
   * @range 0..6
   */
  readonly kind: number;
  /**
   * The vertical speed at the moment (a landing's impact speed is negative).
   *
   * @unit mps (metres per second)
   */
  readonly speed: number;
}

/** `Vehicle.drive`: the rear wheels drive. */
export declare const DRIVE_REAR: 0;

/** `Vehicle.drive`: the front wheels drive. */
export declare const DRIVE_FRONT: 1;

/** `Vehicle.drive`: every wheel drives. */
export declare const DRIVE_ALL: 2;

/**
 * A car on a dynamic chassis (the entity's og.RigidBody and og.Collider). 3D: a raycast car with four wheels (suspension, grip, steering on the front axle); its forward is the chassis's local +z (three.js Object3D.lookAt aims +z). 2D: two wheel bodies on sprung wheel joints (front at +x), driven by a motor. The wheels are not entities: og.physics.vehicleWheels gives their poses (and each tyre's slip) for drawing, smoke and skid marks.
 * The defaults drive a road car of about 1,000 kg: give the chassis that mass (og.RigidBody.mass 1000), since a car-sized box at the collider's density 1 weighs a few kilograms and the engine's and brakes' newtons would throw it. The suspension and the grip scale with the chassis's mass (Bullet's units), the engine and the brakes do not: a heavier car wants proportionally more engineForce and brakeForce. og-feel §12 has the numbers a game starts from.
 * The wheel rays meet what the chassis's collider would (its og.Collider layers against the other's mask, and the other's layers against its mask), so two cars on layers that do not meet (a ghost beside the player) pass through each other.
 *
 * The value of the `Vehicle` component (token `og.Vehicle`).
 */
export interface Vehicle {
  /**
   * The accelerator, -1 (reverse) to 1. Set it every tick.
   *
   * @range -1..1
   */
  throttle: number;
  /**
   * The steering, -1 (left) to 1 (right); 3D only.
   *
   * @range -1..1
   */
  steer: number;
  /**
   * The brake, 0 to 1.
   *
   * @range 0..1
   */
  brake: number;
  /**
   * The distance between the front and rear axles.
   *
   * @default 2.6
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  wheelBase: number;
  /**
   * The distance between the left and right wheels (3D).
   *
   * @default 1.6
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  track: number;
  /**
   * The wheels' radius.
   *
   * @default 0.4
   * @range 0.01..
   * @unit m (metres)
   * @tunable
   */
  wheelRadius: number;
  /**
   * The height of the wheel mounts below the chassis centre.
   *
   * @default 0.2
   * @unit m (metres)
   * @tunable
   */
  mountHeight: number;
  /**
   * The suspension's length at rest.
   *
   * @default 0.35
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  suspensionRest: number;
  /**
   * How far the suspension compresses or extends from rest.
   *
   * @default 0.25
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  suspensionTravel: number;
  /**
   * The suspension's stiffness.
   *
   * @default 30
   * @range 0..
   * @tunable
   */
  suspensionStiffness: number;
  /**
   * The suspension's damping.
   *
   * @default 4
   * @range 0..
   * @tunable
   */
  suspensionDamping: number;
  /**
   * The tyres' grip (3D: the friction slip, how much of its load a tyre holds before it slides: 2 grips a road car through a bend, below 1 it slides like a drift car; 2D: the wheels' friction).
   *
   * @default 2
   * @range 0..
   * @tunable
   */
  grip: number;
  /**
   * The engine's force at full throttle (3D: per driven wheel, in newtons: 2,500 on the two rear wheels push 1,000 kg at 5 m/s²; 2D: the motor's torque, N·m).
   *
   * @default 2500
   * @range 0..
   * @tunable
   */
  engineForce: number;
  /**
   * The top speed: the engine's force fades out towards it (throttle against the motion brakes first).
   *
   * @default 40
   * @range 0..
   * @unit mps (metres per second)
   * @tunable
   */
  maxSpeed: number;
  /**
   * The brakes' force at full brake (3D: per wheel, in newtons, every wheel braking: 2,000 stop 1,000 kg at 8 m/s²; 2D: the wheels' braking torque).
   *
   * @default 2000
   * @range 0..
   * @tunable
   */
  brakeForce: number;
  /**
   * The front wheels' largest steering angle (3D).
   *
   * @default 0.55
   * @range 0..1.2
   * @unit rad (radians)
   * @tunable
   */
  maxSteer: number;
  /**
   * DRIVE_REAR 0, DRIVE_FRONT 1, DRIVE_ALL 2.
   *
   * @range 0..2
   */
  drive: number;
  /**
   * The wheels' mass (2D: the wheel bodies; 3D ignores it).
   *
   * @default 20
   * @range 0.01..
   * @unit kg (kilograms)
   * @tunable
   */
  wheelMass: number;
  /**
   * How much the tyres' sideways grip rolls the chassis in a bend (3D): 0 pushes at the centre of mass's height, so the car corners flat and never tips over from steering (an arcade car); towards 1 at the road, the roll that tips a tall car over (rapier's own car uses 0.1, which rolls a 1,000 kg car at full lock above about 20 m/s). Raise it a little for body roll, with og.RigidBody.centerOfMass lowered to keep the car on its wheels.
   *
   * @range 0..1
   * @tunable
   */
  rollInfluence: number;
  /**
   * Written: the forward speed (m/s; negative reversing).
   *
   * @unit mps (metres per second)
   * @readonly written by the engine
   */
  readonly speed: number;
  /**
   * Written: how many wheels touch the ground.
   *
   * @readonly written by the engine
   */
  readonly contacts: number;
  /**
   * Written: the largest slip of its wheels (og.physics.vehicleWheels' `slip`): 0 every tyre grips, towards 1 one slides; for a squeal's volume or smoke without reading the wheels (3D; 0 in 2D).
   *
   * @range 0..1
   * @readonly written by the engine
   */
  readonly slip: number;
}

/**
 * A Verlet rope: `segments` links from the entity's world position (its og.Transform after its parents'; plus `offset`) to `end` (an entity, plus `endOffset`) or hanging free, swung by gravity and wind, colliding with every solid collider at `radius`. It moves nothing (one-way: a rope joint ties bodies together, og.Joint JOINT_ROPE). Its points are world state (the og.physics state block); og.physics.ropePoints reads them and the renderer draws the rope as a strip of `width` and `color`.
 *
 * The value of the `Rope` component (token `og.Rope`).
 */
export interface Rope {
  /**
   * The number of links (points minus one).
   *
   * @default 16
   * @range 1..256
   */
  segments: number;
  /**
   * The rope's length at rest.
   *
   * @default 4
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  length: number;
  /**
   * The start's offset from the entity's position (world axes).
   *
   * @unit m (metres)
   */
  offset: Vector3;
  /** The entity the far end is tied to; 0 leaves it free. */
  end: Entity;
  /**
   * The far end's offset from `end`'s position.
   *
   * @unit m (metres)
   */
  endOffset: Vector3;
  /**
   * Constraint passes per step: more is stiffer, at more cost.
   *
   * @default 12
   * @range 1..64
   * @tunable
   */
  iterations: number;
  /**
   * How much of its velocity each point keeps per tick (1: none lost).
   *
   * @default 0.99
   * @range 0..1
   * @tunable
   */
  damping: number;
  /**
   * Multiplies og.Physics.gravity.
   *
   * @default 1
   * @range -10..10
   * @tunable
   */
  gravityScale: number;
  /**
   * A steady acceleration (wind).
   *
   * @unit mps2 (metres per second squared)
   * @tunable
   */
  wind: Vector3;
  /**
   * The thickness it collides at (0: no collision).
   *
   * @default 0.05
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  radius: number;
  /**
   * The drawn width.
   *
   * @default 0.08
   * @range 0..
   * @unit m (metres)
   */
  width: number;
  /**
   * The drawn linear colour.
   *
   * @default { x: 0.55, y: 0.42, z: 0.28 }
   */
  color: Vector3;
  /**
   * Whether the renderer draws it.
   *
   * @default true
   */
  visible: boolean;
}

/** `Cloth.pins` bit: the whole top row is pinned. */
export declare const PIN_TOP: 1;

/** `Cloth.pins` bit: the top-left and top-right corners are pinned. */
export declare const PIN_TOP_CORNERS: 2;

/** `Cloth.pins` bit: the whole left column is pinned (a flag on a pole). */
export declare const PIN_LEFT: 4;

/** `Cloth.pins` bit: every fourth point of the top row (curtain rings). */
export declare const PIN_TOP_RINGS: 8;

/**
 * A Verlet cloth: a `columns` × `rows` grid of points, `spacing` apart, hanging from the entity's world position (its og.Transform after its parents': its top-left corner, the grid laid out along the entity's local x and down -y), pinned by `pins` (pinned points follow the entity), with stretch and shear constraints, gravity, wind, collision against every solid collider and, with `thickness`, against itself. One-way like og.Rope; its points are world state. Drawn as a double-sided mesh with `material` (og.render.standardMaterial; 0: a plain cloth colour).
 *
 * The value of the `Cloth` component (token `og.Cloth`).
 */
export interface Cloth {
  /**
   * Points across.
   *
   * @default 12
   * @range 2..64
   */
  columns: number;
  /**
   * Points down.
   *
   * @default 12
   * @range 2..64
   */
  rows: number;
  /**
   * The rest distance between neighbouring points.
   *
   * @default 0.15
   * @range 0.001..
   * @unit m (metres)
   * @tunable
   */
  spacing: number;
  /**
   * Which points are pinned, a sum of PIN_* bits (1 top row, 2 top corners, 4 left column, 8 top rings).
   *
   * @default 1
   */
  pins: number;
  /**
   * Constraint passes per step.
   *
   * @default 8
   * @range 1..64
   * @tunable
   */
  iterations: number;
  /**
   * How much of its velocity each point keeps per tick.
   *
   * @default 0.985
   * @range 0..1
   * @tunable
   */
  damping: number;
  /**
   * Multiplies og.Physics.gravity.
   *
   * @default 1
   * @range -10..10
   * @tunable
   */
  gravityScale: number;
  /**
   * A steady acceleration (wind); the cloth flutters with it.
   *
   * @unit mps2 (metres per second squared)
   * @tunable
   */
  wind: Vector3;
  /**
   * How much the wind varies over the cloth and the tick (0: steady).
   *
   * @default 0.6
   * @range 0..2
   * @tunable
   */
  gust: number;
  /**
   * The thickness it collides at (0: no collision).
   *
   * @default 0.04
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  radius: number;
  /**
   * Self-collision: how far apart its points keep from the points of the cloth that are not their near neighbours (0: none). About `spacing` keeps a cloth that folds or piles up from passing through itself.
   *
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  thickness: number;
  /**
   * Simulation steps per tick, each integrating and then solving `iterations` passes; with self-collision enough that a point moves less than `thickness` per step (a point moves at most half the thickness a step).
   *
   * @default 1
   * @range 1..16
   */
  substeps: number;
  /**
   * How much of its sliding a point touching a solid loses each step (0: it slides freely; near 1 it sticks where it lands, so a cloth lowered onto the floor folds instead of spreading).
   *
   * @range 0..1
   * @tunable
   */
  friction: number;
  /** The material it is drawn with (og.render.standardMaterial); 0 is a plain cloth. */
  material: Material;
  /**
   * Whether the renderer draws it.
   *
   * @default true
   */
  visible: boolean;
}

/**
 * A 2D particle fluid (any profile, in the xy plane): `count` particles spawned in a `size` box centred on the entity, simulated with double density relaxation (Clavet et al. 2005: pressure, near pressure, viscosity) on a fixed grid in a fixed order, so it is deterministic and part of the og.physics state block. Particles collide with every solid collider (pushed out of bodies; they push nothing back) and stay inside `bounds` around the entity when it is set. Fluids of one `group` interact: by `density` they layer, by `mixing` they blend. Drawn as metaballs: the particles blurred into a field the renderer thresholds into one surface of `color`, on sprite layer `layer`.
 *
 * The value of the `Fluid2D` component (token `og.Fluid2D`).
 */
export interface Fluid2D {
  /**
   * How many particles (spawned when the component appears or count changes).
   *
   * @default 1000
   * @range 1..8192
   */
  count: number;
  /**
   * The spawn box, centred on the entity.
   *
   * @default { x: 4, y: 3 }
   * @unit m (metres)
   */
  size: Vector2;
  /**
   * The container around the entity (0 0: none; colliders still hold it).
   *
   * @unit m (metres)
   */
  bounds: Vector2;
  /**
   * The interaction radius: particles closer than this push and drag each other.
   *
   * @default 0.18
   * @range 0.01..
   * @unit m (metres)
   * @tunable
   */
  radius: number;
  /**
   * The density it settles at (neighbours within radius, weighted).
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  restDensity: number;
  /**
   * The pressure's strength (above about 0.15 it gets springy, then unstable).
   *
   * @default 0.1
   * @range 0..
   * @tunable
   */
  stiffness: number;
  /**
   * The near pressure's strength (keeps particles apart, gives surface tension).
   *
   * @default 0.5
   * @range 0..
   * @tunable
   */
  nearStiffness: number;
  /**
   * Linear viscosity (honey above 1).
   *
   * @default 0.15
   * @range 0..4
   * @tunable
   */
  viscosity: number;
  /**
   * Fluids sharing a group above 0 push, drag and (by `mixing`) mix with each other as one fluid, at most four to a group (in entity order; a fifth is simulated alone), and are drawn as one surface whose colour follows the mix; 0 keeps the fluid to itself.
   */
  group: number;
  /**
   * How heavy its particles are against the group's other fluids: a heavier fluid sinks under a lighter one.
   *
   * @default 1
   * @range 0.05..20
   * @tunable
   */
  density: number;
  /**
   * How readily it mixes with the group's other fluids: 0 keeps them apart in layers (the boundary held), 1 blends them where they touch (their mix, colour and weight even out).
   *
   * @range 0..1
   * @tunable
   */
  mixing: number;
  /**
   * Multiplies og.Physics.gravity.
   *
   * @default 1
   * @range -10..10
   * @tunable
   */
  gravityScale: number;
  /**
   * Simulation steps per tick.
   *
   * @default 2
   * @range 1..8
   */
  substeps: number;
  /** The seed of the spawn jitter. */
  seed: number;
  /**
   * The surface's linear colour.
   *
   * @default { x: 0.15, y: 0.45, z: 0.95 }
   */
  color: Vector3;
  /**
   * The surface's opacity.
   *
   * @default 0.9
   * @range 0..1
   */
  opacity: number;
  /**
   * The drawn size of each particle's blob (times radius).
   *
   * @default 1.6
   * @range 0.1..4
   */
  blobScale: number;
  /**
   * The field value where the surface starts (higher: thinner, separate drops).
   *
   * @default 0.45
   * @range 0.01..4
   */
  threshold: number;
  /** The sprite layer it is drawn on. */
  layer: number;
  /**
   * Whether the renderer draws it.
   *
   * @default true
   */
  visible: boolean;
}

/**
 * A 3D particle fluid (the 3D profile): `count` particles spawned in a `size` box centred on the entity, simulated with double density relaxation (Clavet et al. 2005: pressure, near pressure that gives surface tension, viscosity) over a spatial hash in a fixed order, so it is deterministic and part of the og.physics state block. Particles collide with every solid collider, the terrain's height field included (pushed out; they push nothing back), and stay inside `bounds` around the entity when it is set. An emitter adds particles at `emitRate` up to `maxCount`; a sink box removes those that enter it. Fluids of one `group` interact: by `density` they layer, by `mixing` they blend. Drawn as a screen-space surface: the particles' depth smoothed into one surface, refracting what is behind by its thickness and reflecting the sky, lit by the scene's lights. Cost: about 0.9 ms of CPU a tick for 2000 particles at 1 substep, 1.8 ms at 2 (native and wasm alike), growing with the particles; keep a scene's fluids to a few thousand particles. The surface costs about 1.1 ms of GPU (an Apple M4 Pro, the fluid covering a third of a 1280×720 frame) whatever the particle count, growing with the pixels it covers.
 *
 * The value of the `Fluid3D` component (token `og.Fluid3D`).
 */
export interface Fluid3D {
  /**
   * How many particles to spawn in the box (when the component appears or count or seed changes; 0: none, the emitter fills it).
   *
   * @default 2000
   * @range ..32768
   */
  count: number;
  /**
   * The spawn box, centred on the entity.
   *
   * @default { x: 2, y: 1, z: 2 }
   * @unit m (metres)
   */
  size: Vector3;
  /**
   * The container around the entity (0 on an axis: open on that axis; colliders and the terrain still hold it).
   *
   * @unit m (metres)
   */
  bounds: Vector3;
  /**
   * The interaction radius: particles closer than this push and drag each other. Cost grows with the particles, not the radius.
   *
   * @default 0.2
   * @range 0.02..
   * @unit m (metres)
   * @tunable
   */
  radius: number;
  /**
   * The density it settles at (neighbours within radius, weighted).
   *
   * @default 10
   * @range 0..
   * @tunable
   */
  restDensity: number;
  /**
   * The pressure's strength (above about 0.05 it jitters, then springs apart).
   *
   * @default 0.02
   * @range 0..
   * @tunable
   */
  stiffness: number;
  /**
   * The near pressure's strength (keeps particles apart, gives surface tension).
   *
   * @default 0.4
   * @range 0..
   * @tunable
   */
  nearStiffness: number;
  /**
   * Linear viscosity (it also calms the surface; honey above 1).
   *
   * @default 0.4
   * @range 0..4
   * @tunable
   */
  viscosity: number;
  /**
   * Fluids sharing a group above 0 push, drag and (by `mixing`) mix with each other as one fluid, at most four to a group (in entity order; a fifth is simulated alone); 0 keeps the fluid to itself.
   */
  group: number;
  /**
   * How heavy its particles are against the group's other fluids: a heavier fluid sinks under a lighter one.
   *
   * @default 1
   * @range 0.05..20
   * @tunable
   */
  density: number;
  /**
   * How readily it mixes with the group's other fluids: 0 keeps them apart in layers (the boundary held), 1 blends them where they touch (their mix, colour and weight even out).
   *
   * @range 0..1
   * @tunable
   */
  mixing: number;
  /**
   * Multiplies og.Physics.gravity.
   *
   * @default 1
   * @range -10..10
   * @tunable
   */
  gravityScale: number;
  /**
   * Simulation steps per tick (each costs a full pass over the particles).
   *
   * @default 2
   * @range 1..8
   */
  substeps: number;
  /** The seed of the spawn and emitter jitter. */
  seed: number;
  /**
   * The most particles alive at once (the emitter stops there).
   *
   * @default 8192
   * @range 1..32768
   */
  maxCount: number;
  /**
   * Particles the emitter adds per second (0: no emitter).
   *
   * @range 0..20000
   * @tunable
   */
  emitRate: number;
  /**
   * Where the emitter is, from the entity (world axes).
   *
   * @unit m (metres)
   */
  emitOffset: Vector3;
  /**
   * The velocity emitted particles start with.
   *
   * @unit mps (metres per second)
   * @tunable
   */
  emitVelocity: Vector3;
  /**
   * The half-size of the cube new particles appear in around the emitter.
   *
   * @default 0.1
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  emitRadius: number;
  /**
   * The sink's centre, from the entity (world axes).
   *
   * @unit m (metres)
   */
  sinkOffset: Vector3;
  /**
   * The sink box: particles that enter it are removed (0 0 0: no sink).
   *
   * @unit m (metres)
   */
  sinkSize: Vector3;
  /**
   * The fluid's linear colour (what light passing through it keeps).
   *
   * @default { x: 0.2, y: 0.55, z: 0.9 }
   */
  color: Vector3;
  /**
   * How opaque a metre of it is (0: clear as glass, 1: opaque).
   *
   * @default 0.6
   * @range 0..1
   */
  opacity: number;
  /**
   * How much it reflects (scales the Fresnel term).
   *
   * @default 1
   * @range 0..2
   */
  reflectivity: number;
  /**
   * How far refraction bends what is seen through it.
   *
   * @default 1
   * @range 0..4
   */
  refraction: number;
  /**
   * The drawn size of each particle (times radius).
   *
   * @default 0.6
   * @range 0.1..2
   */
  drawScale: number;
  /**
   * How much the surface is smoothed (0: beads, 1: one surface).
   *
   * @default 1
   * @range 0..2
   */
  smoothing: number;
  /**
   * Whether the renderer draws it.
   *
   * @default true
   */
  visible: boolean;
}

/**
 * Height-field water (3D): a `resolution` × `resolution` grid of heights over a `size` square centred on the entity (its y is the rest level), waves travelling at `waveSpeed` and fading by `damping`. Dynamic bodies crossing it push waves (splashes; og.Splash events) and float by `buoyancy`. The heights are world state; an og.Water on the same entity draws them.
 *
 * The value of the `WaterSurface` component (token `og.WaterSurface`).
 */
export interface WaterSurface {
  /**
   * Grid points along each side.
   *
   * @default 64
   * @range 8..256
   */
  resolution: number;
  /**
   * The side of the square of water.
   *
   * @default 16
   * @range 0.1..
   * @unit m (metres)
   * @tunable
   */
  size: number;
  /**
   * How fast waves travel.
   *
   * @default 3
   * @range 0..
   * @unit mps (metres per second)
   * @tunable
   */
  waveSpeed: number;
  /**
   * How much of a wave survives a second (1: no loss).
   *
   * @default 0.4
   * @range 0..1
   * @tunable
   */
  damping: number;
  /**
   * How strongly bodies push waves.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  splash: number;
  /**
   * The upward force on a submerged body per cubic metre of its box under water (0: bodies sink).
   *
   * @default 9.81
   * @range 0..
   * @tunable
   */
  buoyancy: number;
  /**
   * How much a submerged body's velocity is slowed.
   *
   * @default 1.5
   * @range 0..
   * @unit perSecond (a rate per second: the fraction of a value that decays in one second)
   * @tunable
   */
  drag: number;
}

/**
 * 2D water for side-scrollers: a `width` × `depth` pool whose top edge (at the entity's y, centred on its x) is `columns` springs, each pulled to its rest height by `tension`, damped by `damping` and passing waves to its neighbours by `spread`. Bodies (and characters) crossing the surface splash it (og.Splash events) and float by `buoyancy`. The springs are world state; an og.Water2D on the same entity draws it.
 *
 * The value of the `WaterLine` component (token `og.WaterLine`).
 */
export interface WaterLine {
  /**
   * The pool's width.
   *
   * @default 12
   * @range 0.1..
   * @unit m (metres)
   * @tunable
   */
  width: number;
  /**
   * The pool's depth below the surface.
   *
   * @default 3
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  depth: number;
  /**
   * Springs along the surface.
   *
   * @default 96
   * @range 4..512
   */
  columns: number;
  /**
   * How hard each spring pulls back to rest.
   *
   * @default 60
   * @range 0..
   * @tunable
   */
  tension: number;
  /**
   * How fast a spring's motion dies.
   *
   * @default 2.5
   * @range 0..
   * @tunable
   */
  damping: number;
  /**
   * How much of a spring's height its neighbours feel each pass (waves travel).
   *
   * @default 0.25
   * @range 0..0.5
   * @tunable
   */
  spread: number;
  /**
   * How strongly crossing bodies splash.
   *
   * @default 0.04
   * @range 0..
   * @tunable
   */
  splash: number;
  /**
   * The upward force on a submerged body per square metre of its box under water (0: bodies sink).
   *
   * @default 12
   * @range 0..
   * @tunable
   */
  buoyancy: number;
  /**
   * How much a submerged body's velocity is slowed.
   *
   * @default 2
   * @range 0..
   * @unit perSecond (a rate per second: the fraction of a value that decays in one second)
   * @tunable
   */
  drag: number;
}

/**
 * Something crossed a water surface (og.WaterSurface or og.WaterLine): for splash particles and sounds. Read the next tick.
 *
 * The payload of the `Splash` event (token `og.Splash`).
 */
export interface Splash {
  /** The water's entity. */
  readonly water: Entity;
  /** What crossed it. */
  readonly entity: Entity;
  /**
   * Where it crossed the surface.
   *
   * @unit m (metres)
   */
  readonly point: Vector3;
  /**
   * Its vertical speed (negative going in).
   *
   * @unit mps (metres per second)
   */
  readonly speed: number;
}

/**
 * A ragdoll over an og.Skeleton: a capsule body per bone with a parent, joints with limits between them (ball joints with cone limits in 3D; in 2D hinges about z, the bones rapier2d bodies in the xy plane at their animated z, for a skeleton laid out in that plane), built from the skeleton's rest pose. `weight` is where the pose comes from: 0 the animation (the bodies follow it kinematically), 1 the physics (the bodies are dynamic, starting with the animation's velocity); the pose drawn blends between them over `blendTime`, both ways. The bodies are world state (the og.physics state block), not entities.
 *
 * The value of the `Ragdoll` component (token `og.Ragdoll`).
 */
export interface Ragdoll {
  /**
   * 0 animated, 1 ragdoll. Set it to 1 on a hit; back to 0 to recover.
   *
   * @range 0..1
   * @tunable
   */
  weight: number;
  /**
   * How long the blend between animation and physics takes, both ways.
   *
   * @default 0.25
   * @range 0..
   * @unit s (seconds of simulation time)
   * @tunable
   */
  blendTime: number;
  /**
   * The whole body's mass, shared by the bones by length.
   *
   * @default 60
   * @range 0.1..
   * @unit kg (kilograms)
   * @tunable
   */
  mass: number;
  /**
   * A bone capsule's radius as a fraction of its length.
   *
   * @default 0.22
   * @range 0.01..1
   * @tunable
   */
  thickness: number;
  /**
   * How far each joint bends from the rest pose.
   *
   * @default 0.9
   * @range 0..3.1
   * @unit rad (radians)
   * @tunable
   */
  jointLimit: number;
  /**
   * How much of their motion the bones lose per second.
   *
   * @default 0.6
   * @range 0..20
   * @unit perSecond (a rate per second: the fraction of a value that decays in one second)
   * @tunable
   */
  damping: number;
  /** An impulse given to every bone when weight first goes up (a hit). */
  impulse: Vector3;
  /**
   * Written: the blend in use (0 animation .. 1 physics).
   *
   * @readonly written by the engine
   */
  readonly blend: number;
}

/**
 * Which colliders a scene query considers. The zero value considers every solid collider on every layer.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface QueryFilter {
  /**
   * Layers to consider: a collider counts when its layers meet this; 0 means every layer.
   *
   * @default 0
   */
  readonly mask?: number;
  /**
   * Also report sensors.
   *
   * @default false
   */
  readonly includeSensors?: boolean;
  /**
   * An entity to ignore, such as the caster itself.
   *
   * @default null
   */
  readonly exclude?: Entity | null;
}

/**
 * A ray for og.physics.raycast.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RaycastDesc {
  /**
   * Where the ray starts.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly origin?: Vector3;
  /**
   * Where it points; it need not be normalised and must not be zero (in 2D, not zero in xy).
   *
   * @default { x: 0, y: -1, z: 0 }
   */
  readonly direction?: Vector3;
  /**
   * How far it reaches; 0 is unbounded.
   *
   * @default 0
   * @range 0..
   * @unit m (metres)
   */
  readonly maxDistance?: number;
  /**
   * Which colliders count.
   *
   * @default {}
   */
  readonly filter?: QueryFilter;
}

/** The nearest collider a ray meets. */
export interface RaycastHit {
  /** The collider's entity. */
  readonly entity: Entity;
  /**
   * From the origin along the normalised direction.
   *
   * @unit m (metres)
   */
  readonly distance: number;
  /** Where the ray meets the collider. */
  readonly point: Vector3;
  /** The surface normal there (z is 0 in 2D). */
  readonly normal: Vector3;
}

/**
 * The shape of a shape cast or an overlap test: the shape fields of a Collider. The zero value is a 1 m box.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ShapeDesc {
  /**
   * One of the SHAPE_* codes.
   *
   * @default 0
   * @range 0..5
   */
  readonly shape?: number;
  /**
   * Full extents of a box.
   *
   * @default { x: 1, y: 1, z: 1 }
   * @unit m (metres)
   */
  readonly size?: Vector3;
  /**
   * Radius of a sphere, circle, capsule or cylinder.
   *
   * @default 0.5
   * @unit m (metres)
   */
  readonly radius?: number;
  /**
   * Capsule or cylinder height.
   *
   * @default 1
   * @unit m (metres)
   */
  readonly height?: number;
  /**
   * The geometry of a convex or triangle-mesh shape.
   *
   * @default 0
   */
  readonly mesh?: Geometry;
}

/**
 * A shape swept along a direction for og.physics.shapeCast.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ShapeCastDesc {
  /**
   * The swept shape.
   *
   * @default {}
   */
  readonly shape?: ShapeDesc;
  /**
   * Where the sweep starts.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly position?: Vector3;
  /**
   * The shape's rotation; a zero quaternion is no rotation.
   *
   * @default { x: 0, y: 0, z: 0, w: 1 }
   */
  readonly rotation?: Quaternion;
  /**
   * Where it sweeps; it need not be normalised and must not be zero.
   *
   * @default { x: 0, y: -1, z: 0 }
   */
  readonly direction?: Vector3;
  /**
   * How far it sweeps; 0 is unbounded.
   *
   * @default 0
   * @range 0..
   * @unit m (metres)
   */
  readonly maxDistance?: number;
  /**
   * Which colliders count.
   *
   * @default {}
   */
  readonly filter?: QueryFilter;
}

/** The first collider a swept shape meets. */
export interface ShapeCastHit {
  /** The collider's entity. */
  readonly entity: Entity;
  /**
   * How far the shape travelled before touching.
   *
   * @unit m (metres)
   */
  readonly distance: number;
  /** The contact point on the hit collider, in world space. */
  readonly point: Vector3;
  /** The hit collider's surface normal there, in world space. */
  readonly normal: Vector3;
}

/**
 * A shape placed in the world for og.physics.overlap.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface OverlapDesc {
  /**
   * The shape.
   *
   * @default {}
   */
  readonly shape?: ShapeDesc;
  /**
   * Where it is.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly position?: Vector3;
  /**
   * Its rotation; a zero quaternion is no rotation.
   *
   * @default { x: 0, y: 0, z: 0, w: 1 }
   */
  readonly rotation?: Quaternion;
  /**
   * Which colliders count.
   *
   * @default {}
   */
  readonly filter?: QueryFilter;
}

/**
 * Geometry for convex and triangle-mesh colliders, in the mesh's own space.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface MeshDesc {
  /**
   * Vertex positions (2D polygons use x and y).
   *
   * @default []
   */
  readonly positions?: readonly Vector3[];
  /**
   * Triangles as triples of indices into positions; a convex hull ignores them.
   *
   * @default []
   */
  readonly indices?: readonly number[];
}









/** A wheel of an og.Vehicle as the last step left it, for drawing. */
export interface WheelPose {
  /**
   * The wheel's centre in world space.
   *
   * @unit m (metres)
   */
  readonly position: Vector3;
  /** Its rotation (steering and spin) in world space. */
  readonly rotation: Quaternion;
  /** Whether it touches the ground. */
  readonly contact: boolean;
  /**
   * How much the tyre slides (rapier's skid info turned round): 0 it grips, towards 1 it asks for more than its grip holds (0.5 is twice as much); 0 in the air and in 2D. Start smoke and skid marks above about 0.2, and fade a squeal in with it.
   *
   * @range 0..1
   */
  readonly slip: number;
  /**
   * How fast the tyre's contact moves sideways over the ground (along its axle; 0 in the air and in 2D): a slide's speed, for the squeal's pitch or a skid mark's darkness.
   *
   * @unit mps (metres per second)
   */
  readonly sideSpeed: number;
}

/**
 * One bone of a ragdoll rig: a capsule from the bone's position along `axis` for `length`.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RagdollBoneDesc {
  /**
   * The parent bone's index (it must come earlier); -1 for the root.
   *
   * @default -1
   */
  readonly parent?: number;
  /**
   * The bone's rest position relative to the entity.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly position?: Vector3;
  /**
   * The bone's rest rotation relative to the entity; a zero quaternion is no rotation.
   *
   * @default { x: 0, y: 0, z: 0, w: 1 }
   */
  readonly rotation?: Quaternion;
  /**
   * How long the bone is (to its child).
   *
   * @default 0.25
   * @unit m (metres)
   */
  readonly length?: number;
  /**
   * The direction the bone points in its own frame (towards its child).
   *
   * @default { x: 0, y: 1, z: 0 }
   */
  readonly axis?: Vector3;
  /**
   * The skeleton joint (its index in the skin) the bone stands for, so the model is drawn with it; -1 for none.
   *
   * @default -1
   */
  readonly joint?: number;
}

/**
 * The bones of a ragdoll, parents first, for og.physics.setRagdollRig; og.physics.ragdollRigFromSkeleton makes one from a model's skeleton.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RagdollRigDesc {
  /**
   * The bones.
   *
   * @default []
   */
  readonly bones?: readonly RagdollBoneDesc[];
  /**
   * The skeleton the bones' joints belong to (0: none): the entity's posed model is drawn blended towards the bones by og.Ragdoll.blend.
   *
   * @default 0
   */
  readonly skeleton?: SkeletonAsset;
}

/**
 * A ragdoll bone's pose as the last step left it (blended with the animation by og.Ragdoll.weight).
 */
export interface BonePose {
  /**
   * The bone's position in world space.
   *
   * @unit m (metres)
   */
  readonly position: Vector3;
  /** Its rotation in world space. */
  readonly rotation: Quaternion;
}










/** One phase of the last physics step, for og.physics.stats. */
export interface PhysicsPhase {
  /**
   * gather (reading the components), sync (bodies and static colliders into rapier), joints (joints and vehicles), ragdolls, water (buoyancy before the step), rapier (rapier's step, every substep), readout (touching pairs and bodies read back, the rest rule), controllers (characters and platformers), soft (ropes and cloths), fluids (2D), fluids3d or writeBack (components and events written back).
   */
  readonly name: string;
  /**
   * Wall-clock time it took (0 where the host has no clock); never world state.
   *
   * @unit ms (milliseconds)
   */
  readonly ms: number;
  /**
   * Whether any of its work ran on the lanes pool: rapier's stages when the step went to the pool, the other phases when they split by bodies, contact pairs, ragdolls, soft bodies or particles.
   */
  readonly split: boolean;
}

/**
 * What the last physics step cost, phase by phase, and where it ran: for finding where physics time goes. Timings are wall clock and vary run to run; the simulation never reads them.
 */
export interface PhysicsStats {
  /** The tick the step ran (0 before the first). */
  readonly tick: number;
  /** The rapier bodies it stepped (with ragdoll bones and vehicle wheels). */
  readonly bodies: number;
  /** The lanes pool's threads at the time (1: lanes off or single-threaded). */
  readonly threads: number;
  /**
   * Whether the whole step ran on the lanes pool (from a number of bodies per thread count), where rapier splits its broad phase, narrow phase and solver.
   */
  readonly pool: boolean;
  /**
   * Wall-clock time of the whole og.physics system, gather to write-back.
   *
   * @unit ms (milliseconds)
   */
  readonly ms: number;
  /** Every phase, in the order they run. */
  readonly phases: readonly PhysicsPhase[];
}


// ── atmosphere ──────────────────────────────────────────────────
// Fog, a procedural sky, the day-night cycle, weather and water, declared as resources and components (the E5 round). Everything is off by default: a world that sets none of them draws exactly as before.
// The day-night clock is a function of the tick (DayNight.start plus the simulated time times speed), so it is the same on every run and after a restore; rendering reads the world and never writes it.

/** Weather.kind: clear. */
export declare const WEATHER_NONE: 0;

/**
 * Weather.kind: rain streaks falling with the wind, the light dimmed and a grey distance fog.
 */
export declare const WEATHER_RAIN: 1;

/** Weather.kind: flakes drifting down on gusts, a pale fog. */
export declare const WEATHER_SNOW: 2;

/** Weather.kind: dust blowing along the wind near the ground, a warm thick fog. */
export declare const WEATHER_DUST: 3;

/** Weather.kind: no particles, a dense low height fog. */
export declare const WEATHER_MIST: 4;

/**
 * Distance and height fog over the 3D scene and world sprites: distance fog thickens exponentially by density from start (or linearly from start to end when end > start), height fog is densest below height and thins by falloff per metre above it; looking into the sun it turns towards sunColor. The overlay is never fogged.
 *
 * The `Fog` resource (token `og.Fog`).
 */
export interface Fog {
  /** Draw the fog. */
  enabled: boolean;
  /**
   * The colour the fog tends to (linear, before exposure).
   *
   * @default { r: 0.62, g: 0.68, b: 0.76 }
   */
  color: Color;
  /**
   * Distance fog per metre (0: none).
   *
   * @default 0.02
   * @range 0..2
   * @tunable
   */
  density: number;
  /**
   * Where distance fog starts.
   *
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  start: number;
  /**
   * With end > start, a linear ramp to full fog at end as well.
   *
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  end: number;
  /**
   * Height fog at and below height (0: none).
   *
   * @range 0..10
   * @tunable
   */
  heightDensity: number;
  /**
   * The world height below which height fog is densest.
   *
   * @unit m (metres)
   * @tunable
   */
  height: number;
  /**
   * How fast height fog thins above height, per metre.
   *
   * @default 0.5
   * @range 0.0001..20
   * @tunable
   */
  falloff: number;
  /**
   * The fog's colour looking into the sun (forward scattering).
   *
   * @default { r: 1, g: 0.86, b: 0.62 }
   */
  sunColor: Color;
  /**
   * How tight the sun's glow in the fog is (higher: smaller).
   *
   * @default 8
   * @range 0..256
   * @tunable
   */
  sunPower: number;
  /**
   * The most the fog hides.
   *
   * @default 1
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  maxOpacity: number;
  /**
   * How much of the fog covers the sky (0: the sky stays clear).
   *
   * @default 1
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  sky: number;
  /**
   * Light the fog as a participating medium: within volumetricDistance of the camera the fog (density per metre, thickened below height by heightDensity and falloff, as above) scatters the sun through its shadow cascades, so light shafts fall through windows and between trees and move with the sun, and every point, spot and rect light it is lit by (up to RenderCapabilities.volumetricLights at each sample); it dims what lies behind it by the same amounts. RenderCapabilities.volumetricFog names the method (a froxel grid on the full tier, a raymarch on WebGL2). Opaque surfaces and the sky are seen through the volume; blended surfaces, particles and sprites take the plain distance and height fog of the same medium. Past volumetricDistance nothing more is added.
   */
  volumetric: boolean;
  /**
   * How much of the light the fog takes from a ray it scatters (its albedo, tinted by color): 0 dims without glowing, 1 sends all of it on, above 1 brightens the shafts for effect.
   *
   * @default 0.8
   * @range 0..4
   * @tunable
   */
  scattering: number;
  /**
   * Which way the fog scatters (Henyey–Greenstein g): 0 evenly, towards 1 forward (bright looking into the light: shafts against the sun), below 0 back towards the light.
   *
   * @default 0.6
   * @range -0.95..0.95
   * @tunable
   */
  anisotropy: number;
  /**
   * How far from the camera the volume reaches.
   *
   * @default 48
   * @range 4..400
   * @unit m (metres)
   */
  volumetricDistance: number;
}

/**
 * A procedural sky of chosen colours, drawn behind the scene per pixel and lighting it (the image-based light follows it). Without it, RenderSettings.sky draws the built-in sky. A DayNight that is enabled sets the sky itself.
 *
 * The `Sky` resource (token `og.Sky`).
 */
export interface Sky {
  /** Draw and light by this sky. */
  enabled: boolean;
  /**
   * Overhead.
   *
   * @default { r: 0.18, g: 0.36, b: 0.78 }
   */
  zenith: Color;
  /**
   * At the horizon.
   *
   * @default { r: 0.78, g: 0.84, b: 0.92 }
   */
  horizon: Color;
  /**
   * Below the horizon.
   *
   * @default { r: 0.16, g: 0.14, b: 0.12 }
   */
  ground: Color;
  /**
   * Towards the sun (match your DirectionalLight).
   *
   * @default { x: 0.45, y: 0.8, z: 0.35 }
   */
  sunDirection: Vector3;
  /**
   * The sun's colour.
   *
   * @default { r: 1, g: 0.95, b: 0.88 }
   */
  sunColor: Color;
  /**
   * The sun disc's angular radius (0: no disc).
   *
   * @default 1.8
   * @range 0..20
   * @unit deg (degrees)
   * @tunable
   */
  sunSize: number;
  /**
   * The disc's brightness.
   *
   * @default 40
   * @range 0..
   * @tunable
   */
  sunIntensity: number;
  /**
   * The glow around the sun.
   *
   * @default 0.6
   * @range 0..
   * @tunable
   */
  glow: number;
  /**
   * How tight the glow is.
   *
   * @default 24
   * @range 1..
   * @tunable
   */
  glowPower: number;
  /**
   * Stars where the sky is dark: 0 none, 1 a full night sky.
   *
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  stars: number;
  /**
   * A multiplier on the whole sky.
   *
   * @default 1.1
   * @range 0..
   * @tunable
   */
  brightness: number;
  /**
   * The moon disc's angular radius (0: no moon). It sets below the horizon and hides the stars behind it.
   *
   * @range 0..20
   * @unit deg (degrees)
   * @tunable
   */
  moonSize: number;
  /**
   * How much of the moon is lit: 0 new, 0.25 the first quarter (its right half), 0.5 full, 0.75 the last quarter (its left half).
   *
   * @default 0.5
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  moonPhase: number;
  /**
   * The lit moon's brightness.
   *
   * @default 1.4
   * @range 0..
   * @tunable
   */
  moonIntensity: number;
  /**
   * The moon's colour.
   *
   * @default { r: 0.9, g: 0.92, b: 1 }
   */
  moonColor: Color;
  /** Towards the moon; zero puts it opposite sunDirection. */
  moonDirection: Vector3;
}

/**
 * A day-night cycle: the time of day drives the sun's direction, the sky's colours, stars and the moon disc, the first DirectionalLight (the sun by day, a dim moon opposite it by night), the ambient light and the fog's colour. The clock is start plus the simulated seconds times speed, wrapped to 24 hours; the image-based light is regenerated every `step` minutes of game time while the sky drawn per pixel moves smoothly. The drive happens as the frame is drawn and never writes the world: the DirectionalLight keeps the colour, intensity and transform it was given. The probe's `get` answers what is drawn, a pure function of the tick: `get:og.DayNight` adds `hours`, `toSun`, `toLight` (towards the sun by day, the moon by night), `lightColor` and `lightIntensity`, and `get:<sun>.og.DirectionalLight` on the driven light answers the drawn `color` and `intensity` with `toLight` and `drivenBy`.
 *
 * The `DayNight` resource (token `og.DayNight`).
 */
export interface DayNight {
  /** Run the cycle. */
  enabled: boolean;
  /**
   * The time of day at tick 0, in hours (6 sunrise, 12 noon, 18 sunset).
   *
   * @default 9
   * @range 0..24
   * @tunable
   */
  start: number;
  /**
   * Game hours per simulated second (0.05: a day in 8 minutes; 0 holds the time).
   *
   * @default 0.05
   * @range 0..24
   * @tunable
   */
  speed: number;
  /**
   * How far the sun's path leans from overhead.
   *
   * @default 25
   * @range 0..89
   * @unit deg (degrees)
   * @tunable
   */
  tilt: number;
  /**
   * Where the sun rises, turned about +y from +x.
   *
   * @unit deg (degrees)
   * @tunable
   */
  azimuth: number;
  /**
   * A multiplier on the sun and moon light.
   *
   * @default 1
   * @range 0..10
   * @tunable
   */
  lightIntensity: number;
  /**
   * Turn the og.Fog colour towards the horizon's.
   *
   * @default true
   */
  tintFog: boolean;
  /**
   * Game minutes between regenerations of the image-based light.
   *
   * @default 10
   * @range 1..120
   * @tunable
   */
  step: number;
  /**
   * The moon disc's angular radius, opposite the sun so it rises as the sun sets (0: no moon).
   *
   * @default 1.6
   * @range 0..20
   * @unit deg (degrees)
   * @tunable
   */
  moonSize: number;
  /**
   * How much of the moon is lit: 0 new, 0.25 the first quarter, 0.5 full, 0.75 the last quarter.
   *
   * @default 0.5
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  moonPhase: number;
}

/**
 * A weather preset around the camera: particles (rain, snow, dust), its fog (unless og.Fog is enabled, which takes precedence) and dimmed sunlight. Particles are simulated by the renderer from the tick, so they look the same on every run; they never touch the simulation.
 *
 * The `Weather` resource (token `og.Weather`).
 */
export interface Weather {
  /**
   * WEATHER_NONE 0, WEATHER_RAIN 1, WEATHER_SNOW 2, WEATHER_DUST 3, WEATHER_MIST 4.
   *
   * @range 0..4
   * @label Kind
   */
  kind: number;
  /**
   * How heavy: scales the particle rate, the fog and the dimming.
   *
   * @default 1
   * @range 0..2
   * @tunable
   */
  intensity: number;
  /**
   * Where the wind blows the particles.
   *
   * @default { x: 1.5, y: 0, z: 0.5 }
   * @unit mps (metres per second)
   * @tunable
   */
  wind: Vector3;
  /**
   * The size of the region around the camera the particles fill.
   *
   * @default 18
   * @range 1..200
   * @unit m (metres)
   * @tunable
   */
  radius: number;
  /**
   * Multiplies the particles' colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  tint: Color;
  /**
   * How much of the preset's dimming of the sunlight applies.
   *
   * @default 1
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  dim: number;
  /**
   * With an orthographic camera (2D), the sprite layer the particles draw in.
   *
   * @default 50
   */
  layer: number;
}

/**
 * A planar water surface on the entity's xz plane (centred on it, turned with it): colour by the water's depth over what is below, foam where it meets geometry, animated waves, refraction and reflections (screen-space where the tier allows, the sky's cubemap on WebGL2, or a planar reflection), the sun's shadows, caustics below it and an underwater view. With an og.WaterSurface on the same entity its wave heights displace the surface (centred on the entity, unrotated).
 *
 * The value of the `Water` component (token `og.Water`).
 */
export interface Water {
  /**
   * The extent along the entity's x and z.
   *
   * @default { x: 16, y: 16 }
   * @unit m (metres)
   * @tunable
   */
  size: Vector2;
  /**
   * Grid cells per side (the height field's detail).
   *
   * @default 64
   * @range 1..256
   */
  segments: number;
  /**
   * Shallow water's colour (tints what is seen through it).
   *
   * @default { r: 0.55, g: 0.85, b: 0.82 }
   */
  shallow: Color;
  /**
   * Deep water's colour.
   *
   * @default { r: 0.02, g: 0.09, b: 0.14 }
   */
  deep: Color;
  /**
   * The depth over which water turns from shallow to deep.
   *
   * @default 2.5
   * @range 0.01..
   * @unit m (metres)
   * @tunable
   */
  depthFade: number;
  /**
   * How opaque deep water gets.
   *
   * @default 0.92
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  opacity: number;
  /**
   * The foam's colour.
   *
   * @default { r: 0.95, g: 0.97, b: 1 }
   */
  foam: Color;
  /**
   * The water depth over which foam rings geometry (0: none).
   *
   * @default 0.35
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  foamWidth: number;
  /**
   * Detail waves per metre.
   *
   * @default 0.6
   * @range 0..
   * @tunable
   */
  waveScale: number;
  /**
   * How fast the detail waves travel.
   *
   * @default 1
   * @tunable
   */
  waveSpeed: number;
  /**
   * How strongly they bend the surface (0: a mirror).
   *
   * @default 0.35
   * @range 0..4
   * @tunable
   */
  waveStrength: number;
  /**
   * How far refraction shifts what is seen through the water.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  refraction: number;
  /**
   * How much it reflects.
   *
   * @default 1
   * @range 0..2
   * @tunable
   */
  reflectivity: number;
  /**
   * Reflect the scene by drawing it again from the camera mirrored in the surface, at this fraction of the screen's resolution (0: off, the screen-space and sky reflections). One surface a frame, the first that asks; it costs about a second scene pass at that resolution.
   *
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  planarReflection: number;
  /**
   * How much the sun's shadows darken the surface: its highlight, foam and colour (0: none).
   *
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  shadows: number;
  /**
   * With the camera below the surface: the water's fog colour.
   *
   * @default { r: 0.03, g: 0.16, b: 0.2 }
   */
  underwaterColor: Color;
  /**
   * Its fog per metre (0: no underwater view). Below the surface the frame's fog becomes the water's, and the surface seen from below shows the sky inside Snell's window.
   *
   * @default 0.09
   * @range 0..2
   * @tunable
   */
  underwaterDensity: number;
  /**
   * Caustics of the sunlight on everything below the surface (0: none). One surface a frame, the first that has them.
   *
   * @range 0..4
   * @tunable
   */
  caustics: number;
  /**
   * The caustics' cells per metre.
   *
   * @default 0.6
   * @range 0.01..10
   * @tunable
   */
  causticsScale: number;
}

/**
 * Draws the og.WaterLine on the same entity in the sprite order: the body from its surface down, a gradient from the top colour to the bottom one, and a bright lip along the surface.
 *
 * The value of the `Water2D` component (token `og.Water2D`).
 */
export interface Water2D {
  /**
   * The body's colour at the surface.
   *
   * @default { r: 0.16, g: 0.5, b: 0.72 }
   */
  top: Color;
  /**
   * Its opacity there.
   *
   * @default 0.72
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  topOpacity: number;
  /**
   * The colour at the bottom.
   *
   * @default { r: 0.02, g: 0.1, b: 0.25 }
   */
  bottom: Color;
  /**
   * Its opacity there.
   *
   * @default 0.92
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  bottomOpacity: number;
  /**
   * The lip along the surface.
   *
   * @default { r: 0.8, g: 0.95, b: 1 }
   */
  lip: Color;
  /**
   * How thick the lip is (0: none).
   *
   * @default 0.06
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  lipWidth: number;
  /** The sprite layer it is drawn in. */
  layer: number;
}


// ── anim ────────────────────────────────────────────────────────
// Skeletal animation over og-anim: glTF skins and clips (cooked into the `<model>#anim` blob, loaded with og.load.animation), blend-tree graphs with layered state machines, root motion, two-bone and FABRIK IK and blendshapes, evaluated by the engine's og.anim system after the game's systems and before physics.
// Game code owns the components and sets players, graph parameters and IK targets; og.anim writes the pose (kept as world state in the `og.anim` state block, so snapshots, restore and the world hash cover it), the joint palette, root motion into og.Transform and the fields marked written. Asset handles are what og.load.animation and og.anim.graph answer; 0 is none.
// Clips bind to a skeleton's nodes by name, so a clip of one rig plays on another with the same node names; a rig with other names or proportions plays it retargeted (og.anim.retarget at run time, or a model's `animation { library }` import setting at cook time). Assets are not world state: a hot reload registers them again.

/**
 * A rig: a glTF's nodes in topological order with their rest transforms, and its skins (joints and inverse bind matrices).
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type SkeletonAsset = number & { readonly [handleType]: "SkeletonAsset" };

/**
 * An animation clip, resampled at the cook's rate, with its markers.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type ClipAsset = number & { readonly [handleType]: "ClipAsset" };

/**
 * A blend-tree graph (`type animgraph`) whose clip references were resolved to clip handles.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type GraphAsset = number & { readonly [handleType]: "GraphAsset" };

/**
 * A SkeletonAsset handle field.
 *
 * A `SkeletonAsset` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function SkeletonAsset(value?: SkeletonAsset, options?: FieldOptions): FieldSpec<"SkeletonAsset">;

/**
 * A ClipAsset handle field.
 *
 * A `ClipAsset` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function ClipAsset(value?: ClipAsset, options?: FieldOptions): FieldSpec<"ClipAsset">;

/**
 * A GraphAsset handle field.
 *
 * A `GraphAsset` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function GraphAsset(value?: GraphAsset, options?: FieldOptions): FieldSpec<"GraphAsset">;

/**
 * What the root joint's motion does to the entity.
 * - `"off"`: The root animates in place; the entity does not move.
 * - `"planar"`: The root's horizontal (xz) travel and yaw move og.Transform.
 * - `"full"`: The root's whole translation and rotation move og.Transform.
 */
export type RootMotion = "off" | "planar" | "full";

/**
 * A RootMotion field: the variant's index.
 *
 * A `RootMotion` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function RootMotion(value?: RootMotion, options?: FieldOptions): FieldSpec<"RootMotion">;

/**
 * Who decides blendshape weights.
 * - `"component"`: Game code writes w0...w7.
 * - `"clips"`: The playing clips' weight channels write them.
 */
export type BlendshapeSource = "component" | "clips";

/**
 * A BlendshapeSource field: the variant's index.
 *
 * A `BlendshapeSource` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function BlendshapeSource(value?: BlendshapeSource, options?: FieldOptions): FieldSpec<"BlendshapeSource">;

/**
 * How an IK chain is solved.
 * - `"twoBone"`: Analytic two-bone (thigh, shin, foot), bending towards the pole.
 * - `"fabrik"`: FABRIK over `bones` bones, iteratively.
 */
export type IkSolver = "twoBone" | "fabrik";

/**
 * An IkSolver field: the variant's index.
 *
 * A `IkSolver` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function IkSolver(value?: IkSolver, options?: FieldOptions): FieldSpec<"IkSolver">;

/**
 * What an IK chain's target and pole are relative to.
 * - `"world"`: Positions in the frame the skeleton entity's og.Transform is written in (its position, rotation and scale are undone): true world for a skeleton entity without a parent, its parent's frame for a child (a body under a character capsule takes targets relative to the capsule; transform world points by the parent's inverse yourself).
 * - `"model"`: The skeleton's model space.
 */
export type IkSpace = "world" | "model";

/**
 * An IkSpace field: the variant's index.
 *
 * A `IkSpace` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function IkSpace(value?: IkSpace, options?: FieldOptions): FieldSpec<"IkSpace">;

/**
 * A posed skeleton: the entity is drawn skinned by the joint palette og.anim writes each tick. The pose itself (every node's local transform) is world state kept by the `og.anim` state block.
 *
 * The value of the `Skeleton` component (token `og.Skeleton`).
 */
export interface Skeleton {
  /** The skeleton asset (og.load.animation's `skeleton`); 0 is none. */
  asset: SkeletonAsset;
  /**
   * Which of the asset's skins the joint palette is for (glTF skin index).
   *
   * @range 0..
   */
  skin: number;
  /**
   * Root motion extraction: off (animate in place), planar (xz travel and yaw move og.Transform) or full.
   */
  rootMotion: RootMotion;
  /**
   * The joint whose motion is extracted (an index into the skin's joints; 0 is its first joint, usually the hips).
   *
   * @range 0..
   */
  rootJoint: number;
  /**
   * Written: the number of joints in the palette (0 until the first tick).
   *
   * @readonly written by the engine
   */
  readonly jointCount: number;
}

/**
 * Plays one clip on a skeleton. Alone it poses the skeleton from its rest pose; with an og.AnimationGraph it plays over the graph's result with `weight`. Changing `clip` cross-fades from the previous one over `fade`; a clip change starts the new clip at 0 (or at `time` if the game wrote it in the same tick).
 *
 * The value of the `AnimationPlayer` component (token `og.AnimationPlayer`).
 */
export interface AnimationPlayer {
  /** The clip asset; 0 plays nothing. */
  clip: ClipAsset;
  /**
   * The playback position; written every tick, set it to seek (no markers fire across a seek).
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  time: number;
  /**
   * Playback rate (1 = as authored, negative plays backwards).
   *
   * @default 1
   * @range -10..10
   * @tunable
   */
  speed: number;
  /**
   * Wrap at the end of the clip; otherwise hold the last frame and set `finished`.
   *
   * @default true
   */
  loop: boolean;
  /**
   * How much of the pose this player decides (0 = none, 1 = all).
   *
   * @default 1
   * @range 0..1
   * @tunable
   */
  weight: number;
  /**
   * Cross-fade duration when `clip` changes.
   *
   * @default 0.2
   * @range 0..10
   * @unit s (seconds of simulation time)
   * @tunable
   */
  fade: number;
  /**
   * Stepped playback: each pose is held for 1/stepFps seconds, the clip sampled at its time floored to that step (12 is stop motion "on twos", 8 on threes), the same on every tick rate; 0 samples every tick. `time`, markers and root motion stay continuous; a one-shot still ends on its last frame.
   *
   * @range 0..60
   * @tunable
   */
  stepFps: number;
  /**
   * Written: a non-looping clip reached its end.
   *
   * @readonly written by the engine
   */
  readonly finished: boolean;
}

/**
 * Runs a blend-tree graph (og.anim.graph) on a skeleton: its layers' state machines pick motions by the parameters p0...p7, which the graph names in `param` order (`param speed` is p0, the next `param` is p1, ...).
 *
 * The value of the `AnimationGraph` component (token `og.AnimationGraph`).
 */
export interface AnimationGraph {
  /** The graph asset; 0 is none. */
  asset: GraphAsset;
  /**
   * How much of the pose the graph decides over the rest pose.
   *
   * @default 1
   * @range 0..1
   * @tunable
   */
  weight: number;
  /**
   * A playback-rate multiplier for every layer.
   *
   * @default 1
   * @range 0..10
   * @tunable
   */
  speed: number;
  /**
   * Parameter 0 (the graph's first `param`).
   *
   * @tunable
   */
  p0: number;
  /**
   * Parameter 1.
   *
   * @tunable
   */
  p1: number;
  /**
   * Parameter 2.
   *
   * @tunable
   */
  p2: number;
  /**
   * Parameter 3.
   *
   * @tunable
   */
  p3: number;
  /**
   * Parameter 4.
   *
   * @tunable
   */
  p4: number;
  /**
   * Parameter 5.
   *
   * @tunable
   */
  p5: number;
  /**
   * Parameter 6.
   *
   * @tunable
   */
  p6: number;
  /**
   * Parameter 7.
   *
   * @tunable
   */
  p7: number;
  /**
   * Written: the base layer's current state (its index in the graph file).
   *
   * @readonly written by the engine
   */
  readonly state: number;
  /**
   * Written: seconds since the base layer entered `state`.
   *
   * @unit s (seconds of simulation time)
   * @readonly written by the engine
   */
  readonly stateTime: number;
}

/**
 * Morph-target weights of the skeleton entity's mesh, in the order of the glTF mesh's targets. With `source` clips the playing clips' weight channels write them; with `component` game code does.
 *
 * The value of the `Blendshapes` component (token `og.Blendshapes`).
 */
export interface Blendshapes {
  /**
   * Who decides the weights: component (game code writes w0...w7) or clips (the animation writes them).
   */
  source: BlendshapeSource;
  /**
   * The glTF mesh index whose targets the weights drive (0 is the first mesh).
   *
   * @range 0..
   */
  mesh: number;
  /**
   * Weight of target 0.
   *
   * @range -1..2
   * @tunable
   */
  w0: number;
  /**
   * Weight of target 1.
   *
   * @range -1..2
   * @tunable
   */
  w1: number;
  /**
   * Weight of target 2.
   *
   * @range -1..2
   * @tunable
   */
  w2: number;
  /**
   * Weight of target 3.
   *
   * @range -1..2
   * @tunable
   */
  w3: number;
  /**
   * Weight of target 4.
   *
   * @range -1..2
   * @tunable
   */
  w4: number;
  /**
   * Weight of target 5.
   *
   * @range -1..2
   * @tunable
   */
  w5: number;
  /**
   * Weight of target 6.
   *
   * @range -1..2
   * @tunable
   */
  w6: number;
  /**
   * Weight of target 7.
   *
   * @range -1..2
   * @tunable
   */
  w7: number;
}

/**
 * An inverse-kinematics constraint on a skeleton, applied after its clips and graph: the chain of `bones` bones ending at joint `tip` turns so the tip reaches `target`. Put it on its own entity (a skeleton can have many chains); chains apply in query order.
 *
 * The value of the `IkChain` component (token `og.IkChain`).
 */
export interface IkChain {
  /** The entity with the og.Skeleton this chain bends. */
  skeleton: Entity;
  /**
   * The end joint (an index into the skin's joints): the foot or the hand.
   *
   * @range 0..
   */
  tip: number;
  /**
   * How many bones above the tip turn (two-bone always uses 2).
   *
   * @default 2
   * @range 1..32
   */
  bones: number;
  /** twoBone (analytic, with a pole) or fabrik (iterative, any length). */
  solver: IkSolver;
  /**
   * Where the tip should be, in `space`: for `world`, the frame of the skeleton entity's og.Transform (its parent's frame when it has a parent, not true world).
   *
   * @unit m (metres)
   */
  target: Vector3;
  /**
   * A point the middle joint bends towards (two-bone); the zero vector keeps the animated bend.
   *
   * @unit m (metres)
   */
  pole: Vector3;
  /**
   * What `target` and `pole` are relative to: world (the frame the skeleton entity's og.Transform is in, so its parent's frame for a child entity, not true world) or model.
   */
  space: IkSpace;
  /**
   * How much of the solved rotation is applied (0 = the animated pose, 1 = fully solved).
   *
   * @default 1
   * @range 0..1
   * @tunable
   */
  weight: number;
  /**
   * FABRIK iterations at most.
   *
   * @default 10
   * @range 1..64
   */
  iterations: number;
  /**
   * The tip distance (model units) at which the target counts as reached.
   *
   * @default 0.001
   * @range 0..
   * @unit m (metres)
   */
  tolerance: number;
  /**
   * Written: the tip is within `tolerance` of the target after solving.
   *
   * @readonly written by the engine
   */
  readonly reached: boolean;
  /**
   * Written: the tip's distance from the target after solving, in model units.
   *
   * @unit m (metres)
   * @readonly written by the engine
   */
  readonly error: number;
}

/**
 * A named marker of a clip that playback passed in the last tick (read in the tick after it fired, like every event). Cross-fades and blends fire every contributing clip's markers.
 *
 * The payload of the `AnimationEvent` event (token `og.AnimationEvent`).
 */
export interface AnimationEvent {
  /** The skeleton entity. */
  readonly entity: Entity;
  /** The clip asset the marker belongs to. */
  readonly clip: ClipAsset;
  /** The marker's name (at most 63 bytes). */
  readonly name: string;
  /**
   * The marker's time in the clip.
   *
   * @unit s (seconds of simulation time)
   */
  readonly time: number;
  /** The clip's blend weight when it fired. */
  readonly weight: number;
}

/** One clip of a loaded animation. */
export interface ClipInfo {
  /** Its handle, what AnimationPlayer.clip takes. */
  readonly clip: ClipAsset;
  /** The glTF animation's name. */
  readonly name: string;
  /**
   * Its length.
   *
   * @unit s (seconds of simulation time)
   */
  readonly duration: number;
  /** Its markers' names, in time order. */
  readonly markers: readonly string[];
}

/** What og.load.animation registered from a model's `#anim` blob. */
export interface Animation {
  /** The rig, what Skeleton.asset takes. */
  readonly skeleton: SkeletonAsset;
  /** Its clips, in glTF order. */
  readonly clips: readonly ClipInfo[];
  /** The rig's node names, in topological order. */
  readonly nodes: readonly string[];
  /** Each skin's joint count, in glTF order. */
  readonly skins: readonly number[];
  /** Morph targets across the model's meshes. */
  readonly morphTargets: number;
}

/**
 * A blend-tree graph to register.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GraphDesc {
  /**
   * The graph's `.ogt` text (`type animgraph`, `version 1`; og-game §9 has every line).
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * The clips a `clip <motion> "<name>"` line may name, by ClipInfo.name: usually og.load.animation's `clips`.
   *
   * @default []
   */
  readonly clips?: readonly ClipInfo[];
}


/**
 * A humanoid map from og.load.rig (a cooked `.ogt` of `type rig`): the naming scheme a skeleton's node names follow and the standard bones it names outright.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Rig = number & { readonly [handleType]: "Rig" };

/** One standard humanoid bone of a skeleton. */
export interface HumanBone {
  /**
   * The standard bone: hips, spine, chest, upperChest, neck, head, then leftShoulder, leftUpperArm, leftLowerArm, leftHand, the right arm, leftUpperLeg, leftLowerLeg, leftFoot, leftToes and the right leg.
   */
  readonly bone: string;
  /** The skeleton node that is that bone. */
  readonly node: string;
}

/** Which node of a skeleton is each standard humanoid bone (what retargeting maps). */
export interface Humanoid {
  /**
   * The naming scheme the bones were read in: mixamo, vrm, unity, rigify, ue or blender; custom when a rig named them, none when nothing matched.
   */
  readonly scheme: string;
  /** The bones found, in the standard order (hips first). */
  readonly bones: readonly HumanBone[];
}

/**
 * A skeleton and, optionally, the rig that says how its bones are named.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface HumanoidDesc {
  /**
   * The skeleton (og.load.animation's `skeleton`).
   *
   * @default 0
   */
  readonly skeleton?: SkeletonAsset;
  /**
   * A humanoid map from og.load.rig; 0 detects the bones from the node names (Mixamo, VRM, Unity, Rigify, Unreal or Blender names).
   *
   * @default 0
   */
  readonly rig?: Rig;
}


/**
 * A clip made for one skeleton, to play on another (other proportions, rest pose or bone names).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RetargetDesc {
  /**
   * The clip, made for `source`.
   *
   * @default 0
   */
  readonly clip?: ClipAsset;
  /**
   * The skeleton the clip was made for (og.load.animation's `skeleton`: a character's, or an animation library's).
   *
   * @default 0
   */
  readonly source?: SkeletonAsset;
  /**
   * The skeleton that will play it.
   *
   * @default 0
   */
  readonly target?: SkeletonAsset;
  /**
   * How the source's bones are named (og.load.rig); 0 detects them.
   *
   * @default 0
   */
  readonly sourceRig?: Rig;
  /**
   * How the target's bones are named; 0 detects them.
   *
   * @default 0
   */
  readonly targetRig?: Rig;
  /**
   * Solve the legs so a planted foot stays planted on the target's ground (and the hips lower when a leg cannot reach).
   *
   * @default true
   */
  readonly footIk?: boolean;
}


/**
 * A sprite sheet's frames and clips (a cooked `.ogt` of `type flipbook`), from og.load.flipbook.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type FlipbookSheet = number & { readonly [handleType]: "FlipbookSheet" };

/**
 * A FlipbookSheet handle field.
 *
 * A `FlipbookSheet` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function FlipbookSheet(value?: FlipbookSheet, options?: FieldOptions): FieldSpec<"FlipbookSheet">;

/**
 * How a flipbook clip runs past its last frame.
 * - `"loop"`: Back to the first frame.
 * - `"pingPong"`: Back and forth, without repeating the end frames.
 * - `"once"`: Hold the last frame and set Flipbook.finished.
 */
export type FlipMode = "loop" | "pingPong" | "once";

/**
 * Plays a clip of a flipbook sheet on the entity's og.Sprite: extraction draws the sprite with the sheet's texture and the current frame's region and pivot, or, for an `array true` sheet, the frame's layer of its texture array whole (the sprite keeps its size, colour, flips and layer). Played by the engine's og.flipbook system by scaled time (Time.scaledDt: slow motion, hit stop), after the game's systems.
 * Changing clip restarts at the clip's first frame (or at `time`, written in the same tick); writing a different time is a seek. Frame events are og.FlipbookEvent. The playback position is world state (the og.flipbook state block).
 *
 * The value of the `Flipbook` component (token `og.Flipbook`).
 */
export interface Flipbook {
  /** The sheet (og.load.flipbook's `sheet`); 0 is none. */
  sheet: FlipbookSheet;
  /**
   * The clip: an index into FlipbookInfo.clips (og.anim.clip finds one by name).
   *
   * @range 0..
   */
  clip: number;
  /**
   * The playback rate: 1 as authored, 2 twice as fast, 0 holds the frame.
   *
   * @default 1
   * @range 0..10
   * @tunable
   */
  speed: number;
  /**
   * Whether time advances.
   *
   * @default true
   */
  playing: boolean;
  /**
   * Written: the clip frame shown (an index into the clip's frames).
   *
   * @readonly written by the engine
   */
  readonly frame: number;
  /**
   * Seconds into the current cycle; written every tick, set it to seek (no events fire across a seek).
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  time: number;
  /**
   * Written: a `once` clip reached and holds its last frame.
   *
   * @readonly written by the engine
   */
  readonly finished: boolean;
  /**
   * Written: cycles completed by a looping or ping-pong clip.
   *
   * @readonly written by the engine
   */
  readonly loops: number;
}

/**
 * A flipbook entered a frame its clip names in an `event` line (`event 2 "step"`): footsteps, the frame a hit lands, a pickup's sparkle. Read in the tick after, like every event; a clip started at 0 fires its first frame's events.
 *
 * The payload of the `FlipbookEvent` event (token `og.FlipbookEvent`).
 */
export interface FlipbookEvent {
  /** The entity with the og.Flipbook. */
  readonly entity: Entity;
  /** The clip index. */
  readonly clip: number;
  /** The clip frame entered. */
  readonly frame: number;
  /** The event's name (at most 63 bytes). */
  readonly name: string;
}

/** One clip of a loaded flipbook sheet. */
export interface FlipbookClipInfo {
  /** Its index, what Flipbook.clip takes. */
  readonly clip: number;
  /** Its name in the sheet. */
  readonly name: string;
  /** How many frames. */
  readonly frames: number;
  /**
   * Seconds of one cycle (a ping-pong cycle goes there and back).
   *
   * @unit s (seconds of simulation time)
   */
  readonly duration: number;
  /** How it runs past its end. */
  readonly mode: FlipMode;
  /** Its events' names, in frame order. */
  readonly events: readonly string[];
}

/** What og.load.flipbook registered. */
export interface FlipbookInfo {
  /** What Flipbook.sheet takes. */
  readonly sheet: FlipbookSheet;
  /**
   * The sheet's texture (loaded with it; sampled nearest unless the sheet says `filter linear`); 0 for an `array true` sheet.
   */
  readonly texture: Texture;
  /**
   * An `array true` sheet's texture array (its frames as layers, loaded with it); 0 otherwise.
   */
  readonly array: TextureArray;
  /** Its clips, in sheet order. */
  readonly clips: readonly FlipbookClipInfo[];
  /** Frames in the sheet. */
  readonly frames: number;
  /** The first frame's width in pixels. */
  readonly frameWidth: number;
  /** The first frame's height in pixels. */
  readonly frameHeight: number;
}




/**
 * A 2D rig (a cooked `.ogt` of `type rig2d`): bones, slots, region, mesh, bounding-box and child-armature attachments, IK constraints, child armatures and clips, from og.load.rig2d.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Rig2D = number & { readonly [handleType]: "Rig2D" };

/**
 * A Rig2D handle field.
 *
 * A `Rig2D` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Rig2D(value?: Rig2D, options?: FieldOptions): FieldSpec<"Rig2D">;

/**
 * Plays a clip of a 2D rig at the entity: the og.skeleton2d system advances it by scaled time (Time.scaledDt) after the game's systems and fires its events as og.Skeleton2DEvent; extraction draws its slots, in their order, as textured triangles of the sprite batch at `layer` (region attachments as quads, mesh attachments weighted to bones and deformed by the clip), lit by og.Lighting2D when the layer is in its range, as a sprite there is. The rig's pixels are `unit` to a world unit and +y is up; the entity's og.Transform places, turns (about z) and scales it.
 * Changing `clip` restarts at 0 (or at `time`, written in the same tick); writing a different time is a seek and fires no events. The pose is a function of these fields, the Transform and the og.IkTarget2D on it; the playback position is world state (the og.skeleton2d state block).
 *
 * The value of the `Skeleton2D` component (token `og.Skeleton2D`).
 */
export interface Skeleton2D {
  /** The rig (og.load.rig2d's `rig`); 0 is none. */
  rig: Rig2D;
  /**
   * The clip: an index into Rig2DInfo.clips (og.anim.clip finds one by name).
   *
   * @range 0..
   */
  clip: number;
  /**
   * Seconds into the clip; written every tick, set it to seek.
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  time: number;
  /**
   * The playback rate: 1 as authored, 2 twice as fast, negative backwards.
   *
   * @default 1
   * @range -10..10
   * @tunable
   */
  speed: number;
  /**
   * Wrap at the clip's end; otherwise hold the last pose and set `finished`. Rig2DInfo.clips says what the rig's author meant.
   *
   * @default true
   */
  loop: boolean;
  /** Mirror the rig left to right about the entity's position (a character facing left). */
  flipX: boolean;
  /**
   * The sprite layer it draws in, sorted with the sprites there (after them when they share the layer).
   */
  layer: number;
  /**
   * Written: a clip that does not loop reached its end (or its start, played backwards).
   *
   * @readonly written by the engine
   */
  readonly finished: boolean;
}

/**
 * A 2D skeleton's clip passed one of its events (`event 0.2 "step"` in the rig) in the last tick: footsteps, the frame a swing lands. Read in the tick after, like every event; a clip started at 0 fires its events at 0, a looping clip fires each once per pass.
 *
 * The payload of the `Skeleton2DEvent` event (token `og.Skeleton2DEvent`).
 */
export interface Skeleton2DEvent {
  /** The entity with the og.Skeleton2D. */
  readonly entity: Entity;
  /** The clip index. */
  readonly clip: number;
  /** The event's name (at most 63 bytes). */
  readonly name: string;
  /**
   * The event's time in the clip.
   *
   * @unit s (seconds of simulation time)
   */
  readonly time: number;
}

/**
 * Moves one IK constraint of a 2D skeleton (Rig2DInfo.constraints) to a point: a foot planted on the ground under it, a hand on a ledge. Put it on its own entity, one per constraint; og.skeleton2d writes `reached` and `error`, and the drawn pose solves the same way.
 *
 * The value of the `IkTarget2D` component (token `og.IkTarget2D`).
 */
export interface IkTarget2D {
  /** The entity with the og.Skeleton2D. */
  skeleton: Entity;
  /**
   * The constraint: an index into Rig2DInfo.constraints.
   *
   * @range 0..
   */
  constraint: number;
  /**
   * Where the chain's tip should be, in the frame the skeleton entity's og.Transform is written in (the world for an entity without a parent).
   *
   * @unit m (metres)
   */
  target: Vector2;
  /**
   * How much of the solve applies (0 is the animated pose, 1 fully solved), times the constraint's own weight (its `weight` line, or its clip's IK keys).
   *
   * @default 1
   * @range 0..1
   * @tunable
   */
  weight: number;
  /**
   * The distance at which the target counts as reached.
   *
   * @default 0.01
   * @range 0..
   * @unit m (metres)
   */
  tolerance: number;
  /**
   * Written: the tip is within `tolerance` of the target after solving.
   *
   * @readonly written by the engine
   */
  readonly reached: boolean;
  /**
   * Written: the tip's distance from the target after solving (for a one-bone constraint, how far the target lies off the bone's line).
   *
   * @unit m (metres)
   * @readonly written by the engine
   */
  readonly error: number;
}

/** One clip of a loaded 2D rig. */
export interface Rig2DClipInfo {
  /** Its index, what Skeleton2D.clip takes. */
  readonly clip: number;
  /** Its name in the rig. */
  readonly name: string;
  /**
   * Its length.
   *
   * @unit s (seconds of simulation time)
   */
  readonly duration: number;
  /**
   * Whether the rig's author meant it to loop (its `loop` line): copy it into Skeleton2D.loop.
   */
  readonly loop: boolean;
  /** Its events' names, in time order. */
  readonly events: readonly string[];
}

/** What og.load.rig2d registered, or og.anim.rig2dInfo reads back from a handle. */
export interface Rig2DInfo {
  /** What Skeleton2D.rig takes. */
  readonly rig: Rig2D;
  /** The rig's texture (loaded with it). */
  readonly texture: Texture;
  /** Pixels of the rig per world unit. */
  readonly unit: number;
  /** The bones' names, parents first. */
  readonly bones: readonly string[];
  /** The slots' names, in draw order. */
  readonly slots: readonly string[];
  /** The IK constraints' names: an index here is IkTarget2D.constraint. */
  readonly constraints: readonly string[];
  /** Its clips, in file order. */
  readonly clips: readonly Rig2DClipInfo[];
}


/**
 * A bounding box a 2D skeleton's pose shows: a `box` attachment of its rig (or of a child armature in one of its slots) while its slot shows it.
 */
export interface Skeleton2DBox {
  /** The slot showing it (`slot/child-slot` inside a child armature). */
  readonly slot: string;
  /** The box's name in the rig. */
  readonly name: string;
  /**
   * Its polygon, moved by its bones (IK included), in the frame the skeleton entity's og.Transform is written in (the world for an entity without a parent).
   */
  readonly points: readonly Vector2[];
}


// ── tween ───────────────────────────────────────────────────────
// Tweens: animate any numeric field of a component (og.Transform.position, og.Sprite.color, og.Sprite.opacity, a game's own f32) from its value to a target over time with an easing curve, a delay, loops and yoyo; chain them in sequences or run them in parallel. Advanced by the engine's og.tween system after the game's systems and before og.anim and physics, by scaled time (slow motion, hit stop) unless unscaled.
// Running tweens are world state (the og.tween state block): snapshots, restore and the world hash cover them. og.TweenCompleted reports a tween or group that ends on its own; a cancelled one, or one whose entity or component went away, ends silently.
// Feel presets are tweens too: og.tween.squash (squash and stretch that recovers) and og.tween.flash (a hit flash that fades).

/**
 * A running tween or group.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Tween = number & { readonly [handleType]: "Tween" };

/**
 * A Tween handle field.
 *
 * A `Tween` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Tween(value?: Tween, options?: FieldOptions): FieldSpec<"Tween">;

/**
 * An easing curve from the start (0) to the end (1). In starts slow, out ends slow, inOut both. back overshoots, elastic rings, bounce bounces at the end.
 * - `"quadOut"`: Gentle ease out: the default for most motion.
 * - `"linear"`: Constant speed: scrolling, timers, colour cycles.
 * - `"quadIn"`: Gentle ease in.
 * - `"quadInOut"`: Gentle in and out.
 * - `"cubicIn"`: Ease in.
 * - `"cubicOut"`: Ease out: UI sliding in, things settling.
 * - `"cubicInOut"`: In and out: camera moves, deliberate motion.
 * - `"quartIn"`: Strong ease in.
 * - `"quartOut"`: Strong ease out.
 * - `"quartInOut"`: Strong in and out.
 * - `"expoIn"`: Exponential ease in: things falling away, warp-outs.
 * - `"expoOut"`: Exponential ease out: snappy arrivals.
 * - `"expoInOut"`: Exponential in and out.
 * - `"backIn"`: Pulls back first, then goes: anticipation.
 * - `"backOut"`: Overshoots, then settles: pops, squash recovery, buttons.
 * - `"backInOut"`: Pulls back and overshoots.
 * - `"elasticIn"`: Rings before leaving.
 * - `"elasticOut"`: Rings after arriving: springy, cartoon impacts.
 * - `"elasticInOut"`: Rings at both ends.
 * - `"bounceIn"`: Bounces before leaving.
 * - `"bounceOut"`: Bounces at the end: drops, landings.
 * - `"bounceInOut"`: Bounces at both ends.
 */
export type Ease = "quadOut" | "linear" | "quadIn" | "quadInOut" | "cubicIn" | "cubicOut" | "cubicInOut" | "quartIn" | "quartOut" | "quartInOut" | "expoIn" | "expoOut" | "expoInOut" | "backIn" | "backOut" | "backInOut" | "elasticIn" | "elasticOut" | "elasticInOut" | "bounceIn" | "bounceOut" | "bounceInOut";

/**
 * One tween of one field.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TweenDesc {
  /**
   * The entity whose component is tweened.
   *
   * @default 0
   */
  readonly entity?: Entity;
  /**
   * The component token (og.Transform, og.Sprite, a game's own); null is og.Transform.
   *
   * @default null
   */
  readonly component?: AnyComponent | null;
  /**
   * The field, or one lane of it: `position`, `position.y`, `scale`, `color.r`, `opacity`. f32, f64, i32 and u32 fields (rounded), vectors, colours and quaternions (as a rotation) tween; bool, enum and handle fields do not.
   *
   * @default ""
   */
  readonly field?: string;
  /**
   * The end value: one number for a scalar or for every lane, or one per lane.
   *
   * @default []
   */
  readonly to?: readonly number[];
  /**
   * The start value, written from the tick the tween is started and held through its delay; null takes the field's value when the tween starts (after its delay).
   *
   * @default null
   */
  readonly from?: readonly number[] | null;
  /**
   * `to` is added to the start value.
   *
   * @default false
   */
  readonly relative?: boolean;
  /**
   * Seconds from start to end (one cycle).
   *
   * @default 0.3
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  readonly duration?: number;
  /**
   * Seconds before it starts.
   *
   * @default 0
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  readonly delay?: number;
  /**
   * The curve.
   *
   * @default "quadOut"
   */
  readonly ease?: Ease;
  /**
   * Extra cycles: 0 plays once, n repeats n more times, -1 forever.
   *
   * @default 0
   * @range -1..
   */
  readonly loops?: number;
  /**
   * Every other cycle plays backwards (pulses, bobbing).
   *
   * @default false
   */
  readonly yoyo?: boolean;
  /**
   * Advance by real ticks, ignoring Time.scale and hit stop (UI motion).
   *
   * @default false
   */
  readonly unscaled?: boolean;
  /**
   * A number of yours, carried by og.TweenCompleted.
   *
   * @default 0
   */
  readonly tag?: number;
}

/**
 * How a sequence or parallel group runs.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TweenGroupDesc {
  /**
   * Extra runs of the whole group: 0 once, -1 forever; each run replays the spans its tweens resolved in the first.
   *
   * @default 0
   * @range -1..
   */
  readonly loops?: number;
  /**
   * Advance by real ticks, ignoring Time.scale and hit stop.
   *
   * @default false
   */
  readonly unscaled?: boolean;
  /**
   * A number of yours, carried by og.TweenCompleted.
   *
   * @default 0
   */
  readonly tag?: number;
}

/**
 * A tween or group ended on its own (read in the tick after).
 *
 * The payload of the `TweenCompleted` event (token `og.TweenCompleted`).
 */
export interface TweenCompleted {
  /** The handle og.tween.to, sequence, parallel, squash or flash answered. */
  readonly tween: Tween;
  /** The entity of its (first) tween. */
  readonly entity: Entity;
  /** Its tag. */
  readonly tag: number;
}

/**
 * Squash and stretch: the entity's og.Transform.scale jumps to rest times impact and eases back to rest.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SquashDesc {
  /**
   * The entity.
   *
   * @default 0
   */
  readonly entity?: Entity;
  /**
   * The x, y and z factors at the impact: (1.25, 0.8) squashes (a landing, a hit), (0.8, 1.25) stretches (a jump). z is for 3D models (a squash bulges in x and z: (1.15, 0.85, 1.15)); left out or 0 it stays at rest, so a sprite's { x, y } is unchanged. A component left out takes its default: x 1.25, y 0.8, z 0.
   *
   * @default {}
   */
  readonly impact?: Partial<Vector3>;
  /**
   * The scale it recovers to.
   *
   * @default { x: 1, y: 1, z: 1 }
   */
  readonly rest?: Vector3;
  /**
   * Seconds to recover.
   *
   * @default 0.18
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  readonly duration?: number;
  /**
   * The recovery curve; backOut overshoots a little, elasticOut wobbles.
   *
   * @default "backOut"
   */
  readonly ease?: Ease;
}

/**
 * A hit flash: the entity's og.Sprite (else og.Mesh, else og.Model) flashColor set and its flash faded from amount to 0.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface FlashDesc {
  /**
   * The entity.
   *
   * @default 0
   */
  readonly entity?: Entity;
  /**
   * The flash colour (white for a hit, red for damage taken).
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  readonly color?: Color;
  /**
   * How far it starts towards the colour.
   *
   * @default 1
   * @range 0..1
   */
  readonly amount?: number;
  /**
   * Seconds to fade.
   *
   * @default 0.12
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  readonly duration?: number;
  /**
   * The fade's curve; quadIn holds the flash, then drops it.
   *
   * @default "quadIn"
   */
  readonly ease?: Ease;
}









// ── tilemap ─────────────────────────────────────────────────────
// Tilemaps: layers of tiles from a tileset atlas (an `.ogt` of `type tileset`: grid, solid tiles and collision rects, animated tiles, 47-blob and 16-bitmask auto-tiling terrains), loaded from a cooked map (a hand-written `.ogt` of `type tilemap`, a Tiled `.tmj` or an LDtk `.ldtk`) with og.load.tilemap and placed by an og.Tilemap component.
// Layers draw through the sprite batch in 16x16-tile chunks culled to the view, each at the map's sprite layer plus its z, scrolled by its parallax; solid tiles become static og-physics colliders merged into rectangles. Runtime edits (og.tilemap.setTile, re-auto-tiling the neighbourhood) are world state (the og.tilemap state block), and the colliders follow them.
// With og.Lighting2D enabled, a layer whose sprite layer (the map's layer plus its z) lies in minLayer to maxLayer is lit as the sprites there are: multiplied by the ambient plus every Light2D reaching it, with the shadows of Occluder2Ds and castShadow sprites, so a dark cave lit by torches is the map, its lights and its occluders. A layer that should stay as authored in the lit range (a sky, a far parallax backdrop) opts out: a bool custom property `lit` set to false on the Tiled layer, or a `lit false` line in a tilemap `.ogt`'s layer block (TileLayerInfo.lit reports it).
// Cells count from the top-left corner, x right and y down (Tiled's); the map's top-left corner is at the entity's position.

/**
 * A loaded map: its layers, tilesets and objects.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type TilemapAsset = number & { readonly [handleType]: "TilemapAsset" };

/**
 * A TilemapAsset handle field.
 *
 * A `TilemapAsset` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function TilemapAsset(value?: TilemapAsset, options?: FieldOptions): FieldSpec<"TilemapAsset">;

/**
 * Draws a loaded map with its top-left corner at the entity's position (z included) and gives its collision layers static colliders. The og.tilemap system adopts it in the first tick it exists: from then on og.tilemap.setTile and getTile reach it.
 *
 * The value of the `Tilemap` component (token `og.Tilemap`).
 */
export interface Tilemap {
  /** The map (og.load.tilemap's `map`); 0 is none. */
  map: TilemapAsset;
  /**
   * The sprite layer of the map's layer 0; every layer draws at this plus its z (default: its index).
   *
   * @default -10
   */
  layer: number;
  /**
   * A linear tint.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  color: Color;
  /**
   * The alpha, times each layer's.
   *
   * @default 1
   * @range 0..1
   */
  opacity: number;
  /**
   * Whether it is drawn.
   *
   * @default true
   */
  visible: boolean;
  /**
   * Give the layers marked collision static colliders (rectangles merged from the solid tiles).
   *
   * @default true
   */
  collision: boolean;
  /**
   * The collision layers of those colliders, as bits (Collider.layers), for every collision layer of the map that names none of its own (a map layer's `colliderLayers`, with its `colliderMask`, puts a layer of grates on a one-way layer while the walls stay here).
   *
   * @default 1
   */
  colliderLayers: number;
  /**
   * The colliders' friction.
   *
   * @default 0.5
   * @range 0..2
   */
  friction: number;
}

/** One layer of a map. */
export interface TileLayerInfo {
  /** Its name. */
  readonly name: string;
  /** Its index (what drawing order and z start from). */
  readonly index: number;
  /** Its sprite-layer offset from Tilemap.layer. */
  readonly z: number;
  /**
   * Its scroll factors: 1 1 moves with the world, 0.5 at half the camera's speed (far away), 0 stays with the camera.
   */
  readonly parallax: Vector2;
  /** Its solid tiles collide. */
  readonly collision: boolean;
  /**
   * The collision layers (og.Collider.layers bits) of its colliders, from its `colliderLayers` line or Tiled int property; 0 (unset) takes Tilemap.colliderLayers.
   */
  readonly colliderLayers: number;
  /**
   * The layers its colliders collide with (og.Collider.mask bits), from its `colliderMask` line or Tiled int property; 0 (unset) is every layer.
   */
  readonly colliderMask: number;
  /**
   * The auto-tiling terrain of a terrain layer (its cells are 1 filled or 0 empty); empty for a tile layer.
   */
  readonly terrain: string;
  /** Whether it is drawn. */
  readonly visible: boolean;
  /**
   * Whether og.Lighting2D lights it when its sprite layer is in the lit range (true unless the Tiled layer's bool property `lit` or the `.ogt` layer's `lit` line says false).
   */
  readonly lit: boolean;
}

/**
 * An object of a map (a Tiled object, an LDtk entity, an `object` block): spawn points, doors, triggers.
 */
export interface TileObject {
  /** Its name. */
  readonly name: string;
  /** Its type or class. */
  readonly kind: string;
  /**
   * Its top-left corner, in world units relative to the map's top-left corner (y up, so below the corner is negative).
   *
   * @unit m (metres)
   */
  readonly position: Vector2;
  /**
   * Its width and height (0 0 for a point).
   *
   * @unit m (metres)
   */
  readonly size: Vector2;
  /**
   * Its centre, relative to the map's corner.
   *
   * @unit m (metres)
   */
  readonly center: Vector2;
  /** Its custom properties, as text. */
  readonly properties: Readonly<Record<string, string>>;
}

/** What og.load.tilemap loaded. */
export interface TilemapInfo {
  /** What Tilemap.map takes. */
  readonly map: TilemapAsset;
  /** Cells across. */
  readonly width: number;
  /** Cells down. */
  readonly height: number;
  /**
   * A cell's width in world units.
   *
   * @unit m (metres)
   */
  readonly tileWidth: number;
  /**
   * A cell's height in world units.
   *
   * @unit m (metres)
   */
  readonly tileHeight: number;
  /** Its layers, drawing order. */
  readonly layers: readonly TileLayerInfo[];
  /** Its objects. */
  readonly objects: readonly TileObject[];
}

/**
 * One cell to change.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SetTileDesc {
  /**
   * The entity with the og.Tilemap.
   *
   * @default 0
   */
  readonly entity?: Entity;
  /**
   * The layer's name.
   *
   * @default ""
   */
  readonly layer?: string;
  /**
   * The column (0 is the left).
   *
   * @default 0
   */
  readonly x?: number;
  /**
   * The row (0 is the top).
   *
   * @default 0
   */
  readonly y?: number;
  /**
   * A tile layer: the tile id, -1 empty. A terrain layer: 1 fills, 0 clears, and the neighbours re-tile.
   *
   * @default 0
   * @range -1..
   */
  readonly tile?: number;
}

/** A cell of a map. */
export interface TileCell {
  /** The column. */
  readonly x: number;
  /** The row. */
  readonly y: number;
  /** Whether the point was on the map. */
  readonly inside: boolean;
}





// ── audio ───────────────────────────────────────────────────────
// Sound: inside the tick, the engine's og.audio system (after the game's systems and physics) turns AudioSource changes into a deterministic, time-stamped command stream; outside the tick the host's mixer renders it (an AudioWorklet on the web, the output device, a capture or a null sink natively).
// The stream, not the PCM, is world state: snapshots, restore and the world hash cover it, and nothing a sink does reaches the world. Load sounds with og.load.sound; inserting a source plays it, setting `playing` false stops it, and a one-shot sets `playing` back to false when it ends.
// Spatial gain and pan are computed in the tick from the source's world transform relative to the first AudioListener's (three.js convention: the listener looks down -Z, +X is right; with none, the origin looking down -Z). A source, listener or zone on a child entity is placed at its og.Transform composed down og.Parent, as og.MatrixWorld is: a sound on a child is heard where its parent carries it, a listener under a camera rig hears from the rig.

/** SoundEnded.reason: a one-shot played to its end. */
export declare const SOUND_FINISHED: 0;

/** SoundEnded.reason: `playing` was set to false. */
export declare const SOUND_STOPPED: 1;

/** SoundEnded.reason: the AudioSource was removed from a living entity. */
export declare const SOUND_REMOVED: 2;

/** SoundEnded.reason: `sound` changed while it played. */
export declare const SOUND_REPLACED: 3;

/**
 * SoundEnded.reason: more sources played than Audio.maxVoices and this one stood lowest (priority, then volume, then age).
 */
export declare const SOUND_CULLED: 4;

/**
 * The bus a source plays on; every bus passes through master.
 * - `"master"`: The master bus: what every bus passes through.
 * - `"music"`: Music.
 * - `"sfx"`: Sound effects.
 * - `"ui"`: Interface sounds.
 */
export type AudioBus = "master" | "music" | "sfx" | "ui";

/**
 * An AudioBus field: the variant's index.
 *
 * A `AudioBus` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function AudioBus(value?: AudioBus, options?: FieldOptions): FieldSpec<"AudioBus">;

/**
 * A sound an entity plays. Level-triggered: while `playing` is true it plays (inserting a source plays it), setting it false stops it, and a one-shot that reaches its end sets it back to false and sends og.SoundEnded.
 * A one-shot of F frames at rate R and pitch p started in tick S ends in tick S + ceil(F * tickRate / (R * p)) - 1. A despawned entity's voice is silenced without an event.
 *
 * The value of the `AudioSource` component (token `og.AudioSource`).
 */
export interface AudioSource {
  /**
   * The sound, from og.load.sound; 0 plays nothing. A handle not yet loaded waits and starts on the first tick it is there.
   */
  sound: Sound;
  /**
   * The bus it plays on.
   *
   * @default "sfx"
   */
  bus: AudioBus;
  /**
   * Linear gain.
   *
   * @default 1
   * @range 0..4
   * @tunable
   */
  volume: number;
  /**
   * Playback rate: 2 is an octave up and half as long.
   *
   * @default 1
   * @range 0.0625..16
   * @tunable
   */
  pitch: number;
  /**
   * Stereo position of a non-spatial source, -1 left to 1 right.
   *
   * @range -1..1
   * @tunable
   */
  pan: number;
  /** Loop between loopStart and loopEnd, seamlessly. */
  loop: boolean;
  /**
   * Where a loop restarts.
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  loopStart: number;
  /**
   * Where a loop wraps; 0 is the sound's end.
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  loopEnd: number;
  /**
   * Attenuate and pan by the entity's world transform (its og.Transform down og.Parent) relative to the listener.
   */
  spatial: boolean;
  /**
   * Full volume up to this distance, as Web Audio's PannerNode.refDistance.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  refDistance: number;
  /**
   * No quieter past this distance, as Web Audio's PannerNode.maxDistance.
   *
   * @default 100
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  maxDistance: number;
  /**
   * How fast volume falls with distance, in Web Audio's inverse model: ref / (ref + rolloff * (clamp(d, ref, max) - ref)).
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  rolloffFactor: number;
  /**
   * Plays while true; the engine sets it false when a one-shot ends.
   *
   * @default true
   */
  playing: boolean;
  /**
   * When more than Audio.maxVoices would play, the lowest priority is stopped first (then the quietest, then the oldest), with SoundEnded reason SOUND_CULLED.
   *
   * @default 128
   * @range 0..255
   */
  priority: number;
  /**
   * A low-pass filter on this source, its cutoff; 0 is off.
   *
   * @range 0..20000
   * @unit hz (ticks or cycles per second)
   * @tunable
   */
  lowpass: number;
  /**
   * A spatial source behind a collider (a physics ray to the listener) plays at Audio.occlusionGain through a low-pass at Audio.occlusionCutoff.
   */
  occlude: boolean;
  /**
   * A spatial source's low-pass by distance: the cutoff at maxDistance, from 20 kHz at refDistance (log-linear between); 0 is off.
   *
   * @range 0..20000
   * @unit hz (ticks or cycles per second)
   * @tunable
   */
  distanceCutoff: number;
  /**
   * Starts at this world time (seconds since tick 0), to the sample, even inside a tick; 0 or a time past starts at once. Music.nextBar and Music.nextBeat give musical times.
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  startAt: number;
  /**
   * Where in the sound it starts, in seconds of the sound: started at startAt (or at once) from this far in, to the sample and at full gain, so a resumed phrase continues instead of restarting. Past a one-shot's end it plays nothing; a loop wraps into its region.
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  startOffset: number;
  /**
   * Starts, stops and volume changes ramp over this long: a crossfade is one source stopped and another started with the same fade; 0 is 5 ms.
   *
   * @range 0..30
   * @unit s (seconds of simulation time)
   * @tunable
   */
  fade: number;
}

/**
 * Where spatial sources are heard from: the entity's world transform (its og.Transform down og.Parent, so a listener under a camera rig hears from the rig), looking down its -Z with +X to the right. The first listener in query order is used.
 *
 * The value of the `AudioListener` component (token `og.AudioListener`).
 */
export interface AudioListener {
  /**
   * A gain on every spatial source this listener hears.
   *
   * @default 1
   * @range 0..4
   * @tunable
   */
  volume: number;
}

/**
 * Bus volumes; master scales every bus. A change ramps over 256 frames in the mixer.
 *
 * The `Audio` resource (token `og.Audio`).
 */
export interface Audio {
  /**
   * Gain of the master bus, applied to every sound.
   *
   * @default 1
   * @range 0..2
   * @tunable
   */
  master: number;
  /**
   * Gain of the music bus.
   *
   * @default 1
   * @range 0..2
   * @tunable
   */
  music: number;
  /**
   * Gain of the sfx bus.
   *
   * @default 1
   * @range 0..2
   * @tunable
   */
  sfx: number;
  /**
   * Gain of the ui bus.
   *
   * @default 1
   * @range 0..2
   * @tunable
   */
  ui: number;
  /**
   * The most sources that play at once; past it the lowest AudioSource.priority is stopped (SoundEnded reason SOUND_CULLED).
   *
   * @default 32
   * @range 1..256
   */
  maxVoices: number;
  /**
   * The gain of an occluded source (AudioSource.occlude).
   *
   * @default 0.35
   * @range 0..1
   * @tunable
   */
  occlusionGain: number;
  /**
   * The low-pass cutoff of an occluded source.
   *
   * @default 900
   * @range 20..20000
   * @unit hz (ticks or cycles per second)
   * @tunable
   */
  occlusionCutoff: number;
  /**
   * The shared reverb's room size (the AudioBusFx.reverb sends feed it).
   *
   * @default 0.6
   * @range 0..1
   * @tunable
   */
  reverbRoom: number;
  /**
   * The shared reverb's damping of high frequencies.
   *
   * @default 0.4
   * @range 0..1
   * @tunable
   */
  reverbDamping: number;
  /**
   * Seconds from a sample's world time to the speaker, as the host reports it (og.audio.outputLatency).
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   * @readonly written by the engine
   */
  readonly outputLatency: number;
}

/**
 * A bus's insert effects: a high- and low-pass (12 dB/octave), a compressor, side-chain ducking by another bus, and a send to the shared reverb, whose return joins master. One entity per bus (the first in query order counts); without one a bus passes straight through.
 * Changes reach the mixer through the command stream, so they are deterministic and in the world hash; the PCM is the mixer's.
 *
 * The value of the `AudioBusFx` component (token `og.AudioBusFx`).
 */
export interface AudioBusFx {
  /**
   * The bus these effects are on.
   *
   * @default "music"
   */
  bus: AudioBus;
  /**
   * Low-pass cutoff; 0 is off.
   *
   * @range 0..20000
   * @unit hz (ticks or cycles per second)
   * @tunable
   */
  lowpass: number;
  /**
   * High-pass cutoff; 0 is off.
   *
   * @range 0..20000
   * @unit hz (ticks or cycles per second)
   * @tunable
   */
  highpass: number;
  /**
   * How much of the bus goes to the reverb.
   *
   * @range 0..1
   * @tunable
   */
  reverb: number;
  /**
   * The compressor's threshold; 0 dB is off.
   *
   * @range -60..0
   * @unit db (decibels)
   * @tunable
   */
  threshold: number;
  /**
   * The compressor's ratio: dB in over the threshold per dB out.
   *
   * @default 4
   * @range 1..30
   * @tunable
   */
  ratio: number;
  /**
   * The compressor's attack.
   *
   * @default 0.005
   * @range 0..1
   * @unit s (seconds of simulation time)
   * @tunable
   */
  attack: number;
  /**
   * The compressor's release.
   *
   * @default 0.15
   * @range 0..3
   * @unit s (seconds of simulation time)
   * @tunable
   */
  release: number;
  /**
   * Gain after the compressor.
   *
   * @range -24..24
   * @unit db (decibels)
   * @tunable
   */
  makeup: number;
  /** Turns this bus down while duckBus is loud: music under effects or dialogue. */
  duck: boolean;
  /**
   * The bus that ducks this one.
   *
   * @default "sfx"
   */
  duckBus: AudioBus;
  /**
   * How far it is turned down.
   *
   * @default 9
   * @range 0..60
   * @unit db (decibels)
   * @tunable
   */
  duckDepth: number;
  /**
   * How loud the key bus must be.
   *
   * @default -36
   * @range -80..0
   * @unit db (decibels)
   * @tunable
   */
  duckThreshold: number;
  /**
   * How fast it goes down.
   *
   * @default 0.02
   * @range 0..2
   * @unit s (seconds of simulation time)
   * @tunable
   */
  duckAttack: number;
  /**
   * How fast it comes back.
   *
   * @default 0.4
   * @range 0..5
   * @unit s (seconds of simulation time)
   * @tunable
   */
  duckRelease: number;
}

/**
 * A reverb zone: a box (or a sphere) where the shared reverb sounds like this room. While the og.AudioListener is inside, og.audio eases the reverb's room size and damping, and one bus's send to it, towards the zone's; past the volume's surface it fades out linearly over `fade` metres, so walking in through a door the room grows around the listener.
 * Deterministic: each tick the listener's weight in every zone is a function of its position (written to `weight`), the strongest zone wins (ties to the first in query order), and the eased values reach the mixer as fx commands in the stream and the world hash. Audio.reverbRoom, Audio.reverbDamping and the bus's AudioBusFx.reverb stay the open air's: the zone never writes them. The listener and the zone are placed at their world transforms (og.Transform down og.Parent): a zone on a child of a moving room moves with it.
 *
 * The value of the `ReverbZone` component (token `og.ReverbZone`).
 */
export interface ReverbZone {
  /**
   * The box's full size along the entity's axes, centred on it (times og.Transform.scale).
   *
   * @default { x: 10, y: 4, z: 10 }
   * @unit m (metres)
   */
  size: Vector3;
  /**
   * A sphere of this radius about the entity instead of the box (times the largest of og.Transform.scale); 0 uses the box.
   *
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  radius: number;
  /**
   * How far past the volume's surface the zone fades out, linearly; 0 is a hard edge.
   *
   * @default 2
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  fade: number;
  /**
   * The bus whose reverb send the zone sets.
   *
   * @default "sfx"
   */
  bus: AudioBus;
  /**
   * That bus's send to the reverb inside (its AudioBusFx.reverb outside).
   *
   * @default 0.5
   * @range 0..1
   * @tunable
   */
  send: number;
  /**
   * The reverb's room size inside (Audio.reverbRoom outside).
   *
   * @default 0.85
   * @range 0..1
   * @tunable
   */
  room: number;
  /**
   * The reverb's damping of high frequencies inside (Audio.reverbDamping outside).
   *
   * @default 0.3
   * @range 0..1
   * @tunable
   */
  damping: number;
  /**
   * How much of this zone the listener hears, 0 to 1: 1 inside, 0 past the fade (written by og.audio each tick).
   *
   * @range 0..1
   * @readonly written by the engine
   */
  readonly weight: number;
}

/**
 * A musical clock: tempo, metre and where bar 1 starts. The og.music system (before og.audio) writes the bar and beat and the times of the next beat and bar line every tick, and sends MusicBeat for every beat; an AudioSource whose startAt is nextBar starts on the bar line to the sample, so stems started together stay in step and a stinger lands on the downbeat.
 *
 * The value of the `Music` component (token `og.Music`).
 */
export interface Music {
  /**
   * Beats per minute.
   *
   * @default 120
   * @range 20..400
   * @unit bpm (beats per minute)
   * @tunable
   */
  bpm: number;
  /**
   * Beats per bar.
   *
   * @default 4
   * @range 1..16
   */
  beatsPerBar: number;
  /**
   * The world time of bar 1's first beat.
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  start: number;
  /**
   * The bar the next tick starts in, from 1 (0 before start; written by og.music).
   *
   * @readonly written by the engine
   */
  readonly bar: number;
  /**
   * The beat in that bar, from 1 (written by og.music).
   *
   * @readonly written by the engine
   */
  readonly beat: number;
  /**
   * The world time of the next beat from the next tick on (written by og.music).
   *
   * @unit s (seconds of simulation time)
   * @readonly written by the engine
   */
  readonly nextBeat: number;
  /**
   * The world time of the next bar line from the next tick on (written by og.music).
   *
   * @unit s (seconds of simulation time)
   * @readonly written by the engine
   */
  readonly nextBar: number;
}

/**
 * A beat of a Music clock, sent in the tick whose interval holds it (systems read it the tick after).
 *
 * The payload of the `MusicBeat` event (token `og.MusicBeat`).
 */
export interface MusicBeat {
  /** The entity with the Music. */
  readonly entity: Entity;
  /** The bar, from 1. */
  readonly bar: number;
  /** The beat in the bar, from 1 (1 is the bar line). */
  readonly beat: number;
  /** Its world time. */
  readonly time: number;
}

/**
 * A source started, sent in the tick it happens (systems read it the tick after).
 *
 * The payload of the `SoundStarted` event (token `og.SoundStarted`).
 */
export interface SoundStarted {
  /** The entity whose AudioSource started. */
  readonly entity: Entity;
  /** The sound. */
  readonly sound: Sound;
  /** The voice id in the command stream. */
  readonly voice: number;
}

/**
 * A sound ended, sent in the tick it happens (systems read it the tick after).
 *
 * The payload of the `SoundEnded` event (token `og.SoundEnded`).
 */
export interface SoundEnded {
  /** The entity whose AudioSource ended. */
  readonly entity: Entity;
  /** The sound. */
  readonly sound: Sound;
  /** The voice id in the command stream. */
  readonly voice: number;
  /**
   * Why: SOUND_FINISHED (0), SOUND_STOPPED (1), SOUND_REMOVED (2), SOUND_REPLACED (3) or SOUND_CULLED (4).
   */
  readonly reason: number;
}


// ── nav ─────────────────────────────────────────────────────────
// Navigation: grids (A* and jump point search with eight-way moves that never cut a blocked corner, line-of-sight smoothing), flow fields for crowds, and navmeshes (baked at cook time from a level's static colliders and models, or built at runtime for a flat procedural level) with funnel paths and box obstacles that rebuild only the tiles they touch.
// Agents are entities with an og.NavAgent (and an og.Transform): the og.nav system, after the game's systems and before physics, plans their paths, follows them and adds the og.Steering behaviours; an entity with an og.Velocity (a physics body) gets its velocity written, any other its Transform moved. Everything is deterministic and in snapshots: paths, grid edits, obstacles and flow targets.
// Agents move on the profile's plane: x and y in 2D (a grid's rows run up), x and z in 3D (y kept on grids, from the surface on a navmesh).

/**
 * A grid, a navmesh or a flow field agents plan on.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Navigation = number & { readonly [handleType]: "Navigation" };

/**
 * A Navigation handle field.
 *
 * A `Navigation` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Navigation(value?: Navigation, options?: FieldOptions): FieldSpec<"Navigation">;

/** NavAgent.status: not seeking. */
export declare const NAV_IDLE: 0;

/** NavAgent.status: following a path or a flow field. */
export declare const NAV_MOVING: 1;

/** NavAgent.status: within stopDistance of the target. */
export declare const NAV_ARRIVED: 2;

/** NavAgent.status: no path reaches the target, or the agent is off its grid or navmesh. */
export declare const NAV_UNREACHABLE: 3;

/** NavAgent.status: the target is cut off; moving to the reachable point nearest it. */
export declare const NAV_PARTIAL: 4;

/**
 * The grid search a path query runs.
 * - `"jps"`: Jump point search: the same optimal paths as A* with fewer nodes; on a grid with weighted cells it runs A*.
 * - `"astar"`: A* over every cell.
 */
export type NavAlgorithm = "jps" | "astar";

/**
 * An entity that moves itself towards `target` while `seeking`, by a path over `nav` (a grid, navmesh or flow field; 0 goes straight). og.nav re-plans when the target moved more than repathDistance, every repathTicks, or when the grid or navmesh changed, at most 32 plans a tick across agents.
 *
 * The value of the `NavAgent` component (token `og.NavAgent`).
 */
export interface NavAgent {
  /**
   * The grid, navmesh or flow field it plans on (og.nav.grid, og.nav.load, og.nav.flowField); 0 moves straight at the target.
   */
  nav: Navigation;
  /** Where it goes while seeking. */
  target: Vector3;
  /** Moves towards the target while true. */
  seeking: boolean;
  /**
   * Top speed.
   *
   * @default 3.5
   * @range 0..100
   * @unit mps (metres per second)
   * @tunable
   */
  speed: number;
  /**
   * How fast it changes velocity.
   *
   * @default 20
   * @range 0..1000
   * @unit mps2 (metres per second squared)
   * @tunable
   */
  acceleration: number;
  /**
   * Its radius: grid paths keep this clear of walls; separation uses it.
   *
   * @default 0.4
   * @range 0..10
   * @unit m (metres)
   * @tunable
   */
  radius: number;
  /**
   * It has arrived within this distance of the target.
   *
   * @default 0.1
   * @range 0..10
   * @unit m (metres)
   * @tunable
   */
  stopDistance: number;
  /**
   * It slows down inside this distance of the target.
   *
   * @default 1
   * @range 0..20
   * @unit m (metres)
   * @tunable
   */
  slowRadius: number;
  /**
   * Re-plans this often while seeking (0: only when the target moves).
   *
   * @default 30
   * @range 0..3600
   * @unit ticks (fixed simulation steps)
   */
  repathTicks: number;
  /**
   * Re-plans when the target moved farther than this from the last plan.
   *
   * @default 0.5
   * @range 0..100
   * @unit m (metres)
   */
  repathDistance: number;
  /**
   * Pulls grid paths straight where nothing is in the way.
   *
   * @default true
   */
  smooth: boolean;
  /**
   * Turns the entity to face where it moves (3D: about +Y, its +Z forward; 2D: about +Z, its +X forward).
   */
  faceMovement: boolean;
  /**
   * How fast it turns when facing its movement.
   *
   * @default 10
   * @range 0..100
   * @unit radps (radians per second)
   */
  turnSpeed: number;
  /**
   * Moves the entity (its og.Velocity, or its og.Transform) while true; false keeps planning but leaves the motion to the game (a knock-back, a stun).
   *
   * @default true
   */
  drive: boolean;
  /**
   * NAV_IDLE, NAV_MOVING, NAV_ARRIVED, NAV_UNREACHABLE or NAV_PARTIAL (written by og.nav).
   *
   * @readonly written by the engine
   */
  readonly status: number;
  /**
   * Its velocity (written by og.nav).
   *
   * @unit mps (metres per second)
   * @readonly written by the engine
   */
  readonly velocity: Vector3;
  /**
   * The path corner it heads for (written by og.nav).
   *
   * @unit m (metres)
   * @readonly written by the engine
   */
  readonly waypoint: Vector3;
  /**
   * The path length left (written by og.nav).
   *
   * @unit m (metres)
   * @readonly written by the engine
   */
  readonly remaining: number;
  /**
   * Paths planned so far (written by og.nav).
   *
   * @readonly written by the engine
   */
  readonly plans: number;
}

/**
 * Steering behaviours an og.NavAgent adds to its path following, each by weight (0 is off): flee, wander, pursue, evade, separation from other agents and avoidance of colliders ahead (three physics rays).
 *
 * The value of the `Steering` component (token `og.Steering`).
 */
export interface Steering {
  /**
   * Weight of following the path to the target (seek, then arrive at the end).
   *
   * @default 1
   * @range 0..10
   * @tunable
   */
  seek: number;
  /**
   * Weight of fleeing fleeFrom.
   *
   * @range 0..10
   * @tunable
   */
  flee: number;
  /**
   * What it flees.
   *
   * @unit m (metres)
   */
  fleeFrom: Vector3;
  /**
   * It flees (and evades) only inside this distance (0: always).
   *
   * @default 6
   * @range 0..100
   * @unit m (metres)
   * @tunable
   */
  fleeRadius: number;
  /**
   * Weight of wandering.
   *
   * @range 0..10
   * @tunable
   */
  wander: number;
  /**
   * The wander circle's radius.
   *
   * @default 1
   * @range 0..20
   * @unit m (metres)
   * @tunable
   */
  wanderRadius: number;
  /**
   * How far ahead the wander circle is.
   *
   * @default 2
   * @range 0..20
   * @unit m (metres)
   * @tunable
   */
  wanderDistance: number;
  /**
   * How fast the wander point moves round the circle.
   *
   * @default 3
   * @range 0..50
   * @unit radps (radians per second)
   * @tunable
   */
  wanderJitter: number;
  /** An entity to pursue (seek where it will be, from its NavAgent or og.Velocity); 0: none. */
  pursue: Entity;
  /**
   * Weight of pursuing.
   *
   * @default 1
   * @range 0..10
   * @tunable
   */
  pursueWeight: number;
  /** An entity to evade (flee where it will be); 0: none. */
  evade: Entity;
  /**
   * Weight of evading.
   *
   * @default 1
   * @range 0..10
   * @tunable
   */
  evadeWeight: number;
  /**
   * Weight of keeping apart from other agents.
   *
   * @range 0..10
   * @tunable
   */
  separation: number;
  /**
   * Other agents closer than this push it away.
   *
   * @default 1
   * @range 0..20
   * @unit m (metres)
   * @tunable
   */
  separationRadius: number;
  /**
   * Weight of avoiding colliders ahead.
   *
   * @range 0..10
   * @tunable
   */
  avoidance: number;
  /**
   * How far ahead the avoidance rays reach.
   *
   * @default 2
   * @range 0..50
   * @unit m (metres)
   * @tunable
   */
  avoidDistance: number;
  /** Collision layers the avoidance rays hit (0: every layer). */
  avoidMask: number;
  /**
   * The wander angle (written by og.nav).
   *
   * @unit rad (radians)
   * @readonly written by the engine
   */
  readonly wanderAngle: number;
}

/**
 * A grid of square cells over the plane: cell (0, 0)'s lower corner at `origin`, rows running up.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface NavGridDesc {
  /**
   * Cells across.
   *
   * @default 32
   * @range 1..4096
   */
  readonly width?: number;
  /**
   * Cells up.
   *
   * @default 32
   * @range 1..4096
   */
  readonly height?: number;
  /**
   * A cell's side.
   *
   * @default 1
   * @range 0.01..100
   * @unit m (metres)
   */
  readonly cellSize?: number;
  /**
   * Where cell (0, 0)'s lower corner is (x, y in 2D; x, z in 3D).
   *
   * @default { x: 0, y: 0 }
   * @unit m (metres)
   */
  readonly origin?: Vector2;
  /**
   * Row-major cell costs from row 0 (0 blocked, 1 normal, up to 255 to avoid); empty: every cell 1.
   *
   * @default []
   */
  readonly costs?: readonly number[];
}

/**
 * A grid over a placed og.Tilemap: one cell per map cell, blocked where a collision layer has a solid tile.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TilemapGridDesc {
  /**
   * The entity with the og.Tilemap.
   *
   * @default 0
   */
  readonly entity?: Entity;
  /**
   * The layers whose solid tiles block; empty: every collision layer.
   *
   * @default []
   */
  readonly layers?: readonly string[];
  /**
   * Also block this many cells around each blocked one (wide agents).
   *
   * @default 0
   * @range ..8
   */
  readonly inflate?: number;
}

/** A grid cell. */
export interface NavCell {
  /** The column. */
  readonly x: number;
  /** The row (0 is the bottom). */
  readonly y: number;
  /** Whether the point was on the grid. */
  readonly inside: boolean;
}

/**
 * A path query.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PathDesc {
  /**
   * The grid or navmesh.
   *
   * @default 0
   */
  readonly nav?: Navigation;
  /**
   * The start.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly from?: Vector3;
  /**
   * The end.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly to?: Vector3;
  /**
   * The grid search.
   *
   * @default "jps"
   */
  readonly algorithm?: NavAlgorithm;
  /**
   * Pull a grid path straight where nothing is in the way.
   *
   * @default true
   */
  readonly smooth?: boolean;
  /**
   * Keep this clear of grid walls when smoothing.
   *
   * @default 0
   * @range 0..10
   * @unit m (metres)
   */
  readonly radius?: number;
}

/** A path. */
export interface NavPath {
  /** Its corners, start and end included. */
  readonly points: readonly Vector3[];
  /**
   * Its length.
   *
   * @unit m (metres)
   */
  readonly length: number;
  /** Whether it reaches the end (false: it stops at the reachable point nearest it). */
  readonly complete: boolean;
  /** Cells or polygons the search took off its open list. */
  readonly expanded: number;
  /** What ran: jps, astar or navmesh. */
  readonly algorithm: string;
}

/**
 * A flow field over a grid.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface FlowDesc {
  /**
   * The grid.
   *
   * @default 0
   */
  readonly grid?: Navigation;
  /**
   * Where agents go: each point's cell is a target.
   *
   * @default []
   */
  readonly targets?: readonly Vector3[];
}

/**
 * An axis-aligned box.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface NavBox {
  /**
   * Its lowest corner.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly min?: Vector3;
  /**
   * Its highest corner.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly max?: Vector3;
}

/**
 * A navmesh built at runtime over a flat floor (a procedural level): the floor box, the boxes it carves out, and the agent it is for.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface NavMeshDesc {
  /**
   * The floor's lowest corner (its y is the floor's height).
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly min?: Vector3;
  /**
   * The floor's highest corner.
   *
   * @default { x: 16, y: 1, z: 16 }
   * @unit m (metres)
   */
  readonly max?: Vector3;
  /**
   * Boxes standing on the floor (walls, crates).
   *
   * @default []
   */
  readonly blocked?: readonly NavBox[];
  /**
   * The voxel width.
   *
   * @default 0.25
   * @range 0.02..4
   * @unit m (metres)
   */
  readonly cellSize?: number;
  /**
   * Walls are kept this far away.
   *
   * @default 0.4
   * @range 0..10
   * @unit m (metres)
   */
  readonly agentRadius?: number;
  /**
   * The agent's height.
   *
   * @default 1.8
   * @range 0.1..20
   * @unit m (metres)
   */
  readonly agentHeight?: number;
  /**
   * Cells per tile side: an obstacle rebuilds the tiles it touches.
   *
   * @default 32
   * @range 8..256
   */
  readonly tileSize?: number;
}

/** A navmesh's shape. */
export interface NavMeshInfo {
  /** What NavAgent.nav and the queries take. */
  readonly mesh: Navigation;
  /** Its convex polygons. */
  readonly polygons: number;
  /** Its tiles. */
  readonly tiles: number;
  /**
   * The lowest corner of the box around its polygons.
   *
   * @unit m (metres)
   */
  readonly min: Vector3;
  /**
   * The highest corner of the box around its polygons (its highest walkable floor's y).
   *
   * @unit m (metres)
   */
  readonly max: Vector3;
  /**
   * Its voxel width.
   *
   * @unit m (metres)
   */
  readonly cellSize: number;
}

/** What an obstacle change rebuilt. */
export interface NavRebuild {
  /** The tiles rebuilt, by index. */
  readonly tiles: readonly number[];
  /** The navmesh's polygons now. */
  readonly polygons: number;
}














// ── ai ──────────────────────────────────────────────────────────
// Enemy and NPC AI as data: behaviour trees and state machines in `.ogt` files of `type behaviour` (og-ai §3 has every line), perception (sight cones checked against physics occlusion, hearing of spatial sounds and game noises) and a blackboard per agent.
// An entity with an og.AiAgent (and an og.Transform) thinks in the og.ai system before the game's systems: perception first, then its tree or machine, whose running action moves it through its og.NavAgent (og.nav plans and walks after the game's systems) or runs one of the game's commands (GameDesc.commands, with the agent's entity handle as its first argument) before the next tick.
// Conditions read blackboard keys (declared, set by actions or og.ai.set, or built in: self, position, sees, heard, target, targetPosition, targetDistance, lastSeen, sinceSeen, heardAt, sinceHeard, elapsed), the agent's component fields (`Health.hp`) and resource fields (`Game.alarm`), and compare them with a literal or with another key or field read each time (`check Foe.hp below Tuning.fleeHp`), so thresholds live in a tuning resource. og.ai.restart starts a behaviour afresh. Deterministic; memories and blackboards are in snapshots. `dump_ai:<entity>` writes an agent's state, running node and blackboard.

/**
 * A loaded behaviour tree or state machine.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Behaviour = number & { readonly [handleType]: "Behaviour" };

/**
 * A Behaviour handle field.
 *
 * A `Behaviour` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Behaviour(value?: Behaviour, options?: FieldOptions): FieldSpec<"Behaviour">;

/** AiAgent.status: the tree or state is still running. */
export declare const AI_RUNNING: 1;

/** AiAgent.status: the tree succeeded on the last think. */
export declare const AI_SUCCESS: 2;

/** AiAgent.status: the tree failed on the last think. */
export declare const AI_FAILURE: 3;

/**
 * An entity that thinks with a behaviour. Give it an og.NavAgent to move and an og.Perception to see and hear.
 *
 * The value of the `AiAgent` component (token `og.AiAgent`).
 */
export interface AiAgent {
  /** The behaviour it runs (og.ai.load); 0 runs nothing. */
  behaviour: Behaviour;
  /**
   * Thinks while true.
   *
   * @default true
   */
  active: boolean;
  /**
   * Thinks every this many ticks (1: every tick); agents are staggered by their order.
   *
   * @default 1
   * @range 1..600
   * @unit ticks (fixed simulation steps)
   */
  thinkEvery: number;
  /**
   * AI_RUNNING, AI_SUCCESS or AI_FAILURE after the last think (0 before the first; written by og.ai).
   *
   * @readonly written by the engine
   */
  readonly status: number;
  /**
   * The state machine's state, an index into BehaviourInfo.states (written by og.ai).
   *
   * @readonly written by the engine
   */
  readonly state: number;
  /**
   * The node it ran last, an index into BehaviourInfo.nodes (written by og.ai).
   *
   * @readonly written by the engine
   */
  readonly node: number;
  /**
   * Times the state or the running node changed (written by og.ai).
   *
   * @readonly written by the engine
   */
  readonly changes: number;
}

/**
 * What an agent sees and hears. It sees the nearest og.Perceivable of another team within `sight`, inside the `fov` cone around `forward` (or the way it last moved), unless a collider is in the way; it hears a spatial sound that starts (og.SoundStarted) or a game noise (og.ai.noise) within `hearing` times its loudness, except its own: a sound of its entity or of an entity under it (og.Parent), or a noise whose NoiseDesc.source is one of them.
 *
 * The value of the `Perception` component (token `og.Perception`).
 */
export interface Perception {
  /**
   * How far it sees.
   *
   * @default 8
   * @range 0..500
   * @unit m (metres)
   * @tunable
   */
  sight: number;
  /**
   * The width of its view cone (360: all round).
   *
   * @default 120
   * @range 0..360
   * @unit deg (degrees)
   * @tunable
   */
  fov: number;
  /**
   * Its eyes are this far above its position (+y).
   *
   * @range -10..10
   * @unit m (metres)
   */
  eyeHeight: number;
  /**
   * Colliders hide what is behind them.
   *
   * @default true
   */
  occlude: boolean;
  /** Collision layers that block sight (0: every layer). */
  occluders: number;
  /**
   * How far it hears a sound of loudness 1 (a source's volume).
   *
   * @default 6
   * @range 0..500
   * @unit m (metres)
   * @tunable
   */
  hearing: number;
  /**
   * How long a lost target stays its target.
   *
   * @default 3
   * @range 0..600
   * @unit s (seconds of simulation time)
   * @tunable
   */
  memory: number;
  /** Its team: it perceives og.Perceivable entities of other teams. */
  team: number;
  /**
   * Where it looks; zero: the way it last moved (before it moves, the way its Transform faces: +Z in 3D, +X in 2D).
   */
  forward: Vector3;
  /**
   * What it sees or remembers seeing (written by og.ai).
   *
   * @readonly written by the engine
   */
  readonly target: Entity;
  /**
   * It sees the target now (written by og.ai).
   *
   * @readonly written by the engine
   */
  readonly sees: boolean;
  /**
   * Where it last saw the target (written by og.ai).
   *
   * @unit m (metres)
   * @readonly written by the engine
   */
  readonly lastSeen: Vector3;
  /**
   * Seconds since it last saw a target (written by og.ai; 1e6 before the first time).
   *
   * @default 1000000
   * @unit s (seconds of simulation time)
   * @readonly written by the engine
   */
  readonly sinceSeen: number;
  /**
   * It heard something this tick (written by og.ai).
   *
   * @readonly written by the engine
   */
  readonly heard: boolean;
  /**
   * Where it last heard something (written by og.ai).
   *
   * @unit m (metres)
   * @readonly written by the engine
   */
  readonly heardAt: Vector3;
  /**
   * Seconds since it last heard something (written by og.ai; 1e6 before the first time).
   *
   * @default 1000000
   * @unit s (seconds of simulation time)
   * @readonly written by the engine
   */
  readonly sinceHeard: number;
}

/**
 * Something agents of other teams can see.
 *
 * The value of the `Perceivable` component (token `og.Perceivable`).
 */
export interface Perceivable {
  /**
   * Its team (agents of the same team ignore it).
   *
   * @default 1
   */
  team: number;
  /**
   * Agents look at a point this far above its position (+y).
   *
   * @range -10..10
   * @unit m (metres)
   */
  eyeHeight: number;
}

/**
 * An agent's state or running node changed (systems read it the tick after).
 *
 * The payload of the `AiChanged` event (token `og.AiChanged`).
 */
export interface AiChanged {
  /** The agent. */
  readonly entity: Entity;
  /** Its state now. */
  readonly state: number;
  /** The node it runs now. */
  readonly node: number;
  /** AI_RUNNING, AI_SUCCESS or AI_FAILURE. */
  readonly status: number;
}

/** A loaded behaviour. */
export interface BehaviourInfo {
  /** What AiAgent.behaviour takes. */
  readonly behaviour: Behaviour;
  /** Its name. */
  readonly name: string;
  /** tree or machine. */
  readonly kind: string;
  /** A machine's states, by index (AiAgent.state). */
  readonly states: readonly string[];
  /** Its nodes' names, by index (AiAgent.node). */
  readonly nodes: readonly string[];
  /** Its declared blackboard keys. */
  readonly keys: readonly string[];
}

/**
 * A noise agents may hear next tick.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface NoiseDesc {
  /**
   * Where it is.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly position?: Vector3;
  /**
   * How loud: heard within Perception.hearing times this.
   *
   * @default 1
   * @range 0..100
   */
  readonly loudness?: number;
  /**
   * Who makes it (0: nobody): that entity, and the agents it is under (og.Parent), do not hear it.
   *
   * @default 0
   */
  readonly source?: Entity;
}

/** What an agent is doing. */
export interface AiDescription {
  /** Its behaviour's name. */
  readonly behaviour: string;
  /** tree or machine. */
  readonly kind: string;
  /** A machine's state (empty for a tree). */
  readonly state: string;
  /** The node it ran last, such as `chase`. */
  readonly node: string;
  /** The path to it from the root, such as `selector > sequence chase > chase`. */
  readonly path: string;
  /** running, success or failure. */
  readonly status: string;
  /** Its blackboard, keys to numbers, booleans, points and entity handles. */
  readonly blackboard: unknown;
  /** It sees its target. */
  readonly sees: boolean;
  /** Its target (0: none). */
  readonly target: Entity;
}








// ── platform ────────────────────────────────────────────────────
// Platform services as host capabilities (docs/design.md §2.8): cloud store, leaderboards, telemetry, Origin Point and AI marks. Game code names a method and its arguments, never a credential, a URL or a cookie: the host owns them. On the web the shell forwards each call to the Player through origin-checked postMessage (Player protocol 2, docs/web-export.md); natively the host answers from a local JSON store (`og run --platform-store`, default `<project>/.og/platform.json`).
// Calls are asynchronous: each returns a request number at once, and the answer arrives at the start of a later tick, when og.platform sends PlatformAnswered; read it with og.platform.result. Answers are host input, like key presses: the probe's `dump_platform` lists every call, its arguments, its answer and the ticks.
// Errors carry the Player's stable codes: offline, timeout, invalid, unauthorized, not_found, conflict, too_large, rate_limited, unavailable, failed; retryable is true for offline, timeout, unavailable and rate_limited.
// Rooms (roomCreate, roomJoin, roomSend, ...) put this engine with other players' engines: the host carries them (natively `og relay` with `og run --room`, on the web the platform's rooms through the shell, docs/platform-asks.md) and its answers and the other peers' messages arrive as events at the start of a tick (og.RoomJoined, og.RoomPeerJoined, og.RoomPeerLeft, og.RoomMessage), host input like key presses. Deterministic play over a room is og.net's lockstep.

/**
 * How a leaderboard sorts; the first submission to a board fixes it.
 * - `"desc"`: Higher is better (scores).
 * - `"asc"`: Lower is better (times).
 */
export type PlatformOrder = "desc" | "asc";

/**
 * An answer arrived: sent by og.platform at the start of the tick it was applied, read by systems the tick after (og.platform.result has it at once).
 *
 * The payload of the `PlatformAnswered` event (token `og.PlatformAnswered`).
 */
export interface PlatformAnswered {
  /** The call's request number. */
  readonly request: number;
  /** Whether it succeeded; og.platform.result has the data or the error. */
  readonly ok: boolean;
}

/**
 * A write to the store.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface StoreSetDesc {
  /**
   * The key: 1 to 64 of letters, digits, `.`, `_` and `-`.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * Any JSON value, at most 64 KiB encoded.
   *
   * @default null
   */
  readonly value?: unknown;
  /**
   * Write only if the key is at this revision (a conflict otherwise, whose error carries the revision the key is at now); 0 writes whatever is there (the last write wins).
   *
   * @default 0
   */
  readonly revision?: number;
  /**
   * Write only if the key has never been written (a conflict otherwise); revision is then ignored.
   *
   * @default false
   */
  readonly create?: boolean;
}

/**
 * A score.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LeaderboardSubmitDesc {
  /**
   * The board (default `main`).
   *
   * @default "main"
   */
  readonly board?: string;
  /**
   * The score, or a time in seconds for an ascending board.
   *
   * @default 0
   */
  readonly score?: number;
  /**
   * How the board sorts, fixed by its first submission.
   *
   * @default "desc"
   */
  readonly order?: PlatformOrder;
}

/**
 * Which entries.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LeaderboardQueryDesc {
  /**
   * The board (default `main`).
   *
   * @default "main"
   */
  readonly board?: string;
  /**
   * How many entries: the top `count`, or `count` around the player.
   *
   * @default 10
   * @range 1..100
   */
  readonly count?: number;
}

/**
 * Origin Point for something the player did.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PointDesc {
  /**
   * Why, as the game names it: `level.completed`, `daily.streak` (1 to 64 of letters, digits, `.`, `_` and `-`).
   *
   * @default ""
   */
  readonly reason?: string;
  /**
   * How many points the game asks for; the platform decides what it grants.
   *
   * @default 1
   * @range 1..1000
   */
  readonly amount?: number;
}

/**
 * Marks content the game shows as generated by AI, for the platform's disclosure.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface AiMarkDesc {
  /**
   * What: `text`, `image`, `audio`, `level`, `character` (1 to 32 of a-z, 0-9, `_` and `-`, a letter first).
   *
   * @default ""
   */
  readonly kind?: string;
  /**
   * What the player sees it as, at most 120 characters.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * The model or service, when known (at most 80 characters).
   *
   * @default ""
   */
  readonly model?: string;
}

/**
 * A save slot: a long text (a world.snapshot()) kept across as many store keys as it needs.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SaveSlotDesc {
  /**
   * The slot's name: 1 to 60 of letters, digits, `.`, `_` and `-` (`slot1`). Its keys are the name itself (the manifest) and `<slot>.a0`, `<slot>.a1`, ... or `<slot>.b0`, ... (the parts).
   *
   * @default ""
   */
  readonly slot?: string;
  /**
   * The text: at most 512 KiB of UTF-8 (eight parts of 64 KiB).
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * What a title screen shows of the slot (`{ place, hour }`), kept in the manifest so loadSlot with metaOnly reads it without the parts: any JSON, at most 4 KiB encoded.
   *
   * @default null
   */
  readonly meta?: unknown;
}

/**
 * Which slot to read, and how much of it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LoadSlotDesc {
  /**
   * The slot's name, as saveSlot wrote it.
   *
   * @default ""
   */
  readonly slot?: string;
  /**
   * Read the manifest alone: the answer's text stays null (a title's Continue label, before the player asks for the world).
   *
   * @default false
   */
  readonly metaOnly?: boolean;
}

/** Why a call failed. */
export interface PlatformError {
  /**
   * offline, timeout, invalid, unauthorized, not_found, conflict, too_large, rate_limited, unavailable or failed.
   */
  readonly code: string;
  /** What happened, in words. */
  readonly message: string;
  /** Whether the same call may succeed later. */
  readonly retryable: boolean;
  /**
   * For a conflict of a conditional write: the revision the key is at now (read it again, merge, write at it); 0 otherwise.
   */
  readonly revision: number;
}

/** The answer to one call. */
export interface PlatformResult {
  /** The call's request number. */
  readonly request: number;
  /** Its method, such as `store.get`. */
  readonly method: string;
  /** Whether it succeeded. */
  readonly ok: boolean;
  /** The answer (see each function); null on failure. */
  readonly data: unknown;
  /** Why it failed. */
  readonly error?: PlatformError;
  /** The tick it was answered at. */
  readonly tick: number;
  /**
   * The host's clock when it answered, UTC milliseconds since 1970 (PlatformInfo.clock says whose); 0 when the host gave none. A slot call (saveSlot, loadSlot) answers the time of the store answer that ended it.
   *
   * @unit ms (milliseconds)
   */
  readonly time: number;
}

/** What answers platform calls here. */
export interface PlatformInfo {
  /**
   * `player` (a Player frames the game), `local` (the native host's JSON store or the browser's own storage) or `none`.
   */
  readonly host: string;
  /**
   * The methods it answers: store.get, store.set, store.list, leaderboard.submit, leaderboard.top, leaderboard.around, telemetry, point, ai.mark.
   */
  readonly capabilities: readonly string[];
  /** Calls not answered yet. */
  readonly pending: number;
  /**
   * Whose clock og.platform.now and the answers' `time` read: `player` (the Player's answers carry its time), `browser` (the web host's own clock: the Player gives no time yet, docs/platform-asks.md), `system` (the native host's system clock), `fixed` (`og run --clock` and playtests: a clock that starts where it is told and moves with the ticks) or `none`.
   */
  readonly clock: string;
}

















/**
 * A room to create.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RoomDesc {
  /**
   * What the room is called (at most 64 of letters, digits, `.`, `_` and `-`); empty for none. The host gives the room its id.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * The most peers it takes.
   *
   * @default 2
   * @range 1..16
   */
  readonly capacity?: number;
}

/** One peer in the room. */
export interface RoomPeer {
  /** Its number: the room numbers peers in join order from 0 and never gives one twice. */
  readonly peer: number;
  /** Its name, as its host gave it; empty for none. */
  readonly name: string;
  /** Whether it is this engine. */
  readonly local: boolean;
}

/** The room this engine is in. */
export interface RoomInfo {
  /**
   * `none` (in no room), `joining` (roomCreate or roomJoin asked, not answered yet) or `in`.
   */
  readonly state: string;
  /** The room's id; empty in none. */
  readonly room: string;
  /**
   * What carries rooms here: `relay` (natively, `og run --room` through `og relay`), `player` (the platform's rooms through the Player, on the web) or `none` (roomCreate and roomJoin are refused).
   */
  readonly host: string;
  /** This engine's peer number; 0 in none. */
  readonly peer: number;
  /** Every peer present, this one included, by number. */
  readonly peers: readonly RoomPeer[];
}

/** A message another peer sent: what og.RoomMessage announced. */
export interface RoomMessageData {
  /**
   * Its number (og.RoomMessage.message): 1 for the first message this engine received, counting up.
   */
  readonly message: number;
  /** The sender's peer number. */
  readonly from: number;
  /** The tick it was delivered at. */
  readonly tick: number;
  /** What it carries, when it was sent as a JSON value; null when it was sent as bytes. */
  readonly data: unknown;
  /** What it carries, when it was sent as bytes. */
  readonly bytes: Uint8Array | null;
}

/**
 * roomCreate or roomJoin finished, or the host put this engine in a room (`og run --room`, a Play link to a room): sent at the start of the tick after the host answered. With ok false the engine is in no room and `reason` says why.
 *
 * The payload of the `RoomJoined` event (token `og.RoomJoined`).
 */
export interface RoomJoined {
  /** Whether this engine is in the room now. */
  readonly ok: boolean;
  /** The room's id (at most 63 bytes); empty when not ok. */
  readonly room: string;
  /** This engine's peer number. */
  readonly peer: number;
  /** How many peers are in the room, this one included. */
  readonly peers: number;
  /**
   * Why it failed: `unavailable` (no rooms here), `not_found`, `full`, `invalid` or `offline`, then a colon and the host's words; empty when ok.
   */
  readonly reason: string;
}

/**
 * Another peer joined this engine's room.
 *
 * The payload of the `RoomPeerJoined` event (token `og.RoomPeerJoined`).
 */
export interface RoomPeerJoined {
  /** Its peer number. */
  readonly peer: number;
  /** How many peers are in the room now. */
  readonly peers: number;
}

/**
 * A peer left this engine's room (it left, closed, or lost its connection), after every message it sent; or, with peer equal to this engine's own number, this engine is in no room any more (roomLeave, or the host lost the room).
 *
 * The payload of the `RoomPeerLeft` event (token `og.RoomPeerLeft`).
 */
export interface RoomPeerLeft {
  /** Its peer number. */
  readonly peer: number;
  /** How many peers are in the room now (0 once this engine left). */
  readonly peers: number;
}

/**
 * Another peer's message arrived; read it with og.platform.roomMessage. Messages from one peer arrive in the order it sent them. While lockstep runs (og.Lockstep) a message is this peer's host input alone: a system that changes the world from one makes it differ from the other peers' (og.Desync); carry what all must see in og.PlayerInput.
 *
 * The payload of the `RoomMessage` event (token `og.RoomMessage`).
 */
export interface RoomMessage {
  /** The sender's peer number. */
  readonly from: number;
  /** The message's number, for og.platform.roomMessage. */
  readonly message: number;
  /** Whether it was sent as bytes (else as a JSON value). */
  readonly binary: boolean;
}








// ── i18n ────────────────────────────────────────────────────────
// Localization: one string table per locale (`.ogt` of `type strings`: `locale "ja"`, its font chain `fonts "..."`, `string "key" "message"`), messages in a subset of ICU MessageFormat (`{name}`, `{n, number}`, `{n, plural, =0 {..} one {..} other {..}}` with `#`, `{g, select, a {..} other {..}}`), CLDR plural rules, and the locale the host asks for (the browser's `navigator.language` on the web, `og run --locale` natively).
// The active locale is the best loaded table for the request (ja-JP takes ja; zh-TW takes zh-Hant or zh-TW), else the base table (`base true`, else en, else the first). A key it lacks is read from its `fallback` table, then the base. It also sets the UI's font chain (its fonts first, then the other tables') and direction: a right-to-left locale (ar, he, fa, ur) mirrors og.ui's layout, and og-ui shapes Arabic words with the font's own tables (a font cooked with the `arabic` charset carries them). `og cook` warns about every key a locale lacks, or takes other arguments for, than the base.

/** The localization state. */
export interface LocaleInfo {
  /** The locale the host or the game asked for (empty: none). */
  readonly requested: string;
  /** The table in use (empty: none loaded). */
  readonly active: string;
  /** `ltr` or `rtl`. */
  readonly direction: string;
  /** Every loaded table's locale. */
  readonly locales: readonly string[];
  /** The UI's font chain, first first. */
  readonly fonts: readonly string[];
}






// ── ui ──────────────────────────────────────────────────────────
// The engine's UI: an immediate-mode toolkit. Systems describe the tree every tick with these calls; the engine lays it out Clay-style, styles it with the game's UI style (the placeholder prototype style until the game picks one), draws it over the frame and answers the probe (dump_ui, touch_move_to, move_to_text) from it.
// The engine opens the frame before the game's systems (from og.Input and the play surface) and closes it after them, so calls belong inside systems; outside one they do nothing and log a warning. Interaction is decided against the previous tick's rectangles, which are also what the probe resolves, so a scripted touch_move_to and a click agree; response edges (clicked, changed, submitted) last one tick.
// Lengths are CSS pixels of the play surface (origin top left, y down), the unit of og.Input and the probe. Ids: an element's id is unique among its siblings (`idi("Slot", 3)` is `Slot#3`); a path `Inventory/Slot#3` names it after its nearest ancestors. The UI state (hover, press, focus, caret, scroll) is world state: snapshots and the world hash carry it.

/**
 * Which way a container lays out its children.
 * - `"row"`: Left to right.
 * - `"column"`: Top to bottom.
 */
export type Direction = "row" | "column";

/**
 * Where children sit along the main axis.
 * - `"start"`: At the start.
 * - `"center"`: Centred.
 * - `"end"`: At the end.
 * - `"spaceBetween"`: The free space split evenly between children.
 */
export type Align = "start" | "center" | "end" | "spaceBetween";

/**
 * Where children sit across the main axis.
 * - `"start"`: At the start.
 * - `"center"`: Centred.
 * - `"end"`: At the end.
 * - `"stretch"`: Stretched to the container's inner size.
 */
export type Cross = "start" | "center" | "end" | "stretch";

/**
 * How the lines of a label align.
 * - `"start"`: To the start (left).
 * - `"center"`: Centred.
 * - `"end"`: To the end (right).
 */
export type TextAlign = "start" | "center" | "end";

/**
 * Which configured font text uses; display and mono fall back to body, rounded to display, then body.
 * - `"body"`: The body font.
 * - `"display"`: The display font.
 * - `"mono"`: The monospaced font.
 * - `"rounded"`: The rounded font, for levels and scores.
 */
export type FontRole = "body" | "display" | "mono" | "rounded";

/**
 * Button roles: how the style draws each.
 * - `"secondary"`: A neutral fill (the default).
 * - `"primary"`: The primary action: one per screen.
 * - `"quiet"`: Text until hovered.
 * - `"danger"`: A destructive action: danger ink on the danger tint.
 */
export type ButtonVariant = "secondary" | "primary" | "quiet" | "danger";

/**
 * Control heights from the density (h-control-sm, -md, -lg).
 * - `"medium"`: The medium control height.
 * - `"small"`: The small control height.
 * - `"large"`: The large control height.
 */
export type ControlSize = "medium" | "small" | "large";

/**
 * The palette: games run on a stage, so dark is the default.
 * - `"dark"`: The dark palette.
 * - `"light"`: The light palette.
 */
export type Scheme = "dark" | "light";

/**
 * Control sizes by the input the player holds: auto picks touch sizes on a coarse pointer or a play surface under 768 px, mouse sizes otherwise.
 * - `"auto"`: Touch sizes on a coarse pointer or a narrow surface, mouse sizes otherwise.
 * - `"mouse"`: A mouse and keyboard: the tightest controls.
 * - `"pad"`: A gamepad across a room: larger controls, rows and gutters that read from a couch.
 * - `"touch"`: A finger: every control at least 44 CSS px.
 */
export type DensityMode = "auto" | "mouse" | "pad" | "touch";

/**
 * A colour role of the UI style in effect (the game's style, or the prototype style before it picks one).
 * - `"backdrop"`: The screen behind the UI where nothing else is.
 * - `"well"`: A recessed ground inside a panel: wells, map frames, empty slots.
 * - `"panel"`: Panels and cards.
 * - `"panelHover"`: A panel under the pointer.
 * - `"panelRaised"`: Dialogs, menus and tooltips.
 * - `"line"`: Hairlines and frames.
 * - `"lineStrong"`: Stronger frames: high contrast, map borders, scrollbars.
 * - `"ink"`: The main ink.
 * - `"inkMuted"`: Secondary ink: captions, keys, inactive tabs.
 * - `"inkFaint"`: The faintest ink that still reads: placeholders, hints.
 * - `"inkInverse"`: Ink on a ground of ink (a selected chip).
 * - `"control"`: Neutral control fills: secondary buttons, tracks, fields.
 * - `"controlStrong"`: A neutral control hovered or pressed; selection.
 * - `"primary"`: The one directed action: primary buttons, a slider's fill, a switch that is on, the caret.
 * - `"primaryHover"`: primary hovered.
 * - `"primaryActive"`: primary pressed.
 * - `"onPrimary"`: Ink on primary.
 * - `"primarySoft"`: A translucent wash of primary.
 * - `"highlight"`: Emphasised ink: a speaker's name, a price, a link, the player's map dot.
 * - `"focus"`: The focus ring of key and pad navigation.
 * - `"knob"`: Slider and switch knobs.
 * - `"hud"`: HUD capsules over the game (scores, timers, toasts).
 * - `"scrim"`: Behind a dialog.
 * - `"shadow"`: Drop shadows.
 * - `"sheen"`: A skeleton's passing sheen.
 * - `"success"`: Success.
 * - `"warn"`: Warnings.
 * - `"danger"`: Danger and errors.
 * - `"info"`: Information.
 * - `"health"`: A health meter.
 * - `"mana"`: A mana meter.
 * - `"xp"`: An experience meter.
 * - `"gold"`: Currency: coins, gold, prices that can be paid.
 */
export type ThemeColor = "backdrop" | "well" | "panel" | "panelHover" | "panelRaised" | "line" | "lineStrong" | "ink" | "inkMuted" | "inkFaint" | "inkInverse" | "control" | "controlStrong" | "primary" | "primaryHover" | "primaryActive" | "onPrimary" | "primarySoft" | "highlight" | "focus" | "knob" | "hud" | "scrim" | "shadow" | "sheen" | "success" | "warn" | "danger" | "info" | "health" | "mana" | "xp" | "gold";

/**
 * A step of the style's type ramp.
 * - `"body"`: The default step.
 * - `"xs"`: A step of the ramp.
 * - `"meta"`: A step of the ramp.
 * - `"caption"`: A step of the ramp.
 * - `"sm"`: A step of the ramp.
 * - `"reading"`: A step of the ramp.
 * - `"base"`: A step of the ramp.
 * - `"lg"`: A step of the ramp.
 * - `"xl"`: A step of the ramp.
 * - `"xl2"`: A step of the ramp.
 * - `"xl3"`: A step of the ramp.
 * - `"xl4"`: A step of the ramp.
 * - `"xl5"`: A step of the ramp.
 * - `"xl6"`: A step of the ramp.
 * - `"xl7"`: A step of the ramp.
 * - `"xl8"`: A step of the ramp.
 * - `"displaySm"`: A step of the ramp.
 * - `"displayMd"`: A step of the ramp.
 * - `"displayLg"`: A step of the ramp.
 * - `"displayXl"`: A step of the ramp.
 * - `"displayXl2"`: A step of the ramp.
 */
export type TypeStepName = "body" | "xs" | "meta" | "caption" | "sm" | "reading" | "base" | "lg" | "xl" | "xl2" | "xl3" | "xl4" | "xl5" | "xl6" | "xl7" | "xl8" | "displaySm" | "displayMd" | "displayLg" | "displayXl" | "displayXl2";

/** The bounds of a fit or grow size. */
export interface SizeRange {
  /**
   * The smallest size.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly min?: number;
  /**
   * The largest size; absent is unbounded.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly max?: number;
}

/** A fixed size. */
export interface FixedSize {
  /**
   * The size.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly px: number;
}

/** A size relative to the parent. */
export interface PercentSize {
  /**
   * The fraction of the parent's inner size.
   *
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly fraction: number;
}

/** A size along one axis. Absent (null) is fit: the content's size. */
export type Size =
  /** The content's size, within the range. */
  | ({ readonly kind: "fit" } & SizeRange)
  /** The content's size, then a share of the parent's free space, within the range. */
  | ({ readonly kind: "grow" } & SizeRange)
  /** Exactly this size. */
  | ({ readonly kind: "fixed" } & FixedSize)
  /** A fraction of the parent's inner size. */
  | ({ readonly kind: "percent" } & PercentSize);

/** Padding or margins on four sides. */
export interface Insets {
  /**
   * The left side.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly left: number;
  /**
   * The top side.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly top: number;
  /**
   * The right side.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly right: number;
  /**
   * The bottom side.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly bottom: number;
}

/**
 * A colour: a theme token, or an sRGB colour as authored (components 0 to 1), with an alpha. Neither is white.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface Paint {
  /**
   * A semantic colour of the theme.
   *
   * @default null
   */
  readonly token?: ThemeColor | null;
  /**
   * An sRGB colour as authored, when no token is given.
   *
   * @default null
   */
  readonly color?: Color | null;
  /**
   * The opacity.
   *
   * @default 1
   * @range 0..1
   */
  readonly alpha?: number;
}

/**
 * A container: its layout and its look.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ElementDesc {
  /**
   * Which way it lays out its children.
   *
   * @default "row"
   */
  readonly direction?: Direction;
  /**
   * Its width; null fits the content.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height; null fits the content.
   *
   * @default null
   */
  readonly height?: Size | null;
  /**
   * Space inside its edges.
   *
   * @default { left: 0, top: 0, right: 0, bottom: 0 }
   */
  readonly padding?: Insets;
  /**
   * Space between children.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly gap?: number;
  /**
   * Where children sit along the main axis.
   *
   * @default "start"
   */
  readonly align?: Align;
  /**
   * Where children sit across it.
   *
   * @default "start"
   */
  readonly cross?: Cross;
  /**
   * Break a row into lines at its inner width.
   *
   * @default false
   */
  readonly wrap?: boolean;
  /**
   * Scroll its content vertically and clip it.
   *
   * @default false
   */
  readonly scroll?: boolean;
  /**
   * Scroll its content horizontally and clip it (with scroll: both ways).
   *
   * @default false
   */
  readonly scrollX?: boolean;
  /**
   * Report a Response from close(), as a button does.
   *
   * @default false
   */
  readonly interactive?: boolean;
  /**
   * Its fill; null draws none.
   *
   * @default null
   */
  readonly background?: Paint | null;
  /**
   * Corner radius.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly radius?: number;
  /**
   * The style frame its background takes (a HUD element, a card); null is a plain fill. Without a style the fill is drawn either way.
   *
   * @default null
   */
  readonly frame?: StylePart | null;
  /**
   * Width over height (0: none): with the height left to fit, it is as tall as its laid-out width over this; with a fixed height and the width left to fit, as wide as the height times it.
   *
   * @default 0
   * @range 0..
   */
  readonly aspect?: number;
}

/**
 * How a label draws its text.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LabelDesc {
  /**
   * The type step; null is body.
   *
   * @default null
   */
  readonly step?: TypeStepName | null;
  /**
   * The ink; null is ink.
   *
   * @default null
   */
  readonly color?: Paint | null;
  /**
   * How the lines align.
   *
   * @default "start"
   */
  readonly align?: TextAlign;
  /**
   * Keep to one line (breaking only at `\n`).
   *
   * @default false
   */
  readonly nowrap?: boolean;
  /**
   * Its width; null fits the text.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Which font.
   *
   * @default "body"
   */
  readonly font?: FontRole;
  /**
   * A heading: a style with a flourish frames it.
   *
   * @default false
   */
  readonly heading?: boolean;
  /**
   * Cut with an ellipsis where the text does not fit its width: on its one line (with nowrap; the label can then shrink in a row) or on its last line (maxLines). The probe still reads the whole text.
   *
   * @default false
   */
  readonly ellipsis?: boolean;
  /**
   * The most lines a wrapping label shows; 0 is no limit. With ellipsis the last ends in an ellipsis when text is left out.
   *
   * @default 0
   */
  readonly maxLines?: number;
}

/**
 * A button's look.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ButtonDesc {
  /**
   * The look.
   *
   * @default "secondary"
   */
  readonly variant?: ButtonVariant;
  /**
   * The control height.
   *
   * @default "medium"
   */
  readonly size?: ControlSize;
  /**
   * Neutral and inert.
   *
   * @default false
   */
  readonly disabled?: boolean;
  /**
   * Its width; null fits the label.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Take the focus when it appears (it was not on screen the tick before) and no dialog is open: what opens takes the focus (og-ui §4), so a menu's first item answers Enter and the pad's A from its first tick. og.ui.mainMenu's first item has it.
   *
   * @default false
   */
  readonly autofocus?: boolean;
}

/**
 * A card: panel, the panel radius, padded and spaced by the style.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PanelDesc {
  /**
   * Lay children left to right (the default is top to bottom).
   *
   * @default false
   */
  readonly row?: boolean;
  /**
   * Its width; null fits the content.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height; null fits the content.
   *
   * @default null
   */
  readonly height?: Size | null;
  /**
   * Space inside; null is space.lg on every side.
   *
   * @default null
   */
  readonly padding?: Insets | null;
  /**
   * Space between children; null is space.md.
   *
   * @default null
   */
  readonly gap?: number | null;
  /**
   * Where children sit along the main axis.
   *
   * @default "start"
   */
  readonly align?: Align;
  /**
   * Where children sit across it.
   *
   * @default "start"
   */
  readonly cross?: Cross;
  /**
   * Its fill; null is panel.
   *
   * @default null
   */
  readonly background?: Paint | null;
}

/**
 * Which ways a scroll container scrolls.
 * - `"vertical"`: Up and down (the default).
 * - `"horizontal"`: Left and right.
 * - `"both"`: Both ways.
 */
export type ScrollAxes = "vertical" | "horizontal" | "both";

/**
 * A scroll container: the wheel (both axes, deltaX and deltaY) or a drag scrolls it, and a released drag keeps its momentum, slowing over motion.spring.momentum's response (none under reduced motion); it clips its content; a scrollbar on each scrolling axis shows the position, and a press on one grabs and drags it. Nested containers hand off: what the innermost under the pointer cannot take (it reached its end, or does not scroll that way) goes to the one around it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ScrollDesc {
  /**
   * Its width; null fits the content.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height; null fits the content.
   *
   * @default null
   */
  readonly height?: Size | null;
  /**
   * Space inside.
   *
   * @default { left: 0, top: 0, right: 0, bottom: 0 }
   */
  readonly padding?: Insets;
  /**
   * Space between children.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly gap?: number;
  /**
   * Which ways it scrolls.
   *
   * @default "vertical"
   */
  readonly axes?: ScrollAxes;
}

/**
 * A texture region, or a solid rectangle without a texture.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ImageDesc {
  /**
   * The texture; null draws a solid rectangle of the tint.
   *
   * @default null
   */
  readonly texture?: Texture | null;
  /**
   * The region drawn, [u0, v0, u1, v1] with v down; null is the whole texture.
   *
   * @default null
   */
  readonly uv?: [number, number, number, number] | null;
  /**
   * Multiplies the texture; null is white.
   *
   * @default null
   */
  readonly tint?: Paint | null;
  /**
   * Its width; null fits.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height; null fits.
   *
   * @default null
   */
  readonly height?: Size | null;
  /**
   * Width over height (0: none): with the height left to fit, the image is as tall as its laid-out width over this (a growing column keeps a picture's proportions); with a fixed height and the width left to fit, as wide as the height times it.
   *
   * @default 0
   * @range 0..
   */
  readonly aspect?: number;
  /**
   * What a screen reader calls it (and the probe's text): a picture that means something (a portrait, a star earned) names it in the game's words; empty is decoration.
   *
   * @default ""
   */
  readonly label?: string;
}

/**
 * A zoomable picture (a map, an artwork): what it shows and its zoom limits.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ZoomViewDesc {
  /**
   * The picture; null draws a solid rectangle of the tint.
   *
   * @default null
   */
  readonly texture?: Texture | null;
  /**
   * The picture's region, [u0, v0, u1, v1] with v down; null is the whole texture.
   *
   * @default null
   */
  readonly uv?: [number, number, number, number] | null;
  /**
   * Multiplies the texture; null is white.
   *
   * @default null
   */
  readonly tint?: Paint | null;
  /**
   * Its width; null fits.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height; null fits.
   *
   * @default null
   */
  readonly height?: Size | null;
  /**
   * Width over height (0: none), as ImageDesc.aspect.
   *
   * @default 0
   * @range 0..
   */
  readonly aspect?: number;
  /**
   * The least zoom; 0 is 1 (the whole picture).
   *
   * @default 0
   * @range 0..
   */
  readonly minZoom?: number;
  /**
   * The most zoom; 0 is 4.
   *
   * @default 0
   * @range 0..
   */
  readonly maxZoom?: number;
}

/** Where a zoom view is. */
export interface ZoomResult {
  /** Its zoom: 1 shows the whole picture, 2 half of it each way. */
  readonly zoom: number;
  /** The middle of what it shows, as a fraction of the picture's width (0.5 at rest). */
  readonly centerX: number;
  /** The middle of what it shows, as a fraction of the picture's height (0.5 at rest). */
  readonly centerY: number;
}

/**
 * A progress bar.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ProgressDesc {
  /**
   * Its width; null grows into the free space.
   *
   * @default null
   */
  readonly width?: Size | null;
}

/**
 * A slider's range and width.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SliderDesc {
  /**
   * The smallest value; min equal to max (the zero value) means 0 to 1.
   *
   * @default 0
   */
  readonly min?: number;
  /**
   * The largest value.
   *
   * @default 0
   */
  readonly max?: number;
  /**
   * Snap to multiples of step from min; 0 is continuous.
   *
   * @default 0
   * @range 0..
   */
  readonly step?: number;
  /**
   * Its width; null grows.
   *
   * @default null
   */
  readonly width?: Size | null;
}

/**
 * A one-line text field.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface FieldDesc {
  /**
   * Shown, dimmed, while the field is empty.
   *
   * @default ""
   */
  readonly placeholder?: string;
  /**
   * Its width; null grows.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * The most characters it takes; 0 is no limit.
   *
   * @default 0
   */
  readonly maxChars?: number;
}

/**
 * A multi-line text area.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TextAreaDesc {
  /**
   * Shown, dimmed, while the area is empty.
   *
   * @default ""
   */
  readonly placeholder?: string;
  /**
   * Its width; null grows.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Lines shown at once; 0 is 4. More scroll.
   *
   * @default 0
   */
  readonly rows?: number;
  /**
   * The most characters it takes, line breaks included; 0 is no limit.
   *
   * @default 0
   */
  readonly maxChars?: number;
}

/**
 * What happened to an element this tick. Edges (clicked, changed, submitted) are true for exactly one tick.
 */
export interface Response {
  /** The pointer is over it and no other element holds the press. */
  readonly hovered: boolean;
  /** The press that started on it is still held. */
  readonly pressed: boolean;
  /** Pressed and released inside it. */
  readonly clicked: boolean;
  /** Its value changed (slider, toggle, text field). */
  readonly changed: boolean;
  /**
   * It has focus: a text field being edited, or the control keyboard and gamepad navigation landed on.
   */
  readonly focused: boolean;
  /** Enter was pressed in it (text field). */
  readonly submitted: boolean;
  /**
   * A list row or grid cell a press started on was released over this one: that row's or cell's index (drag and drop).
   */
  readonly dropped: number | null;
  /**
   * The gamepad (0 to 3) whose A pressed it this tick; null when the pointer or the keyboard did. Pads 1 to 3 press only while og.ui.setPlayers allows them (local multiplayer).
   */
  readonly pad: number | null;
  /**
   * Bit k: pad k's focus is on it (bit 0 is the shared focus of the pointer, the keyboard and pad 0; bits 1 to 3 the other players' pads).
   */
  readonly focusedBy: number;
}

/** A slider's response and its value after this tick's input. */
export interface SliderResult {
  /** What happened to it. */
  readonly response: Response;
  /** The value, to keep in game state and pass back next tick. */
  readonly value: number;
}

/** A toggle's response and its state after this tick's input. */
export interface ToggleResult {
  /** What happened to it. */
  readonly response: Response;
  /** The state, to keep in game state and pass back next tick. */
  readonly on: boolean;
}

/** A text field's response and its text after this tick's input. */
export interface FieldResult {
  /** What happened to it. */
  readonly response: Response;
  /** The text, to keep in game state and pass back next tick. */
  readonly value: string;
}

/** The play surface the UI lays out in. */
export interface Viewport {
  /**
   * Its width.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly width: number;
  /**
   * Its height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly height: number;
  /**
   * Device pixels per CSS pixel; rectangles snap to device pixels.
   *
   * @default 1
   */
  readonly dpr: number;
  /** The device's unsafe margins (notch, home indicator); the root pads by them. */
  readonly safeArea: Insets;
  /** A touch screen or another coarse pointer. */
  readonly coarsePointer: boolean;
  /**
   * The page prefers reduced motion (prefers-reduced-motion: reduce, from the web shell): the UI's theme only fades, whatever UiDesc.reducedMotion says. Omitted is false.
   */
  readonly reducedMotion?: boolean;
  /**
   * The page prefers more contrast (prefers-contrast: more, from the web shell): the UI's theme is the high-contrast one, whatever UiDesc.contrast says. Omitted is false.
   */
  readonly highContrast?: boolean;
  /**
   * The page prefers less transparency (prefers-reduced-transparency: reduce, from the web shell): glass parts are opaque surfaces and the scrim and translucent fills flat, whatever UiDesc.reducedTransparency says (UiTheme.reducedTransparency). Omitted is false.
   */
  readonly reducedTransparency?: boolean;
  /**
   * Text controls take macOS's editing keys: Command with A, C, X and Z, Command+←/→ to the line's ends (↑/↓ the text's), Option+←/→ and Option+Backspace by words, Control+A and E to the line's ends. Off: Control takes the shortcuts and moves by words (Alt too), Home and End go to the line's ends. The web shell sets it from the browser's platform (Apple's); a native host sets it on macOS, so a script that holds Ctrl or Meta edits as a person on that OS would (og.ui.previewViewport pins either). Omitted is false.
   */
  readonly macKeys?: boolean;
  /**
   * Where the host's own control covers the surface: on a published page the web shell's round Back to the Portal, top left (about 12, 12 to 52, 52 CSS px; its name shows beside it for the first seconds of play). Anchored groups keep out of it, moved along the edge they sit on (a top-left group to its right), so the HUD templates never draw under it; a game drawing its own corner reads it from og.ui.frameViewport. Empty (omitted) is none: native hosts and pages without Back.
   */
  readonly covered?: UiRect;
  /** Its size class: phone portrait or landscape, tablet, desktop. */
  readonly breakpoint: Breakpoint;
}

/**
 * How the UI looks: the scheme, the density, its fonts, the contrast, the game's primary colour, motion, transparency, the icon atlas, the style and the sounds. og.ui.configure takes it in part: a field left out keeps the value the last call (or a look's style) gave it, null clears it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface UiDesc {
  /**
   * The palette.
   *
   * @default "dark"
   */
  readonly scheme?: Scheme;
  /**
   * The control sizes.
   *
   * @default "auto"
   */
  readonly density?: DensityMode;
  /**
   * The body font, from og.load.font; null measures text at half an em per character and draws none.
   *
   * @default null
   */
  readonly font?: Font | null;
  /**
   * The display font; null uses the body font.
   *
   * @default null
   */
  readonly displayFont?: Font | null;
  /**
   * The monospaced font; null uses the body font.
   *
   * @default null
   */
  readonly monoFont?: Font | null;
  /**
   * The rounded font (levels and scores); null uses the display font, then the body font.
   *
   * @default null
   */
  readonly roundedFont?: Font | null;
  /**
   * Fonts for other scripts (Hebrew, Devanagari, a CJK face with `font { runtime true }`), tried in order after the locale's (og.i18n) for a text the role's font cannot draw whole. A text mixing scripts takes each run from the first font that has it (the role's for its Latin, a fallback for its CJK or Arabic), one label in several fonts.
   *
   * @default []
   */
  readonly fallbackFonts?: readonly Font[];
  /**
   * Standard or high contrast. The page's prefers-contrast: more takes high contrast too (Viewport.highContrast).
   *
   * @default "standard"
   */
  readonly contrast?: Contrast;
  /**
   * An sRGB colour as the game's primary (its hover, pressed and soft tones derived from it, the ink on it whichever of ink and inkInverse reads better); null keeps the style's.
   *
   * @default null
   */
  readonly primaryColor?: Color | null;
  /**
   * Transitions only fade; sheens stop. The page's prefers-reduced-motion turns it on too (Viewport.reducedMotion).
   *
   * @default false
   */
  readonly reducedMotion?: boolean;
  /**
   * Nothing is see-through: glass parts (a style's glass line) are opaque surfaces (no blur of the frame under them, their frame whole), the scrim is the opaque colour it makes over backdrop and the translucent fills (control, controlStrong, primarySoft, hud) the opaque colours they make over panel. The page's prefers-reduced-transparency turns it on too (Viewport.reducedTransparency); UiTheme.reducedTransparency says it is on.
   *
   * @default false
   */
  readonly reducedTransparency?: boolean;
  /**
   * The icon atlas og.ui.icon draws from; null takes the style's.
   *
   * @default null
   */
  readonly icons?: IconAtlasDesc | null;
  /**
   * Where toasts stack; null is the bottom centre.
   *
   * @default null
   */
  readonly toastAnchor?: Anchor | null;
  /**
   * A UI style from og.load.style: its palette, radii, type ramp, fonts, icons, frames, ornaments, motion and sounds replace the prototype style (fonts, icons and a primary colour given here still win); null is the prototype style, the engine's placeholder.
   *
   * @default null
   */
  readonly style?: Style | null;
  /**
   * Sounds for UI events (they win over the style's).
   *
   * @default null
   */
  readonly sounds?: UiSounds | null;
}

/**
 * Sounds the UI plays through og-audio (the ui bus) on its events; null is silent.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface UiSounds {
  /**
   * The pointer reaches a control, or keyboard or gamepad focus moves to one.
   *
   * @default null
   */
  readonly hover?: Sound | null;
  /**
   * A control is pressed: by the pointer, Enter, Space or the gamepad's A.
   *
   * @default null
   */
  readonly press?: Sound | null;
  /**
   * A dialog opens.
   *
   * @default null
   */
  readonly open?: Sound | null;
  /**
   * A dialog closes.
   *
   * @default null
   */
  readonly close?: Sound | null;
  /**
   * A success toast.
   *
   * @default null
   */
  readonly success?: Sound | null;
  /**
   * A refused press or a danger toast.
   *
   * @default null
   */
  readonly error?: Sound | null;
}

/**
 * An element class a style dresses with a nine-slice frame.
 * - `"panel"`: Dialogs and screen panels.
 * - `"card"`: Cards.
 * - `"button"`: Buttons (with hover, pressed and disabled states).
 * - `"field"`: Text fields.
 * - `"slot"`: Inventory and grid cells.
 * - `"tooltip"`: Tooltips.
 * - `"hud"`: HUD elements over the game (counters, timers) and toasts.
 * - `"track"`: A progress bar's track.
 */
export type StylePart = "panel" | "card" | "button" | "field" | "slot" | "tooltip" | "hud" | "track";

/** One step of the type ramp. */
export interface TypeStep {
  /**
   * The font size.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size: number;
  /**
   * The leading of Latin text.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly lineHeight: number;
  /** Latin tracking, in em. */
  readonly trackingEm: number;
  /**
   * The leading text with Chinese, Japanese or Korean characters takes.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly cjkLineHeight: number;
}

/** The control, row and header heights and the gutter of the density in effect. */
export interface ControlSizes {
  /**
   * A small control's height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly controlSm: number;
  /**
   * A control's height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly controlMd: number;
  /**
   * A large control's height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly controlLg: number;
  /**
   * The gutter.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly gutter: number;
  /**
   * The header height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly header: number;
  /**
   * The row height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly row: number;
  /**
   * The section spacing.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly section: number;
}

/** The style values the UI draws with, read-only. */
export interface UiTheme {
  /** The palette in use. */
  readonly scheme: Scheme;
  /** Each ThemeColor as `#rrggbb` or `#rrggbbaa` (sRGB as authored). */
  readonly palette: Readonly<Record<string, string>>;
  /** The spacing scale by name (xxs, xs, sm, md, lg, xl, xxl, xl3), in CSS pixels. */
  readonly space: Readonly<Record<string, number>>;
  /** The corner radii by name (small, control, bar, panel), in CSS pixels. */
  readonly radius: Readonly<Record<string, number>>;
  /** Each TypeStepName's step. */
  readonly typeRamp: Readonly<Record<string, TypeStep>>;
  /** The control sizes of the density in effect. */
  readonly sizes: ControlSizes;
  /**
   * The motion durations by name (instant, fast, base, slow, spring, bounce), in milliseconds.
   */
  readonly durationsMs: Readonly<Record<string, number>>;
  /** The configured style's name; empty for the prototype style. */
  readonly style: string;
  /**
   * The variant in effect: high when the game asked for it or the page prefers more contrast.
   */
  readonly contrast: Contrast;
  /**
   * Transitions only fade and sheens stop: the game asked for it or the page prefers reduced motion. A game's own animations read it to keep their state changes and drop their movement.
   */
  readonly reducedMotion: boolean;
  /**
   * Nothing is see-through: the scrim and the translucent fills of `palette` are already the opaque colours they make over panel (the scrim over backdrop) and glass is an opaque surface; the game asked for it or the page prefers reduced transparency. A game's own translucent overlays read it to draw opaque too.
   */
  readonly reducedTransparency: boolean;
}

/** Builds a container's children: ui calls made inside it nest under the container. */
export type UiBuild = () => void;












/**
 * How a rich text paragraph draws: its defaults, which its markup's spans override.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RichTextDesc {
  /**
   * The type step; null is body.
   *
   * @default null
   */
  readonly step?: TypeStepName | null;
  /**
   * The ink; null is ink.
   *
   * @default null
   */
  readonly color?: Paint | null;
  /**
   * How the lines align.
   *
   * @default "start"
   */
  readonly align?: TextAlign;
  /**
   * Keep to one line (breaking only at `\n`).
   *
   * @default false
   */
  readonly nowrap?: boolean;
  /**
   * Its width; null fits the text.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Which font.
   *
   * @default "body"
   */
  readonly font?: FontRole;
}

/** What a rich text's links did this tick. */
export interface RichTextResult {
  /** Its links together: hovered while one is, clicked when one was. */
  readonly response: Response;
  /** The link clicked this tick: its `[link=…]` id. */
  readonly link: string | null;
  /** The link under the pointer. */
  readonly hovered: string | null;
}













/**
 * A point of the safe area an anchored group sits at.
 * - `"topLeft"`: The top left corner.
 * - `"top"`: The middle of the top edge.
 * - `"topRight"`: The top right corner.
 * - `"left"`: The middle of the left edge.
 * - `"center"`: The centre.
 * - `"right"`: The middle of the right edge.
 * - `"bottomLeft"`: The bottom left corner.
 * - `"bottom"`: The middle of the bottom edge.
 * - `"bottomRight"`: The bottom right corner.
 */
export type Anchor = "topLeft" | "top" | "topRight" | "left" | "center" | "right" | "bottomLeft" | "bottom" | "bottomRight";

/**
 * How a transition moves its content in and out. Under reduced motion every kind only fades.
 * - `"fade"`: Opacity only.
 * - `"slideUp"`: Rises into place from below.
 * - `"slideDown"`: Drops into place from above.
 * - `"slideLeft"`: Enters from the right.
 * - `"slideRight"`: Enters from the left.
 * - `"scale"`: Grows from slightly smaller around its centre.
 * - `"pop"`: Springs up from smaller around its centre, overshoots and settles.
 */
export type TransitionKind = "fade" | "slideUp" | "slideDown" | "slideLeft" | "slideRight" | "scale" | "pop";

/**
 * A status tone of badges, toasts and map markers.
 * - `"neutral"`: Neutral: control fill and muted ink.
 * - `"success"`: Success.
 * - `"warn"`: Warning (orange).
 * - `"danger"`: Danger.
 * - `"info"`: Information.
 * - `"primary"`: The game's identity: the primary's soft wash and highlight ink.
 * - `"inverse"`: Neutral emphasis: ink fill, inverse ink.
 */
export type Tone = "neutral" | "success" | "warn" | "danger" | "info" | "primary" | "inverse";

/**
 * What a meter measures, which sets its colour.
 * - `"health"`: health; warn under 30 %, danger under 15 %.
 * - `"mana"`: mana.
 * - `"xp"`: xp.
 * - `"primary"`: primary.
 */
export type MeterKind = "health" | "mana" | "xp" | "primary";

/**
 * The play surface's size class.
 * - `"desktop"`: 1024 CSS px wide or more, the shorter side 600 or more.
 * - `"tablet"`: The shorter side 600 or more, under 1024 wide.
 * - `"phoneLandscape"`: The shorter side under 600, wider than tall.
 * - `"phonePortrait"`: The shorter side under 600, taller than wide.
 */
export type Breakpoint = "desktop" | "tablet" | "phoneLandscape" | "phonePortrait";

/**
 * The theme variant: high contrast remaps muted inks, fills and lines to the style's stronger ones and outlines every control (no new colour).
 * - `"standard"`: The style's palette as it is.
 * - `"high"`: Stronger inks, fills and edges.
 */
export type Contrast = "standard" | "high";

/**
 * A focus-navigation intent, as the arrow keys, Tab, Enter, Space, Escape and a gamepad's d-pad, stick, A and B give.
 * - `"up"`: Up.
 * - `"down"`: Down.
 * - `"left"`: Left.
 * - `"right"`: Right.
 * - `"next"`: The next control in reading order (Tab).
 * - `"prev"`: The previous one (Shift+Tab).
 * - `"activate"`: Press the focused control (Enter, Space, A).
 * - `"back"`: Leave: the topmost dialog or menu closes, else og.ui.back() answers true (Escape, B).
 */
export type NavIntent = "up" | "down" | "left" | "right" | "next" | "prev" | "activate" | "back";

/** A rectangle of the play surface. */
export interface UiRect {
  /**
   * Its left edge.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly x: number;
  /**
   * Its top edge.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly y: number;
  /**
   * Its width.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly width: number;
  /**
   * Its height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly height: number;
}

/**
 * The icon atlas: a texture of equal cells in a grid, named in reading order, or an icon set cooked like a font (font).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface IconAtlasDesc {
  /**
   * An icon set from og.load.font (an `.ogt` of `type icons` naming SVGs, such as the UI kit's iconoir set): its icons are distance-field glyphs, crisp at any size and tinted by their ink, named by the set; texture, columns, rows and names are then not used.
   *
   * @default null
   */
  readonly font?: Font | null;
  /**
   * The atlas texture, from og.load.texture.
   *
   * @default null
   */
  readonly texture?: Texture | null;
  /**
   * Cells across.
   *
   * @default 1
   * @range 1..
   */
  readonly columns?: number;
  /**
   * Cells down.
   *
   * @default 1
   * @range 1..
   */
  readonly rows?: number;
  /**
   * Each cell's name, in reading order: og.ui.icon draws a cell by its name.
   *
   * @default []
   */
  readonly names?: readonly string[];
}

/**
 * An anchored group: a layer at a point of the safe area, over the tree (HUD corners, a centred title, a bar of touch controls).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface AnchorDesc {
  /**
   * Where it sits.
   *
   * @default "topLeft"
   */
  readonly anchor?: Anchor;
  /**
   * The inset from the safe area's edges; null is the density's gutter.
   *
   * @default null
   */
  readonly margin?: number | null;
  /**
   * Moves it right after placing.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly offsetX?: number;
  /**
   * Moves it down after placing.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly offsetY?: number;
  /**
   * Which way it lays out its children.
   *
   * @default "row"
   */
  readonly direction?: Direction;
  /**
   * Space between children.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly gap?: number;
  /**
   * Space inside its edges.
   *
   * @default { left: 0, top: 0, right: 0, bottom: 0 }
   */
  readonly padding?: Insets;
  /**
   * Where children sit along the main axis.
   *
   * @default "start"
   */
  readonly align?: Align;
  /**
   * Where children sit across it.
   *
   * @default "start"
   */
  readonly cross?: Cross;
  /**
   * Its fill; null draws none.
   *
   * @default null
   */
  readonly background?: Paint | null;
  /**
   * Corner radius.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly radius?: number;
  /**
   * Its width; null fits the content.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height; null fits the content.
   *
   * @default null
   */
  readonly height?: Size | null;
  /**
   * Draw over dialogs and spotlights (a tutorial callout, a dragged item); the default draws under them.
   *
   * @default false
   */
  readonly over?: boolean;
  /**
   * The style frame its background takes; null is the panel's.
   *
   * @default null
   */
  readonly frame?: StylePart | null;
  /**
   * Shown settled the first time it is built: no style entrance or stagger, its controls answering at once. For the screen a game opens on while it is that: the web draws it behind the play gate, where an entrance would stay part-way until the tap.
   *
   * @default false
   */
  readonly instant?: boolean;
}

/**
 * How a transition moves, timed in ticks from the style's motion.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TransitionDesc {
  /**
   * The movement.
   *
   * @default "fade"
   */
  readonly kind?: TransitionKind;
  /**
   * How long entering and leaving take; 0 is motion.duration.base (240 ms).
   *
   * @default 0
   * @range 0..
   * @unit ms (milliseconds)
   */
  readonly durationMs?: number;
  /**
   * How far slides travel; 0 is space.xl.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly distance?: number;
  /**
   * Shown at once the first time it is built (no entrance).
   *
   * @default false
   */
  readonly instant?: boolean;
}

/** Where a transition or dialog is this tick. */
export interface Presence {
  /** Build its content: shown, entering or leaving. */
  readonly visible: boolean;
  /**
   * 0 gone, 1 settled.
   *
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly progress: number;
  /** Moving in. */
  readonly entering: boolean;
  /** Moving out. */
  readonly leaving: boolean;
}

/**
 * A dialog: a scrim over everything and a centred panel. Focus moves in and stays while it is the topmost; Back and the backdrop dismiss it unless locked.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ModalDesc {
  /**
   * A title over the content; empty is none.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * The panel's width; null is 420 CSS px, at most the surface less its gutters.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Back and the backdrop do not dismiss it (a choice must be made).
   *
   * @default false
   */
  readonly locked?: boolean;
  /**
   * A press on the backdrop does not dismiss it.
   *
   * @default false
   */
  readonly ignoreBackdrop?: boolean;
  /**
   * How it enters and leaves; null is scale.
   *
   * @default null
   */
  readonly transition?: TransitionKind | null;
  /**
   * The backdrop's fill; null is scrim.
   *
   * @default null
   */
  readonly backdrop?: Paint | null;
  /**
   * No panel: the content sits centred on the backdrop (full-screen menus, loading).
   *
   * @default false
   */
  readonly bare?: boolean;
}

/** A dialog's answer this tick. */
export interface ModalResult {
  /** Build its content (open, or leaving). */
  readonly visible: boolean;
  /** Back or a press on the backdrop asked to close it: set your open flag false. */
  readonly dismissed: boolean;
}

/**
 * A spotlight: the surface dimmed except a hole around a target.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SpotlightDesc {
  /**
   * Room around the target; 0 is space.sm.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly padding?: number;
}

/**
 * A virtualised list: only the rows in view (and one more each side) are built, fillers keep the scroll range of every row.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ListDesc {
  /**
   * How many rows there are.
   *
   * @default 0
   */
  readonly count?: number;
  /**
   * Every row's height; 0 is the density's row height.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly rowHeight?: number;
  /**
   * Space between rows.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly gap?: number;
  /**
   * Its width; null fits.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height (it scrolls inside); null grows.
   *
   * @default null
   */
  readonly height?: Size | null;
  /**
   * Space inside.
   *
   * @default { left: 0, top: 0, right: 0, bottom: 0 }
   */
  readonly padding?: Insets;
  /**
   * The selected row (controlStrong), for og.ui.list.
   *
   * @default null
   */
  readonly selected?: number | null;
}

/** The rows (or cells) to build this tick: first up to, not including, last. */
export interface ListRange {
  /** The first index. */
  readonly first: number;
  /** One past the last index. */
  readonly last: number;
}

/** What happened in a list or grid this tick. */
export interface ListResult {
  /** The row or cell pressed (or activated) this tick. */
  readonly clicked: number | null;
  /** A drag and drop ended: the index it started on. */
  readonly dropFrom: number | null;
  /** And the index it was released over. */
  readonly dropTo: number | null;
}

/**
 * A virtualised grid of equal cells (inventories, level select), scrolling vertically.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GridDesc {
  /**
   * How many cells there are.
   *
   * @default 0
   */
  readonly count?: number;
  /**
   * A cell's width; 0 is 64 (never under the 44 px touch target). With columns set and a width other than fit, the cells share the grid's width instead, as wide as it allows (never under the touch target), and keep their height.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly cellWidth?: number;
  /**
   * A cell's height; 0 is its width.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly cellHeight?: number;
  /**
   * Cells across, exactly, whatever the width: a fit grid is that many cells of cellWidth, any other the cells share; 0 is as many cells of cellWidth as fit.
   *
   * @default 0
   */
  readonly columns?: number;
  /**
   * Space between cells; 0 is space.sm.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly gap?: number;
  /**
   * Its width; null fits (exactly the columns when columns is set).
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height; null grows.
   *
   * @default null
   */
  readonly height?: Size | null;
  /**
   * Space inside.
   *
   * @default { left: 0, top: 0, right: 0, bottom: 0 }
   */
  readonly padding?: Insets;
  /**
   * The selected cell, for og.ui.grid.
   *
   * @default null
   */
  readonly selected?: number | null;
}

/**
 * A segmented control of tabs.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TabsDesc {
  /**
   * Equal segments across the width.
   *
   * @default false
   */
  readonly fill?: boolean;
  /**
   * Its width; null fits (grows with fill).
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * What a screen reader calls the row of tabs (its accessible name), never drawn; empty is its id. A localised game passes its string table's.
   *
   * @default ""
   */
  readonly label?: string;
}

/** A choice control's response and the chosen index after this tick's input. */
export interface ChoiceResult {
  /** What happened to it; changed when the choice moved. */
  readonly response: Response;
  /** The chosen index, to keep in game state and pass back next tick. */
  readonly selected: number;
}

/**
 * A select: a trigger that opens a menu of options.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SelectDesc {
  /**
   * Shown while nothing is chosen.
   *
   * @default ""
   */
  readonly placeholder?: string;
  /**
   * Its width; null grows from 160.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Neutral and inert.
   *
   * @default false
   */
  readonly disabled?: boolean;
  /**
   * What a screen reader calls it (its accessible name; the chosen option is its value), never drawn; empty is its id. A localised game passes its string table's.
   *
   * @default ""
   */
  readonly label?: string;
}

/**
 * A number stepper's range, step and digits.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface StepperDesc {
  /**
   * The smallest value; min equal to max (the zero value) is unbounded.
   *
   * @default 0
   */
  readonly min?: number;
  /**
   * The largest value.
   *
   * @default 0
   */
  readonly max?: number;
  /**
   * How much a press changes it; 0 is 1.
   *
   * @default 0
   * @range 0..
   */
  readonly step?: number;
  /**
   * Digits after the point.
   *
   * @default 0
   * @range ..6
   */
  readonly decimals?: number;
}

/** A stepper's response and value after this tick's input. */
export interface StepperResult {
  /** What happened to it. */
  readonly response: Response;
  /** The value, to keep in game state and pass back next tick. */
  readonly value: number;
}

/**
 * How a radio group lays out, and its name.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RadioDesc {
  /**
   * Options in a row (the default is a column).
   *
   * @default false
   */
  readonly horizontal?: boolean;
  /**
   * What a screen reader calls the group (its accessible name), never drawn; empty is its id. A localised game passes its string table's.
   *
   * @default ""
   */
  readonly label?: string;
}

/** A checkbox list's response and values after this tick's input. */
export interface CheckListResult {
  /** What happened to it; changed when any box flipped. */
  readonly response: Response;
  /** The values, to keep in game state and pass back next tick. */
  readonly values: readonly boolean[];
}

/**
 * A toast's tone and time.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ToastDesc {
  /**
   * Its tone: a dot of the tone's colour before the text.
   *
   * @default "neutral"
   */
  readonly tone?: Tone;
  /**
   * How long it stays; 0 is 3200 ms (6000 for danger).
   *
   * @default 0
   * @range 0..
   * @unit ms (milliseconds)
   */
  readonly durationMs?: number;
}

/**
 * A chip's state.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ChipDesc {
  /**
   * Selected: inverted to ink.
   *
   * @default false
   */
  readonly selected?: boolean;
  /**
   * Neutral and inert.
   *
   * @default false
   */
  readonly disabled?: boolean;
}

/**
 * A meter: a track with a fill that animates its changes by the tick.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface MeterDesc {
  /**
   * What it measures, which sets its colour.
   *
   * @default "health"
   */
  readonly kind?: MeterKind;
  /**
   * A colour of your own instead of the kind's.
   *
   * @default null
   */
  readonly color?: Paint | null;
  /**
   * Text inside the bar (such as `75 / 100`); empty is none.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Its width; null grows.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height; 0 is space.md, or the caption's line with a label.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly height?: number;
  /**
   * Notches dividing it into this many parts; 0 or 1 is none.
   *
   * @default 0
   */
  readonly segments?: number;
}

/**
 * A cooldown ring's size and centre.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CooldownDesc {
  /**
   * Its diameter; 0 is 56 (never under 44).
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
  /**
   * Text in the middle (a key, seconds left); empty is none.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * An icon of the atlas in the middle, by name; empty is none.
   *
   * @default ""
   */
  readonly icon?: string;
}

/**
 * An icon's size, ink and name.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface IconDesc {
  /**
   * Its size; 0 is 24.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
  /**
   * Its ink; null is ink.
   *
   * @default null
   */
  readonly tint?: Paint | null;
  /**
   * What a screen reader calls it (and the probe's text): null is the icon's name in the atlas (an English word: `star`); empty is decoration, named by nothing, so a card or button it sits on is named by its other text. A localised game names a meaningful icon from its string table.
   *
   * @default null
   */
  readonly label?: string | null;
}

/**
 * A marker on a mini-map.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface MapMarker {
  /**
   * Across the map, 0 to 1.
   *
   * @default 0
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly x?: number;
  /**
   * Down the map, 0 to 1.
   *
   * @default 0
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly y?: number;
  /**
   * Its colour; neutral is the stage ink.
   *
   * @default "neutral"
   */
  readonly tone?: Tone;
  /**
   * The player: the primary, larger, ringed.
   *
   * @default false
   */
  readonly player?: boolean;
}

/**
 * A mini-map frame.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface MinimapDesc {
  /**
   * The map; null is the stage colour.
   *
   * @default null
   */
  readonly texture?: Texture | null;
  /**
   * The region of the map shown, [u0, v0, u1, v1]; null is the whole texture.
   *
   * @default null
   */
  readonly uv?: [number, number, number, number] | null;
  /**
   * Its side; 0 is 160.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
  /**
   * Markers over the map.
   *
   * @default []
   */
  readonly markers?: readonly MapMarker[];
  /**
   * A zone name under it; empty is none.
   *
   * @default ""
   */
  readonly label?: string;
}

/**
 * A dialogue box: a portrait slot, the speaker and the line typed out by the tick.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DialogueDesc {
  /**
   * Who speaks (primary ink).
   *
   * @default ""
   */
  readonly speaker?: string;
  /**
   * The line. A new text starts typing again.
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * A portrait texture; null is the speaker's initial.
   *
   * @default null
   */
  readonly portrait?: Texture | null;
  /**
   * The portrait's region of the texture; null is the whole texture.
   *
   * @default null
   */
  readonly portraitUv?: [number, number, number, number] | null;
  /**
   * A portrait from the icon atlas by name, when no texture is given; empty is the speaker's initial.
   *
   * @default ""
   */
  readonly portraitIcon?: string;
  /**
   * Typing speed; 0 is 40.
   *
   * @default 0
   * @range 0..
   */
  readonly charsPerSecond?: number;
  /**
   * The portrait on the right.
   *
   * @default false
   */
  readonly portraitRight?: boolean;
}

/** A dialogue box's answer this tick. */
export interface DialogueResult {
  /** Characters shown. */
  readonly shown: number;
  /** The whole line is shown. */
  readonly done: boolean;
  /** Pressed (or activated) while done: show the next line. */
  readonly advanced: boolean;
}

/**
 * A virtual joystick.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface JoystickDesc {
  /**
   * The base's diameter; 0 is 128 (never under three touch targets).
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
  /**
   * The ignored radius; 0 is 0.15.
   *
   * @default 0
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly deadZone?: number;
}

/** A joystick's vector this tick. */
export interface JoystickResult {
  /** -1 left to 1 right; 0 inside the dead zone. */
  readonly x: number;
  /** -1 up to 1 down; 0 inside the dead zone. */
  readonly y: number;
  /** A press that started on it is held. */
  readonly active: boolean;
}

/**
 * A d-pad.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DpadDesc {
  /**
   * The pad's size; 0 is 132 (each arm at least 44).
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
}

/** Which arms of a d-pad are held this tick. */
export interface DpadResult {
  /** Up is held. */
  readonly up: boolean;
  /** Down is held. */
  readonly down: boolean;
  /** Left is held. */
  readonly left: boolean;
  /** Right is held. */
  readonly right: boolean;
}

/**
 * A square touch action button (jump, fire), cornered by the style's control radius.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ActionButtonDesc {
  /**
   * Its side; 0 is 64 (never under 44).
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
  /**
   * The primary button (one per screen).
   *
   * @default false
   */
  readonly primary?: boolean;
  /**
   * An icon of the atlas by name, instead of the label; empty is the label.
   *
   * @default ""
   */
  readonly icon?: string;
}

/**
 * A skeleton: the neutral fill standing in for content that is loading.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SkeletonDesc {
  /**
   * Its width; null grows.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * A line's height; 0 is the body size.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly height?: number;
  /**
   * Stacked lines; 0 or 1 is one, the last of several is shorter.
   *
   * @default 0
   */
  readonly lines?: number;
}

/**
 * Builds one row or cell of a virtualised list or grid: ui calls made inside nest under it.
 *
 * @param index Its index.
 */
export type UiItemBuild = (index: number) => void;


















































/**
 * One node of the accessibility tree: an element with a role (layout containers and spacers have none and are left out).
 */
export interface A11yNode {
  /** Its path, as touch_move_to and og.ui.focus name it. */
  readonly path: string;
  /** The path of the nearest node above it in the tree; empty at the top. */
  readonly parent: string;
  /**
   * Its role (probe.UiElement.role): button, link, text, heading, image, checkbox, radio, switch, slider, spinbutton, textbox, combobox, listbox, option, tab, tablist, radiogroup, progressbar, meter, timer, status, dialog, tooltip, separator, group.
   */
  readonly role: string;
  /**
   * Its accessible name: what a screen reader says it is. A text field or area is named by the label before it in its row, else by its placeholder, else by its id; never by the text in it (that is its value).
   */
  readonly name: string;
  /**
   * Read after the name (aria-description): a text field's placeholder when a label names it; empty for none.
   */
  readonly description: string;
  /**
   * Its value as text (a slider's number, a progress's percent, a field's text); empty for none.
   */
  readonly value: string;
  /**
   * Its states: disabled, focused, focusable, checked, selected, expanded, pressed, busy, offscreen (a scroll container cuts it out of view).
   */
  readonly states: readonly string[];
  /**
   * A range's smallest value (aria-valuemin): a slider's min, a bounded stepper's, 0 for a progress (in percent) and a meter; null for a stepper without bounds and a control that is no range.
   */
  readonly valueMin: number | null;
  /**
   * A range's largest value (aria-valuemax): a slider's max, a bounded stepper's, 100 for a progress, a meter's max; null as valueMin is.
   */
  readonly valueMax: number | null;
  /**
   * A range's value as a number (aria-valuenow; value says it as people read it): a slider's, a stepper's, a meter's, a progress's percent; null for a control that is no range.
   */
  readonly valueNow: number | null;
  /**
   * Left edge of the whole element.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly x: number;
  /**
   * Top edge of the whole element.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly y: number;
  /**
   * Width.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly width: number;
  /**
   * Height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly height: number;
}

/**
 * The last frame's accessibility tree, built from the same nodes the UI draws and takes input on.
 */
export interface A11yTree {
  /** The nodes, in tree order: the page, then each layer over it (a dialog after the page). */
  readonly nodes: readonly A11yNode[];
  /**
   * The paths keyboard Tab and the pads visit, in order: the nodes marked focusable, within the topmost dialog while one is open.
   */
  readonly focusOrder: readonly string[];
  /** The path keyboard and pad 0's focus is on; empty for none. */
  readonly focused: string;
}






/**
 * A number that counts to its value.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CounterDesc {
  /**
   * The type step; null is body.
   *
   * @default null
   */
  readonly step?: TypeStepName | null;
  /**
   * Which font: mono (the numeric role) for counts in a row, display for a big score.
   *
   * @default "body"
   */
  readonly font?: FontRole;
  /**
   * The ink; null is ink.
   *
   * @default null
   */
  readonly color?: Paint | null;
  /**
   * How it aligns in its width.
   *
   * @default "start"
   */
  readonly align?: TextAlign;
  /**
   * Its width; null fits the text.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * How long a change takes to count; 0 is 900 ms.
   *
   * @default 0
   * @range 0..
   * @unit ms (milliseconds)
   */
  readonly durationMs?: number;
  /**
   * Where the first count starts; null starts at the value (no count).
   *
   * @default null
   */
  readonly from?: number | null;
  /**
   * Digits after the point.
   *
   * @default 0
   */
  readonly decimals?: number;
  /**
   * Text before the number.
   *
   * @default ""
   */
  readonly prefix?: string;
  /**
   * Text after the number.
   *
   * @default ""
   */
  readonly suffix?: string;
}


/**
 * Rays and sparks thrown from an element.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface BurstDesc {
  /**
   * Rays; 0 is 8.
   *
   * @default 0
   */
  readonly count?: number;
  /**
   * Their ink; null is the primary.
   *
   * @default null
   */
  readonly color?: Paint | null;
  /**
   * How far they fly from the element's centre; 0 is 1.4 times its size.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly radius?: number;
  /**
   * How long they fly; 0 is motion.duration.slow plus base.
   *
   * @default 0
   * @range 0..
   * @unit ms (milliseconds)
   */
  readonly durationMs?: number;
  /**
   * Wait this long after the element first appears.
   *
   * @default 0
   * @range 0..
   * @unit ms (milliseconds)
   */
  readonly delayMs?: number;
}


/**
 * A shake of an element: three sideways swings decaying to rest.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ShakeDesc {
  /**
   * The first swing; 0 is the style's motion.shake, else 6.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly amount?: number;
  /**
   * How long it takes to come to rest; 0 is motion.duration.slow.
   *
   * @default 0
   * @range 0..
   * @unit ms (milliseconds)
   */
  readonly durationMs?: number;
  /**
   * Also play the style's error sound, as a refused press does.
   *
   * @default false
   */
  readonly sound?: boolean;
}


/**
 * How a screen transition covers the old screen and reveals the new one.
 * - `"fade"`: The cover fades in and out.
 * - `"wipe"`: An edge sweeps across.
 * - `"iris"`: A circle closes on the centre and opens again.
 * - `"dissolve"`: Cells appear in a fixed scattered order.
 * - `"slide"`: A sheet slides over and away.
 */
export type ScreenWipe = "fade" | "wipe" | "iris" | "dissolve" | "slide";

/**
 * Where a wipe or a slide comes from.
 * - `"left"`: From the left.
 * - `"right"`: From the right.
 * - `"top"`: From the top.
 * - `"bottom"`: From the bottom.
 */
export type WipeSide = "left" | "right" | "top" | "bottom";

/**
 * How a screen transition looks.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ScreenTransitionDesc {
  /**
   * The movement.
   *
   * @default "fade"
   */
  readonly kind?: ScreenWipe;
  /**
   * Cover and reveal together; 0 is twice motion.duration.slow.
   *
   * @default 0
   * @range 0..
   * @unit ms (milliseconds)
   */
  readonly durationMs?: number;
  /**
   * The cover's colour (the texture's tint); null is backdrop.
   *
   * @default null
   */
  readonly color?: Paint | null;
  /**
   * A texture stretched over the surface as the cover; null is the flat colour.
   *
   * @default null
   */
  readonly texture?: Texture | null;
  /**
   * Where a wipe or a slide comes from.
   *
   * @default "left"
   */
  readonly from?: WipeSide;
}

/** Where a screen transition is this tick. */
export interface ScreenTransitionResult {
  /** The screen to build this tick: the old one until the cover closes, then the new one. */
  readonly shown: string;
  /** The cover is closing over the old screen. */
  readonly covering: boolean;
  /** The cover is opening on the new screen. */
  readonly revealing: boolean;
  /**
   * How much of the surface is covered, 0 to 1.
   *
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly cover: number;
}



/**
 * A graphics quality preset.
 * - `"medium"`: Medium.
 * - `"low"`: Low: the fewest effects.
 * - `"high"`: High: every effect.
 */
export type GraphicsQuality = "medium" | "low" | "high";

/**
 * A template's action, by its id (a menu item, `buy:<item>`, `share`).
 *
 * @param action The action's id.
 */
export type UiAction = (action: string) => void;

/**
 * The settings after a change: persist them (cloud save, local storage through the shell) and apply them.
 *
 * @param settings The new settings.
 */
export type SettingsChanged = (settings: GameSettings) => void;

/**
 * An item of a menu: an id the game acts on and the label a player reads.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface MenuItem {
  /**
   * The id answered when it is chosen; also its probe id.
   *
   * @default ""
   */
  readonly id?: string;
  /**
   * What it says.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Shown but inert.
   *
   * @default false
   */
  readonly disabled?: boolean;
  /**
   * The primary action (one per screen).
   *
   * @default false
   */
  readonly primary?: boolean;
}

/** What a menu-like template answered this tick. */
export interface MenuResult {
  /** The id of the item chosen this tick. */
  readonly selected: string | null;
}

/**
 * A title screen: the title, a subtitle, a column of items (the first primary unless one says so) and a footer. Centred on desktop and tablet, bottom-weighted on a phone.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface MainMenuDesc {
  /**
   * The game's title, in display type.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * A line under it; empty is none.
   *
   * @default ""
   */
  readonly subtitle?: string;
  /**
   * The items, such as Play, Continue, Settings, Quit.
   *
   * @default []
   */
  readonly items?: readonly MenuItem[];
  /**
   * A line at the bottom (a version, credits); empty is none.
   *
   * @default ""
   */
  readonly footer?: string;
  /**
   * Called with the chosen item's id.
   *
   * @default null
   */
  readonly onSelect?: UiAction | null;
  /**
   * Shown settled the first time it is built (AnchorDesc.instant): pass it while the title is the screen the game opens on, so the frame behind the web's play gate is the whole title; a title the player comes back to enters with the style.
   *
   * @default false
   */
  readonly instant?: boolean;
}

/**
 * A pause screen: a dialog over the frozen game with Resume first. Back resumes.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PauseMenuDesc {
  /**
   * Whether the game is paused.
   *
   * @default false
   */
  readonly open?: boolean;
  /**
   * Its title.
   *
   * @default "Paused"
   */
  readonly title?: string;
  /**
   * The items; empty is Resume, Settings, Quit to menu (`resume`, `settings`, `quit`).
   *
   * @default []
   */
  readonly items?: readonly MenuItem[];
  /**
   * Called with the chosen item's id; Back answers `resume`.
   *
   * @default null
   */
  readonly onSelect?: UiAction | null;
}

/**
 * A control the settings screen lists: an action and the key bound to it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface KeyBinding {
  /**
   * The action's id.
   *
   * @default ""
   */
  readonly action?: string;
  /**
   * What a player reads.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * The bound key, as shown (`Space`, `A`, `←`).
   *
   * @default ""
   */
  readonly key?: string;
}

/**
 * A language the settings screen offers, in its own name.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LanguageOption {
  /**
   * Its code (`en`, `zh-CN`).
   *
   * @default ""
   */
  readonly code?: string;
  /**
   * Its name in itself (`English`, `简体中文`).
   *
   * @default ""
   */
  readonly label?: string;
}

/**
 * The settings most games share. The game keeps them (a resource, a save) and passes them in every tick.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GameSettings {
  /**
   * The master volume.
   *
   * @default 1
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly masterVolume?: number;
  /**
   * The music bus.
   *
   * @default 0.8
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly musicVolume?: number;
  /**
   * The effects bus.
   *
   * @default 1
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly sfxVolume?: number;
  /**
   * Everything silent.
   *
   * @default false
   */
  readonly muted?: boolean;
  /**
   * The graphics preset.
   *
   * @default "medium"
   */
  readonly quality?: GraphicsQuality;
  /**
   * Show the frame rate.
   *
   * @default false
   */
  readonly showFps?: boolean;
  /**
   * Keep state changes, drop movement (the UI's transitions only fade).
   *
   * @default false
   */
  readonly reducedMotion?: boolean;
  /**
   * The high-contrast theme.
   *
   * @default false
   */
  readonly highContrast?: boolean;
  /**
   * Vibrate on hits (where the device can).
   *
   * @default true
   */
  readonly vibration?: boolean;
  /**
   * Invert the vertical look axis.
   *
   * @default false
   */
  readonly invertY?: boolean;
  /**
   * Look sensitivity.
   *
   * @default 1
   * @range 0.1..3
   */
  readonly sensitivity?: number;
  /**
   * The language code.
   *
   * @default "en"
   */
  readonly language?: string;
  /**
   * The controls, as shown on the Controls tab.
   *
   * @default []
   */
  readonly bindings?: readonly KeyBinding[];
}

/**
 * A settings screen: Audio, Graphics, Controls and Language tabs (a select on a phone) over the game's settings, with persistence hooks.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SettingsDesc {
  /**
   * Whether it is shown.
   *
   * @default false
   */
  readonly open?: boolean;
  /**
   * The settings from game state.
   *
   * @default {}
   */
  readonly settings?: GameSettings;
  /**
   * The languages offered; empty hides the Language tab.
   *
   * @default []
   */
  readonly languages?: readonly LanguageOption[];
  /**
   * Called on every change (apply it at once: volume, contrast).
   *
   * @default null
   */
  readonly onChange?: SettingsChanged | null;
  /**
   * Called when the screen closes after changes (persist here).
   *
   * @default null
   */
  readonly onSave?: SettingsChanged | null;
  /**
   * The dialog's title. Every word of the screen is a field with its English default: a localised game passes its string table's (og.i18n.t).
   *
   * @default "Settings"
   */
  readonly title?: string;
  /**
   * The Audio tab.
   *
   * @default "Audio"
   */
  readonly audioLabel?: string;
  /**
   * The Graphics tab.
   *
   * @default "Graphics"
   */
  readonly graphicsLabel?: string;
  /**
   * The Controls tab.
   *
   * @default "Controls"
   */
  readonly controlsLabel?: string;
  /**
   * The Language tab.
   *
   * @default "Language"
   */
  readonly languageLabel?: string;
  /**
   * The master volume's slider.
   *
   * @default "Master"
   */
  readonly masterLabel?: string;
  /**
   * The music volume's slider.
   *
   * @default "Music"
   */
  readonly musicLabel?: string;
  /**
   * The effects volume's slider.
   *
   * @default "Effects"
   */
  readonly effectsLabel?: string;
  /**
   * The mute toggle.
   *
   * @default "Mute everything"
   */
  readonly muteLabel?: string;
  /**
   * The graphics preset's heading, also the presets' group's accessible name.
   *
   * @default "Quality"
   */
  readonly qualityLabel?: string;
  /**
   * The low preset.
   *
   * @default "Low"
   */
  readonly lowLabel?: string;
  /**
   * The medium preset.
   *
   * @default "Medium"
   */
  readonly mediumLabel?: string;
  /**
   * The high preset.
   *
   * @default "High"
   */
  readonly highLabel?: string;
  /**
   * The frame-rate toggle.
   *
   * @default "Show frame rate"
   */
  readonly fpsLabel?: string;
  /**
   * The reduced-motion toggle.
   *
   * @default "Reduce motion"
   */
  readonly motionLabel?: string;
  /**
   * The high-contrast toggle.
   *
   * @default "High contrast"
   */
  readonly contrastLabel?: string;
  /**
   * The look sensitivity's slider.
   *
   * @default "Sensitivity"
   */
  readonly sensitivityLabel?: string;
  /**
   * The invert-Y checkbox.
   *
   * @default "Invert vertical look"
   */
  readonly invertLabel?: string;
  /**
   * The vibration checkbox.
   *
   * @default "Vibration"
   */
  readonly vibrationLabel?: string;
  /**
   * The button that closes the screen.
   *
   * @default "Done"
   */
  readonly doneLabel?: string;
  /**
   * What a screen reader calls the row of tabs (the select on a phone): the group's accessible name, never drawn.
   *
   * @default "Section"
   */
  readonly sectionLabel?: string;
  /**
   * What a screen reader calls the list of languages: the group's accessible name, never drawn.
   *
   * @default "Languages"
   */
  readonly languagesLabel?: string;
}

/** The settings screen's answer this tick. */
export interface SettingsResult {
  /** The settings after this tick's input: keep them. */
  readonly settings: GameSettings;
  /** Something changed this tick. */
  readonly changed: boolean;
  /** The player closed the screen (Back, Done): set open false. */
  readonly closed: boolean;
}

/**
 * An ability on a HUD: a cooldown ring with its key.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface Ability {
  /**
   * Its id (and probe id).
   *
   * @default ""
   */
  readonly id?: string;
  /**
   * The key or button shown in the middle.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * An icon of the atlas instead of the key; empty is the key.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * How much of its cooldown is left: 1 just used, 0 ready.
   *
   * @default 0
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly remaining?: number;
}

/**
 * A platformer HUD: lives as hearts and a level name top left, coins and score top right, a timer, a pause button. What a game leaves out it does not draw: no hearts with maxLives 0 (and no health), no coins or score capsule without coins or score.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PlatformerHudDesc {
  /**
   * Lives left.
   *
   * @default 0
   */
  readonly lives?: number;
  /**
   * Lives at most (hearts drawn); 0 draws no hearts and no capsule for them (a game without lives).
   *
   * @default 3
   */
  readonly maxLives?: number;
  /**
   * Coins collected; null or left out draws no coins capsule.
   *
   * @default null
   */
  readonly coins?: number | null;
  /**
   * The score; null or left out draws no score capsule.
   *
   * @default null
   */
  readonly score?: number | null;
  /**
   * The level's name; empty is none.
   *
   * @default ""
   */
  readonly level?: string;
  /**
   * Seconds left; null hides the timer.
   *
   * @default null
   */
  readonly timeLeft?: number | null;
  /**
   * A health bar instead of hearts, 0 to 1; null shows hearts.
   *
   * @default null
   */
  readonly health?: number | null;
  /**
   * Show touch controls (a d-pad and Jump) on phones and tablets.
   *
   * @default false
   */
  readonly touch?: boolean;
  /**
   * The touch Jump button. Every word of the HUD is a field with its English default: a localised game passes its string table's (og.i18n.t).
   *
   * @default "Jump"
   */
  readonly jumpLabel?: string;
  /**
   * The pause button's name (what a screen reader says; the button shows its icon).
   *
   * @default "Pause"
   */
  readonly pauseLabel?: string;
}

/** What a HUD answered this tick. */
export interface HudResult {
  /** The pause button was pressed (or Back). */
  readonly paused: boolean;
  /**
   * A HUD action pressed this tick (an ability's id, `jump`, a generator's `buy:<id>`, `tap`).
   */
  readonly action: string | null;
  /** The touch controls' horizontal input, -1 to 1. */
  readonly moveX: number;
  /** The touch controls' vertical input, -1 (up) to 1. */
  readonly moveY: number;
  /** The touch action buttons held this tick. */
  readonly held: readonly string[];
}

/**
 * A shooter HUD: health and armor bottom left, ammo bottom right, abilities as cooldown rings, kills top right, a crosshair, an optional mini-map.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ShooterHudDesc {
  /**
   * Health.
   *
   * @default 0
   */
  readonly health?: number;
  /**
   * Full health.
   *
   * @default 100
   */
  readonly maxHealth?: number;
  /**
   * Armor; 0 with maxArmor 0 hides it.
   *
   * @default 0
   */
  readonly armor?: number;
  /**
   * Full armor.
   *
   * @default 0
   */
  readonly maxArmor?: number;
  /**
   * Rounds in the magazine.
   *
   * @default 0
   */
  readonly ammo?: number;
  /**
   * Rounds a magazine holds.
   *
   * @default 0
   */
  readonly magazine?: number;
  /**
   * Rounds in reserve.
   *
   * @default 0
   */
  readonly reserve?: number;
  /**
   * The weapon's name.
   *
   * @default ""
   */
  readonly weapon?: string;
  /**
   * Kills.
   *
   * @default 0
   */
  readonly kills?: number;
  /**
   * Abilities with cooldowns.
   *
   * @default []
   */
  readonly abilities?: readonly Ability[];
  /**
   * Draw a crosshair at the centre.
   *
   * @default true
   */
  readonly crosshair?: boolean;
  /**
   * A mini-map top left; null is none.
   *
   * @default null
   */
  readonly map?: MinimapDesc | null;
  /**
   * Show touch controls (a stick and Fire) on phones and tablets.
   *
   * @default false
   */
  readonly touch?: boolean;
  /**
   * The word after the kill count. Every word of the HUD is a field with its English default: a localised game passes its string table's (og.i18n.t).
   *
   * @default "kills"
   */
  readonly killsLabel?: string;
  /**
   * The touch Fire button.
   *
   * @default "Fire"
   */
  readonly fireLabel?: string;
  /**
   * The pause button's name (what a screen reader says; the button shows its icon).
   *
   * @default "Pause"
   */
  readonly pauseLabel?: string;
}

/**
 * A puzzle HUD: level and moves, the score towards a target with three star marks, an optional timer.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PuzzleHudDesc {
  /**
   * The level's name.
   *
   * @default ""
   */
  readonly level?: string;
  /**
   * The score.
   *
   * @default 0
   */
  readonly score?: number;
  /**
   * The score that wins.
   *
   * @default 1000
   */
  readonly target?: number;
  /**
   * The scores for one, two and three stars; empty is a third, two thirds and all of the target.
   *
   * @default []
   */
  readonly stars?: readonly number[];
  /**
   * Moves left; null hides it.
   *
   * @default null
   */
  readonly moves?: number | null;
  /**
   * Seconds left; null hides it.
   *
   * @default null
   */
  readonly timeLeft?: number | null;
  /**
   * The caption under the moves left. Every word of the HUD is a field with its English default: a localised game passes its string table's (og.i18n.t).
   *
   * @default "moves left"
   */
  readonly movesLabel?: string;
  /**
   * The line beside the stars; `{target}` is replaced by the target, formatted (1,000). A localised game passes its string table's, with `{target}` in it.
   *
   * @default "Target {target}"
   */
  readonly targetLabel?: string;
  /**
   * The pause button's name (what a screen reader says; the button shows its icon).
   *
   * @default "Pause"
   */
  readonly pauseLabel?: string;
}

/**
 * A currency an idle game counts.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface Currency {
  /**
   * Its id.
   *
   * @default ""
   */
  readonly id?: string;
  /**
   * Its name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * How much there is.
   *
   * @default 0
   */
  readonly amount?: number;
  /**
   * How much it earns per second.
   *
   * @default 0
   */
  readonly perSecond?: number;
  /**
   * An icon of the atlas; empty is none.
   *
   * @default ""
   */
  readonly icon?: string;
}

/**
 * A generator an idle game lists: level, cost, progress.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface Generator {
  /**
   * Its id: buying answers `buy:<id>`.
   *
   * @default ""
   */
  readonly id?: string;
  /**
   * Its name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its level.
   *
   * @default 0
   */
  readonly level?: number;
  /**
   * The next level's price.
   *
   * @default 0
   */
  readonly cost?: number;
  /**
   * The player can pay for it.
   *
   * @default false
   */
  readonly affordable?: boolean;
  /**
   * Its cycle, 0 to 1.
   *
   * @default 0
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly progress?: number;
  /**
   * An icon of the atlas; empty is none.
   *
   * @default ""
   */
  readonly icon?: string;
}

/**
 * An idle or incremental HUD: currencies with their rates on top, a big tap target, the generators in a list.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface IdleHudDesc {
  /**
   * The currencies, the first the main one.
   *
   * @default []
   */
  readonly currencies?: readonly Currency[];
  /**
   * The generators.
   *
   * @default []
   */
  readonly generators?: readonly Generator[];
  /**
   * The tap target's label; empty hides it.
   *
   * @default "Tap"
   */
  readonly tapLabel?: string;
  /**
   * A generator's level badge; `{level}` is replaced by its level. Every word of the HUD is a field with its English default: a localised game passes its string table's (og.i18n.t), with `{level}` in it.
   *
   * @default "Lv {level}"
   */
  readonly levelLabel?: string;
  /**
   * A currency's rate; `{rate}` is replaced by its per-second amount, formatted (+12/s is `+{rate}/s`). A localised game passes its string table's, with `{rate}` in it.
   *
   * @default "+{rate}/s"
   */
  readonly rateLabel?: string;
  /**
   * The pause button's name (what a screen reader says; the button shows its icon).
   *
   * @default "Pause"
   */
  readonly pauseLabel?: string;
}

/**
 * One split of a racing HUD: a sector's time and its delta against the reference (the record's split).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RaceSplit {
  /**
   * Its name (`S1`, a sector's), the game's own word.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Its time in seconds, shown as m:ss.mmm; null is not yet run (a dash).
   *
   * @default null
   */
  readonly time?: number | null;
  /**
   * Against the reference in seconds, shown signed (`-0.412`, `+1.050`, with the ASCII minus every pack draws): negative is ahead, in success ink; positive behind, in danger ink; null shows none.
   *
   * @default null
   */
  readonly delta?: number | null;
}

/**
 * A racing HUD: position and lap top left, the times top right (the race time, a live delta, the best, the splits), the speedometer and gear bottom right, the boost meter, an optional mini-map. A time trial sets racers 0 (no place) and boostMeter false (no meter), and shows its sectors as splits.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RacingHudDesc {
  /**
   * The speed, in `unit`.
   *
   * @default 0
   */
  readonly speed?: number;
  /**
   * The dial's top.
   *
   * @default 300
   */
  readonly maxSpeed?: number;
  /**
   * The speed's unit.
   *
   * @default "km/h"
   */
  readonly unit?: string;
  /**
   * The gear (`3`, `R`, `N`).
   *
   * @default ""
   */
  readonly gear?: string;
  /**
   * The lap.
   *
   * @default 1
   */
  readonly lap?: number;
  /**
   * Laps in the race.
   *
   * @default 3
   */
  readonly laps?: number;
  /**
   * The player's place.
   *
   * @default 1
   */
  readonly position?: number;
  /**
   * Cars in the race; 0 hides the place (a time trial).
   *
   * @default 8
   */
  readonly racers?: number;
  /**
   * The race time in seconds.
   *
   * @default 0
   */
  readonly time?: number;
  /**
   * A live delta in seconds against the reference lap, under the time, signed like a split's: negative (ahead) in success ink, positive in danger ink; null hides it.
   *
   * @default null
   */
  readonly delta?: number | null;
  /**
   * The best lap in seconds; null hides it.
   *
   * @default null
   */
  readonly best?: number | null;
  /**
   * Splits under the times (a time trial's sectors), one row each: label, time, delta; empty is none.
   *
   * @default []
   */
  readonly splits?: readonly RaceSplit[];
  /**
   * The boost meter, 0 to 1.
   *
   * @default 0
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly boost?: number;
  /**
   * Draw the boost meter and the touch Boost pedal; a race without boost (a time trial) sets false.
   *
   * @default true
   */
  readonly boostMeter?: boolean;
  /**
   * A mini-map; null is none.
   *
   * @default null
   */
  readonly map?: MinimapDesc | null;
  /**
   * Show touch controls (steering arrows, Brake, Boost) on phones and tablets.
   *
   * @default false
   */
  readonly touch?: boolean;
  /**
   * The caption under the lap. Every word of the HUD is a field with its English default (unit too): a localised game passes its string table's (og.i18n.t).
   *
   * @default "lap"
   */
  readonly lapLabel?: string;
  /**
   * The best lap's line; `{best}` is replaced by the best lap as m:ss.mmm. A localised game passes its string table's, with `{best}` in it.
   *
   * @default "Best {best}"
   */
  readonly bestLabel?: string;
  /**
   * The pause button's name (what a screen reader says; the button shows its icon).
   *
   * @default "Pause"
   */
  readonly pauseLabel?: string;
  /**
   * The boost meter's label and the touch Boost pedal.
   *
   * @default "Boost"
   */
  readonly boostLabel?: string;
  /**
   * The touch Brake pedal.
   *
   * @default "Brake"
   */
  readonly brakeLabel?: string;
  /**
   * The touch steering arrow left.
   *
   * @default "Left"
   */
  readonly leftLabel?: string;
  /**
   * The touch steering arrow right.
   *
   * @default "Right"
   */
  readonly rightLabel?: string;
}

/**
 * One line of a cutscene.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DialogueLine {
  /**
   * Who speaks.
   *
   * @default ""
   */
  readonly speaker?: string;
  /**
   * What they say.
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * An icon of the atlas as the portrait; empty is the speaker's initial.
   *
   * @default ""
   */
  readonly portrait?: string;
}

/**
 * A dialogue scene: letterbox bars, the dialogue box at the bottom, Skip at the top right.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CutsceneDesc {
  /**
   * Whether it plays.
   *
   * @default false
   */
  readonly open?: boolean;
  /**
   * The lines.
   *
   * @default []
   */
  readonly lines?: readonly DialogueLine[];
  /**
   * The line shown, from game state.
   *
   * @default 0
   */
  readonly index?: number;
  /**
   * Cinematic bars.
   *
   * @default true
   */
  readonly letterbox?: boolean;
  /**
   * Show Skip.
   *
   * @default true
   */
  readonly skippable?: boolean;
  /**
   * The Skip button. Every word of the cutscene is a field with its English default: a localised game passes its string table's (og.i18n.t).
   *
   * @default "Skip"
   */
  readonly skipLabel?: string;
}

/** A cutscene's answer this tick. */
export interface CutsceneResult {
  /** The line to show next tick: keep it. */
  readonly index: number;
  /** The last line was advanced past. */
  readonly finished: boolean;
  /** Skip was pressed (or Back). */
  readonly skipped: boolean;
}

/**
 * An item a shop sells.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ShopItem {
  /**
   * Its id: buying answers it.
   *
   * @default ""
   */
  readonly id?: string;
  /**
   * Its name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its price in the shop's currency.
   *
   * @default 0
   */
  readonly price?: number;
  /**
   * A line about it.
   *
   * @default ""
   */
  readonly description?: string;
  /**
   * An icon of the atlas; empty is none.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * Its category tab; empty is the first.
   *
   * @default ""
   */
  readonly category?: string;
  /**
   * Already owned (one-off items): shown, not for sale.
   *
   * @default false
   */
  readonly owned?: boolean;
}

/**
 * A shop: the balance, category tabs, a grid of items with prices, a detail panel with Buy (disabled when the balance is short).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ShopDesc {
  /**
   * Its title.
   *
   * @default "Shop"
   */
  readonly title?: string;
  /**
   * The currency's name (`coins`).
   *
   * @default "coins"
   */
  readonly currency?: string;
  /**
   * The currency's icon; empty is none.
   *
   * @default ""
   */
  readonly currencyIcon?: string;
  /**
   * What the player has.
   *
   * @default 0
   */
  readonly balance?: number;
  /**
   * What is for sale.
   *
   * @default []
   */
  readonly items?: readonly ShopItem[];
  /**
   * Category tabs; empty is one list.
   *
   * @default []
   */
  readonly categories?: readonly string[];
  /**
   * Called with the bought item's id.
   *
   * @default null
   */
  readonly onBuy?: UiAction | null;
  /**
   * The detail panel's price line. Every word of the shop is a field with its English default (title and currency too): a localised game passes its string table's (og.i18n.t).
   *
   * @default "Price"
   */
  readonly priceLabel?: string;
  /**
   * The Buy button.
   *
   * @default "Buy"
   */
  readonly buyLabel?: string;
  /**
   * The Buy button while the balance is short of the price.
   *
   * @default "Not enough"
   */
  readonly shortLabel?: string;
  /**
   * An owned item's badge and button.
   *
   * @default "Owned"
   */
  readonly ownedLabel?: string;
  /**
   * The detail panel when the category has no items.
   *
   * @default "Nothing for sale here"
   */
  readonly emptyLabel?: string;
}

/** A shop's answer this tick. */
export interface ShopResult {
  /** The item bought this tick: take the price and grant it. */
  readonly bought: string | null;
  /**
   * The item whose Buy was pressed this tick while the balance was short: the button shook with the error sound (og.ui.shake) and nothing was bought.
   */
  readonly refused: string | null;
  /** Close was pressed (or Back). */
  readonly closed: boolean;
}

/**
 * What an inventory slot holds.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface InventoryItem {
  /**
   * The slot index.
   *
   * @default 0
   */
  readonly slot?: number;
  /**
   * The item's id.
   *
   * @default ""
   */
  readonly id?: string;
  /**
   * Its name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * How many (stacks); 0 or 1 hides the count.
   *
   * @default 0
   */
  readonly count?: number;
  /**
   * An icon of the atlas; empty is its initial.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * A line about it.
   *
   * @default ""
   */
  readonly description?: string;
}

/**
 * An inventory: a grid of slots, items dragged between them, a detail panel with Use.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface InventoryDesc {
  /**
   * Its title.
   *
   * @default "Inventory"
   */
  readonly title?: string;
  /**
   * How many slots.
   *
   * @default 24
   */
  readonly slots?: number;
  /**
   * Slots across; 0 fits the width.
   *
   * @default 0
   */
  readonly columns?: number;
  /**
   * The items, by slot.
   *
   * @default []
   */
  readonly items?: readonly InventoryItem[];
  /**
   * The detail panel's action; empty hides it. Every word of the inventory is a field with its English default (title too): a localised game passes its string table's (og.i18n.t).
   *
   * @default "Use"
   */
  readonly useLabel?: string;
  /**
   * The detail panel while no slot is selected.
   *
   * @default "Select an item. Drag it to another slot to move it."
   */
  readonly hintLabel?: string;
  /**
   * The detail panel's count line, beside a stacked item's count.
   *
   * @default "Count"
   */
  readonly countLabel?: string;
}

/** An item dragged from one slot to another. */
export interface SlotMove {
  /** The slot it left. */
  readonly from: number;
  /** The slot it was dropped on (swap when that slot holds an item). */
  readonly to: number;
}

/** An inventory's answer this tick. */
export interface InventoryResult {
  /** The selected slot. */
  readonly selected: number | null;
  /** A drag and drop this tick: move (or swap) the items. */
  readonly moved: SlotMove | null;
  /** The id of the item Use was pressed for. */
  readonly used: string | null;
  /** Close was pressed (or Back). */
  readonly closed: boolean;
}

/**
 * A level in a level select.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LevelInfo {
  /**
   * Its name or number.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Stars earned, 0 to 3.
   *
   * @default 0
   */
  readonly stars?: number;
  /**
   * Not yet reachable: a lock, inert.
   *
   * @default false
   */
  readonly locked?: boolean;
  /**
   * The best result as shown; empty is none.
   *
   * @default ""
   */
  readonly best?: string;
}

/**
 * A level select: a grid of level cards with stars and locks, the total stars, and a Back button. Back closes it from every input: the button, a press on the scrim around it, Escape and the pad's B.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LevelSelectDesc {
  /**
   * Its title. Every word of the screen is a field with its English default: a localised game passes its string table's (og.i18n.t).
   *
   * @default "Levels"
   */
  readonly title?: string;
  /**
   * The levels.
   *
   * @default []
   */
  readonly levels?: readonly LevelInfo[];
  /**
   * Cards across; 0 fits the width.
   *
   * @default 0
   */
  readonly columns?: number;
  /**
   * The button that closes it.
   *
   * @default "Back"
   */
  readonly backLabel?: string;
  /**
   * What a screen reader says for a card's stars (and the total's): `{stars}` is replaced by the stars earned, `{max}` by the most there are. The stars themselves are drawn, never this text.
   *
   * @default "{stars} of {max} stars"
   */
  readonly starsLabel?: string;
  /**
   * What a screen reader calls a locked card (it shows only a lock): `{name}` is replaced by the level's name or number.
   *
   * @default "{name}, locked"
   */
  readonly lockedLabel?: string;
}

/** A level select's answer this tick. */
export interface LevelSelectResult {
  /** The level picked this tick. */
  readonly chosen: number | null;
  /**
   * The player asked to close it: its Back button, a press on the scrim, Escape or the pad's B.
   */
  readonly closed: boolean;
}

/**
 * A labelled value on a results screen.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface StatRow {
  /**
   * The label.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * The value, as shown.
   *
   * @default ""
   */
  readonly value?: string;
}

/**
 * How a results screen shows its score and best.
 * - `"points"`: A whole number grouped with commas (12,840), counted up from 0; the best is the higher.
 * - `"time"`: Seconds as m:ss.mmm (1:36.233), counted up from 0:00.000: a time trial's result. Lower is better: the best is the lower, and a best of 0 is none (no best line).
 */
export type ScoreFormat = "points" | "time";

/**
 * A results screen: the title, stars that land one by one, the score counting up (points, or a time with `format` time) under its label, the best (a badge for a new best), stats, actions and a share hook.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ResultsDesc {
  /**
   * Whether it is shown.
   *
   * @default false
   */
  readonly open?: boolean;
  /**
   * Its title.
   *
   * @default "Level complete"
   */
  readonly title?: string;
  /**
   * The score: points, or seconds with `format` time.
   *
   * @default 0
   */
  readonly score?: number;
  /**
   * The best score: points, or seconds with `format` time (0 is none).
   *
   * @default 0
   */
  readonly best?: number;
  /**
   * How the score and best are shown: points (the default) or a time.
   *
   * @default "points"
   */
  readonly format?: ScoreFormat;
  /**
   * A caption over the score (`Total time`, `Score`); empty is none. A localised game passes its string table's.
   *
   * @default ""
   */
  readonly scoreLabel?: string;
  /**
   * This run set the best.
   *
   * @default false
   */
  readonly newBest?: boolean;
  /**
   * Stars earned.
   *
   * @default 0
   */
  readonly stars?: number;
  /**
   * Stars possible; 0 hides them.
   *
   * @default 3
   */
  readonly maxStars?: number;
  /**
   * Stats under the score.
   *
   * @default []
   */
  readonly stats?: readonly StatRow[];
  /**
   * The actions; empty is Retry, Next and Menu (`retry`, `next`, `menu`).
   *
   * @default []
   */
  readonly actions?: readonly MenuItem[];
  /**
   * Text a share would post; empty hides Share.
   *
   * @default ""
   */
  readonly shareText?: string;
  /**
   * Called with the share text when Share is pressed (the shell posts it).
   *
   * @default null
   */
  readonly onShare?: UiAction | null;
  /**
   * The best score's line; `{best}` is replaced by the best score, formatted as the score is (m:ss.mmm with `format` time). A localised game passes its string table's (og.i18n.t), with `{best}` in it.
   *
   * @default "Best {best}"
   */
  readonly bestLabel?: string;
  /**
   * The badge a new best shows.
   *
   * @default "New best"
   */
  readonly newBestLabel?: string;
  /**
   * The Share button's label.
   *
   * @default "Share"
   */
  readonly shareLabel?: string;
}

/**
 * A row of a leaderboard.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LeaderboardEntry {
  /**
   * The rank.
   *
   * @default 0
   */
  readonly rank?: number;
  /**
   * The player's name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * The score, as shown.
   *
   * @default ""
   */
  readonly score?: string;
  /**
   * This is the player: highlighted.
   *
   * @default false
   */
  readonly player?: boolean;
}

/**
 * A leaderboard: scope tabs, a virtualised list of ranks (medal colours for 1 to 3, the player highlighted), and honest loading, error and empty states.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LeaderboardDesc {
  /**
   * Its title.
   *
   * @default "Leaderboard"
   */
  readonly title?: string;
  /**
   * Scope tabs (Today, This week, All time); empty is none.
   *
   * @default []
   */
  readonly scopes?: readonly string[];
  /**
   * The rows.
   *
   * @default []
   */
  readonly entries?: readonly LeaderboardEntry[];
  /**
   * The rows are loading: skeletons.
   *
   * @default false
   */
  readonly loading?: boolean;
  /**
   * Loading failed: this message and Retry.
   *
   * @default ""
   */
  readonly error?: string;
  /**
   * Shown when there are no rows.
   *
   * @default "No scores yet"
   */
  readonly empty?: string;
  /**
   * The button under an error. Every word of the leaderboard is a field with its English default (title and empty too; error and scopes are the game's): a localised game passes its string table's (og.i18n.t).
   *
   * @default "Retry"
   */
  readonly retryLabel?: string;
}

/** A leaderboard's answer this tick. */
export interface LeaderboardResult {
  /** The scope tab chosen: keep it and load its rows. */
  readonly scope: number;
  /** The scope changed this tick. */
  readonly changed: boolean;
  /** Retry was pressed. */
  readonly retry: boolean;
  /** Close was pressed (or Back). */
  readonly closed: boolean;
}

/**
 * A step of a tutorial: the element to point at, a title and a line.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TutorialStep {
  /**
   * The element's id or path (as touch_move_to takes); empty points at nothing.
   *
   * @default ""
   */
  readonly target?: string;
  /**
   * The step's title.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * What to do.
   *
   * @default ""
   */
  readonly text?: string;
}

/**
 * An onboarding overlay: a spotlight on each step's target, a callout beside it with Next and Skip, and the step count.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TutorialDesc {
  /**
   * Whether it is shown.
   *
   * @default false
   */
  readonly open?: boolean;
  /**
   * The steps.
   *
   * @default []
   */
  readonly steps?: readonly TutorialStep[];
  /**
   * The step shown, from game state.
   *
   * @default 0
   */
  readonly index?: number;
  /**
   * The Skip button. Every word of the overlay is a field with its English default: a localised game passes its string table's (og.i18n.t).
   *
   * @default "Skip"
   */
  readonly skipLabel?: string;
  /**
   * The button that shows the next step.
   *
   * @default "Next"
   */
  readonly nextLabel?: string;
  /**
   * That button on the last step.
   *
   * @default "Done"
   */
  readonly doneLabel?: string;
  /**
   * The step count; `{step}` is replaced by the step's number from 1 and `{count}` by the number of steps. A localised game passes its string table's, with both in it.
   *
   * @default "{step} of {count}"
   */
  readonly stepLabel?: string;
}

/** A tutorial's answer this tick. */
export interface TutorialResult {
  /** The step to show next tick: keep it. */
  readonly index: number;
  /** The last step's Done was pressed. */
  readonly finished: boolean;
  /** Skip was pressed (or Back). */
  readonly skipped: boolean;
}

/**
 * A loading screen: the title, a real progress bar (or a spinner while unknown) and tips that change by the tick.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LoadingDesc {
  /**
   * What is loading.
   *
   * @default "Loading"
   */
  readonly title?: string;
  /**
   * How far, 0 to 1; null is unknown (a spinner, never a fake bar).
   *
   * @default null
   */
  readonly progress?: number | null;
  /**
   * Tips shown in turn.
   *
   * @default []
   */
  readonly tips?: readonly string[];
  /**
   * Seconds per tip.
   *
   * @default 4
   * @range 1..
   */
  readonly tipSeconds?: number;
}

/**
 * A confirm dialog: a title, a consequence sentence, Cancel and the action.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ConfirmDesc {
  /**
   * Whether it is open.
   *
   * @default false
   */
  readonly open?: boolean;
  /**
   * The question.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * What will happen, as rich text markup (og.ui.richText: [b], [color=…], [icon=…]…), so a translation carries its own emphasis.
   *
   * @default ""
   */
  readonly message?: string;
  /**
   * The action's label.
   *
   * @default "Confirm"
   */
  readonly confirmLabel?: string;
  /**
   * Cancel's label.
   *
   * @default "Cancel"
   */
  readonly cancelLabel?: string;
  /**
   * The action destroys something: the danger button.
   *
   * @default false
   */
  readonly danger?: boolean;
}

/** A confirm dialog's answer this tick. */
export interface ConfirmResult {
  /** The action was chosen. */
  readonly confirmed: boolean;
  /** Cancel, Back or the backdrop. */
  readonly cancelled: boolean;
}

/**
 * An alert: a title, a message and one button.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface AlertDesc {
  /**
   * Whether it is open.
   *
   * @default false
   */
  readonly open?: boolean;
  /**
   * The title.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * The message, as rich text markup (og.ui.richText).
   *
   * @default ""
   */
  readonly message?: string;
  /**
   * The button's label.
   *
   * @default "OK"
   */
  readonly okLabel?: string;
}



















// ── systems ─────────────────────────────────────────────────────
// Systems the engine ships as TypeScript: game-level code, listed explicitly in GameDesc.systems like the game's own, so the schedule (and dump_world) shows them and nothing runs that the game did not ask for.

// ── commands ────────────────────────────────────────────────────
// The command channel: actions scheduled on simulation time, from `og run --commands "t:action[:args];..."` or the probe's commands().
// Core actions ship with the engine; a module or a game registers more. Unknown actions warn and continue. Not in publish builds.
// Outputs are relative to the output directory: an absolute path, a `..` component or a name starting with `.` (kept for the harness, such as a playtest's kept store) is refused. A file an earlier step of the script wrote is never overwritten: the later step's is written as `<stem>-2.<ext>` with a warning naming both steps.

/**
 * The actions the engine ships. Arguments follow the action, separated by `:`.
 * - `"screenshot"`: `screenshot[:path][:full|:N]`: capture the next frame. Without a renderer it writes nothing and logs, not as a warning, that the frame is unchecked.
 * - `"listen"`: `listen[:path][:seconds]`: write the last seconds of rendered audio as a WAV.
 * - `"touch_move_to"`: `touch_move_to:<ui id>`: move the pointer to a UI element.
 * - `"touch_move"`: `touch_move:x,y`: move the pointer to CSS pixel coordinates.
 * - `"touch_start"`: `touch_start`: press the pointer where it is.
 * - `"touch_end"`: `touch_end`: release the pointer.
 * - `"scroll"`: `scroll:dx,dy`: scroll at the pointer.
 * - `"touch"`: `touch:<id>:<down|move|up>:<x,y | ui id>`: a further touch (pointer 1 to 4) held while the pointer is busy elsewhere.
 * - `"gamepad"`: `gamepad:<a+b+...|->[@pad][:lx,ly[,rx,ry]][:lt,rt]`: set a pad's whole state (pad 0 unless `@1` to `@3` follows the buttons); `gamepad:off` disconnects it.
 * - `"key_down"`: `key_down:[Mod+]<Key>`: hold a key by its code, with the modifiers named before it (`Shift+ArrowLeft`, `Meta+ArrowLeft` for Command, apart from Ctrl).
 * - `"key_up"`: `key_up:[Mod+]<Key>`: release a key, then the modifiers named before it.
 * - `"type"`: `type:<text>`: type text, two ticks per character: its key press and the character as typed text (input.TextInput); a character no key types is typed text alone. The text runs to the `;` exactly, a trailing space included; after it a text field takes typed text only, so a bare key_down types nothing.
 * - `"compose"`: `compose:<text>`: an input method's composition so far (input.TextInput composing); a focused text field shows it underlined at the caret, `compose:` with nothing ends it.
 * - `"commit"`: `commit:<text>`: an input method commits text; a focused text field inserts it and the composition ends.
 * - `"move_to_text"`: `move_to_text:[=]text[@N]`: move the pointer to the UI element showing the text (exactly with `=`, the Nth with `@N`).
 * - `"teleport"`: `teleport:<entity>:x,y,z`: set an entity's position.
 * - `"frame_entity"`: `frame_entity:<entity>`: point the camera at an entity.
 * - `"dump_world"`: `dump_world[:path]`: tables, systems and their matched counts.
 * - `"dump_entity"`: `dump_entity:<name>[:path]`: one entity's components and transform.
 * - `"dump_ui"`: `dump_ui[:path]`: the UI tree with bounds, and `style`, the configured UI style's name (empty for the placeholder prototype).
 * - `"set"`: `set:<entity>.<component>.<field>=<value>` or `set:<Resource>.<field>=<value>`: a Tune write through the schema: a number, `true` or `false` for a bool, `x,y,z` for a vector, an enum's variant name or its index; refused outside the field's FieldOptions min/max, never clamped.
 * - `"get"`: `get:<Resource>[.<field>]` or `get:<entity>[.<component>[.<field>]][:path]`: read through the schema as game code sees it (an enum as its variant's name, `"music"`; og.Time whole, with rate and the time scale; the DirectionalLight og.DayNight drives with the colour, intensity and direction it is drawn with), logged and written as JSON to `get-<tick>.json` (more unnamed ones on one tick: `get-<tick>-2.json`, ...).
 * - `"seed"`: `seed:<n>`: reseed the Rng resource.
 * - `"pause"`: `pause`: hold the clock; advance still steps.
 * - `"resume"`: `resume`: release the clock.
 * - `"advance"`: `advance:<ticks>`: take fixed steps.
 * - `"quit"`: `quit`: exit.
 * - `"describe"`: `describe[:path]`: write the frame as data (probe.Description): the visible entities with screen boxes, depth order and occlusion from the entity-id pass, what each secondary camera (og.CameraView, og.Mirror) shows from its own pass, the UI elements with rects and text, the camera.
 * - `"expect"`: `expect:<what>`: assert at this tick (probe.ExpectResult): `entity:<id>:within:x,y,w,h` (its visible box, CSS pixels) or `:within:x0,y0,z0,x1,y1,z1` (its position, world units), `event:<Name>[:count]`, `ui:text:[=]<text>`, `region:x,y,w,h:color:#rrggbb[:tolerance]`, `frame:<ref.png>:rmse:<=v`, `state:<path>==<value>` (also != < <= > >=), `entity:<id>:<path>==<value>` (a field of its dump_entity: `render.lod==2`, `og.Transform.position.y>0`), `timings:<path>==<value>` (state().timings: `triangles<=1500000`). A failure is a warning with the tick and the observed value; `og run --json` lists every result and exits 1 on a failure. Frame expectations without a renderer are unchecked, as are an entity's `render.*` fields and the renderer's timings (`triangles`, `render`, `passes`).
 * - `"save_scene"`: `save_scene[:path]`: write the boot scene's `.ogt` back from the world, only the lines whose values changed (comments and order kept) and the fields set or patch tuned; to the scene's source, or with a path to that file under the output directory.
 * - `"help"`: `help`: list every action the channel knows, core and registered.
 */
export type CoreAction = "screenshot" | "listen" | "touch_move_to" | "touch_move" | "touch_start" | "touch_end" | "scroll" | "touch" | "gamepad" | "key_down" | "key_up" | "type" | "compose" | "commit" | "move_to_text" | "teleport" | "frame_entity" | "dump_world" | "dump_entity" | "dump_ui" | "set" | "get" | "seed" | "pause" | "resume" | "advance" | "quit" | "describe" | "expect" | "save_scene" | "help";

/**
 * A registered action. It runs at the start of the tick it was scheduled for.
 *
 * @param world The world.
 * @param args The `:`-separated arguments after the action name.
 */
export type CommandHandler = (world: World, args: readonly string[]) => void;

/**
 * How a registered action describes itself in `og run --help` and the wiki.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CommandDesc {
  /**
   * The argument form, such as `<count>`.
   *
   * @default ""
   */
  readonly usage?: string;
  /**
   * One sentence on what it does.
   *
   * @default ""
   */
  readonly doc?: string;
}

/** One parsed step of a command script. */
export interface CommandStep {
  /**
   * When it runs, in simulation seconds from the start of the script.
   *
   * @unit s (seconds of simulation time)
   */
  readonly at: number;
  /** The action name. */
  readonly action: string;
  /** The arguments. */
  readonly args: readonly string[];
}

/** One action as help lists it. */
export interface CommandInfo {
  /** The action name. */
  readonly name: string;
  /** Its argument form, such as `screenshot[:path][:full|:N]`. */
  readonly usage: string;
  /** One sentence on what it does. */
  readonly doc: string;
  /** Shipped with the engine (true) or registered by a module or a game. */
  readonly core: boolean;
}

/** What scheduling a script did. */
export interface CommandsReport {
  /** Steps scheduled. */
  readonly scheduled: number;
  /** The tick of the last step. */
  readonly untilTick: number;
  /** Unknown actions and other skipped steps, each naming the fix. */
  readonly warnings: readonly string[];
}





// ── probe ───────────────────────────────────────────────────────
// The probe: RuntimeProbe protocol 5 (origin/packages/contracts/src/probe.ts) served natively, plus the engine's wire members.
// In a page it is published on globalThis.__runtime_probe__; on native, `og` serves it as JSON-RPC 2.0 over a loopback WebSocket at /probe.
// A refusal is false or null exactly as protocol 5; a JSON-RPC error is only for a malformed call or an unknown method.
// Probe entity ids are entity names; an unnamed entity is `#<index>.<generation>`.

/** The global the page publishes the probe on. */
export declare const PROBE_GLOBAL: "__runtime_probe__";

/** The RuntimeProbe contract revision both sides speak. */
export declare const PROBE_VERSION: 5;

/** The wire revision hello reports. */
export declare const PROBE_WIRE_VERSION: 1;

/** The page query parameter that starts a deterministic replay from a uint32 seed. */
export declare const REPLAY_SEED_PARAM: "replay-seed";

/** The longest playtest one request may advance, in fixed steps. */
export declare const PLAYTEST_MAX_TICKS: 3600;

/** The most operations one probe program may carry. */
export declare const PROBE_PROGRAM_MAX_OPERATIONS: 256;

/** What a screen point resolved to. */
export interface PickResult {
  /** The probe id of the entity under the point. */
  readonly entity: string;
  /**
   * Where the pick landed in world space: two components in the 2D profile, three in 3D; nothing pads a plane with an unmeasured zero.
   */
  readonly world: [number, number] | [number, number, number];
  /** Where a person would change it, as `player.ts::jump`. */
  readonly source: string | null;
}

/** One published tunable parameter and the range it means something over. */
export interface ProbeTunable {
  /** Where the value lives, as `Health.hp`. */
  readonly path: string;
  /** The person-facing name. */
  readonly label?: string;
  /** The smallest meaningful value. */
  readonly min?: number;
  /** The largest meaningful value. */
  readonly max?: number;
  /** Tune's increment. */
  readonly step?: number;
}

/** One entity the probe publishes in state(). */
export interface ProbeEntityRef {
  /** Its probe id. */
  readonly id: string;
  /** A person-facing name. */
  readonly label?: string;
}

/** The fixed timings head of state(). */
export interface ProbeTimings {
  /**
   * Time the last tick took.
   *
   * @unit ms (milliseconds)
   */
  readonly step: number;
  /**
   * Time the last frame's render took; 0 without a renderer.
   *
   * @unit ms (milliseconds)
   */
  readonly render: number;
  /**
   * step + render.
   *
   * @unit ms (milliseconds)
   */
  readonly frameMs: number;
  /** The last tick's systems in run order, each with its milliseconds. */
  readonly systems: readonly ProbeTiming[];
  /**
   * The last rendered frame's render passes in the render graph's order, each with its milliseconds (CPU encode time; empty without a renderer).
   */
  readonly passes: readonly ProbeTiming[];
  /**
   * Triangles the last frame's main pass drew (after culling and level of detail); 0 without a renderer.
   */
  readonly triangles: number;
}

/** One named span of the last tick or frame. */
export interface ProbeTiming {
  /** A system's name, or a render pass's (`Main`, `Cascade0`, `Bloom`). */
  readonly name: string;
  /**
   * Its time.
   *
   * @unit ms (milliseconds)
   */
  readonly ms: number;
}

/** The probe's state(): a fixed head, then the game's own facts. */
export interface ProbeState {
  /** Fixed steps since load. */
  readonly tick: number;
  /** Whether the clock is held. */
  readonly paused: boolean;
  /** The seed the world was assembled from. */
  readonly seed: number;
  /** The last tick's timings. */
  readonly timings: ProbeTimings;
  /** The named entities. */
  readonly entities: readonly ProbeEntityRef[];
  /** What GameDesc.facts returned, or null. */
  readonly facts: unknown;
}

/** The advance operation's payload. */
export interface AdvanceOperation {
  /** Fixed steps to take. */
  readonly steps: number;
}

/** The input operation's payload. */
export interface InputOperation {
  /** The event. */
  readonly event: InjectedInput;
}

/** The pick operation's payload. */
export interface PickOperation {
  /** Horizontal position in CSS pixels. */
  readonly x: number;
  /** Vertical position in CSS pixels. */
  readonly y: number;
}

/** The entity operation's payload. */
export interface EntityOperation {
  /** The probe id. */
  readonly id: string;
}

/** The expect operation's payload. */
export interface ExpectOperation {
  /** The expectation, as after `expect:` in a script, such as `state:started==true`. */
  readonly what: string;
}

/**
 * One operation of a probe program, which runs in one evaluation so a scripted segment costs one round trip.
 */
export type ProbeOperation =
  /** Take fixed steps; answers the tick. */
  | ({ readonly op: "advance" } & AdvanceOperation)
  /** Inject an event; answers true. */
  | ({ readonly op: "input" } & InputOperation)
  /** Hold the clock; answers true. */
  | { readonly op: "pause" }
  /** Release the clock; answers true. */
  | { readonly op: "resume" }
  /** Read the state; answers `{ tick, state }`. */
  | { readonly op: "read" }
  /** Pick a point; answers its result or null. */
  | ({ readonly op: "pick" } & PickOperation)
  /** Read an entity's parameters; null is a refusal. */
  | ({ readonly op: "entity" } & EntityOperation)
  /** Evaluate an expectation; answers its ExpectResult, failed or not (a malformed one is a refusal). */
  | ({ readonly op: "expect" } & ExpectOperation)
  /** Describe the frame; answers its Description. */
  | { readonly op: "describe" };

/**
 * What a program observed: one answer per operation that ran. A false, or entity's null, stops the program and `refused` names its index.
 */
export interface ProbeProgramResult {
  /** The answers, in order. */
  readonly results: readonly unknown[];
  /** The index of the refusing operation, or null. */
  readonly refused: number | null;
  /** The tick after the last operation that ran. */
  readonly tick: number;
}

/** The wire's hello: who is speaking. */
export interface Hello {
  /** PROBE_VERSION. */
  readonly protocol: number;
  /** PROBE_WIRE_VERSION. */
  readonly wire: number;
  /** Always `origin-engine`. */
  readonly substrate: string;
  /** `origin-engine <version>`. */
  readonly engine: string;
  /**
   * The fixed tick rate.
   *
   * @unit hz (ticks or cycles per second)
   */
  readonly ticksPerSecond: number;
  /** The renderer backend. */
  readonly tier: RenderTier;
  /** The build configuration. */
  readonly build: BuildProfile;
  /** The probe members this build serves. */
  readonly members: readonly string[];
  /**
   * The engine modules this build carries beyond the core (`physics`, `audio`, `ui`, `cook`), so a client knows which members do something.
   */
  readonly modules: readonly string[];
}

/**
 * The build configuration; publish builds keep only the player-facing probe members.
 * - `"dev"`: A development build: every member, the command channel, dumps.
 * - `"release"`: An optimised build with every member.
 * - `"publish"`: A published game: hello, tick, state, pick, entity, input and capture only.
 */
export type BuildProfile = "dev" | "release" | "publish";

/** A captured frame. */
export interface Capture {
  /** The tick the frame shows. */
  readonly tick: number;
  /** Width in pixels. */
  readonly width: number;
  /** Height in pixels. */
  readonly height: number;
  /** The PNG, base64. */
  readonly png: string;
}

/** What the host's mixer rendered last: the audio up to the current tick. */
export interface Heard {
  /** The tick it ends at. */
  readonly tick: number;
  /**
   * Frames per second.
   *
   * @unit hz (ticks or cycles per second)
   */
  readonly rate: number;
  /** Stereo frames in the WAV. */
  readonly frames: number;
  /** The largest sample, 0 to 1. */
  readonly peak: number;
  /** The root mean square of both channels. */
  readonly rms: number;
  /** A 16-bit stereo WAV, base64. */
  readonly wav: string;
}

/** The engine's allocator, counted (both hosts install a counting global allocator). */
export interface ProbeHeap {
  /**
   * Bytes allocated and not freed.
   *
   * @unit bytes (bytes)
   */
  readonly live: number;
  /**
   * The most bytes live at once since load.
   *
   * @unit bytes (bytes)
   */
  readonly peak: number;
  /** Allocations made since load. */
  readonly allocations: number;
}

/** The world's size. */
export interface ProbeWorldMemory {
  /** Entities alive. */
  readonly entities: number;
}

/** The streaming memory budget (og.stream.budget): what the admitted groups' assets take. */
export interface ProbeStreamMemory {
  /**
   * The budget.
   *
   * @unit bytes (bytes)
   */
  readonly budget: number;
  /**
   * What the admitted groups take now.
   *
   * @unit bytes (bytes)
   */
  readonly used: number;
  /**
   * The most they took at once.
   *
   * @unit bytes (bytes)
   */
  readonly peak: number;
}

/**
 * What the engine holds, in one shape on both hosts; a figure a host cannot read is null. A publish page, whose probe has no `memory`, answers the same from the engine Worker (`__runtime_probe__.memory()`, docs/web-export.md).
 */
export interface ProbeMemory {
  /** `web` or `native`. */
  readonly host: string;
  /**
   * The wasm linear memory's size, which the browser reserved for the engine and never takes back; null natively.
   *
   * @unit bytes (bytes)
   */
  readonly linear: number | null;
  /**
   * The process's resident memory (Linux /proc/self/status, macOS task_info); null on the web, where a page cannot read its process's.
   *
   * @unit bytes (bytes)
   */
  readonly resident: number | null;
  /** The engine's allocator; null where the host counts nothing. */
  readonly heap: ProbeHeap | null;
  /** The world. */
  readonly world: ProbeWorldMemory;
  /** The streaming budget. */
  readonly stream: ProbeStreamMemory | null;
  /**
   * The sound bank's samples in memory (decoded sounds and the resident chunks of streamed ones); null without audio.
   *
   * @unit bytes (bytes)
   */
  readonly audio: number | null;
}

/**
 * A parameter sweep: for each value, restore the snapshot taken at the call, patch, advance and capture.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SweepDesc {
  /**
   * The probe id of the entity.
   *
   * @default ""
   */
  readonly id?: string;
  /**
   * The parameter, as `Health.hp`.
   *
   * @default ""
   */
  readonly parameter?: string;
  /**
   * The values to try, in order.
   *
   * @default []
   */
  readonly values?: readonly unknown[];
  /**
   * Fixed steps to advance after each patch.
   *
   * @default 0
   */
  readonly steps?: number;
  /**
   * Capture width; 0 is the render target's.
   *
   * @default 0
   */
  readonly width?: number;
  /**
   * Capture height; 0 is the render target's.
   *
   * @default 0
   */
  readonly height?: number;
}

/** One frame of a sweep. */
export interface SweepFrame {
  /** The value patched in. */
  readonly value: unknown;
  /** The tick captured. */
  readonly tick: number;
  /** The PNG, base64. */
  readonly png: string;
}

/** A sweep's frames, one per value. */
export interface SweepResult {
  /** The frames. */
  readonly frames: readonly SweepFrame[];
}

/**
 * What a hot reload did. Components are re-registered by name and migrated field by field through the schema.
 */
export interface ReloadReport {
  /** The tick the world was restored at. */
  readonly tick: number;
  /**
   * From bundle swap to restored world.
   *
   * @unit ms (milliseconds)
   */
  readonly ms: number;
  /** Fields carried across a layout change, as `Health.hp`. */
  readonly migrated: readonly string[];
  /** Components or fields that no longer exist, dropped with their data. */
  readonly dropped: readonly string[];
  /** Anything else worth reading, each naming the fix. */
  readonly warnings: readonly string[];
}

/**
 * A snapshot of the whole world (components, resources, Rng states, pending events) kept in the running process.
 */
export interface SnapshotInfo {
  /** The snapshot's id, for restore. */
  readonly id: string;
  /** The tick it was taken at. */
  readonly tick: number;
  /** The world hash at that tick, hex: equal hashes mean equal worlds. */
  readonly hash: string;
  /**
   * Its size.
   *
   * @unit bytes (bytes)
   */
  readonly bytes: number;
}

/**
 * What dump() reports.
 * - `"world"`: Archetype tables, systems and their matched counts.
 * - `"entity"`: One entity's components and transform.
 * - `"ui"`: The UI tree with bounds.
 * - `"desync"`: The first desync of a lockstep session (og.net): its tick, both hashes and the component-level diff of the two peers' snapshots.
 */
export type DumpKind = "world" | "entity" | "ui" | "desync";

/**
 * What to dump.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DumpDesc {
  /**
   * The kind of dump.
   *
   * @default "world"
   */
  readonly what?: DumpKind;
  /**
   * The probe id, for an entity dump.
   *
   * @default ""
   */
  readonly id?: string;
}

/** One archetype table. */
export interface ArchetypeDump {
  /** Its components, by name. */
  readonly components: readonly string[];
  /** Entities in it. */
  readonly count: number;
}

/**
 * One system. The engine fills reads, writes and matched for every system the game declared; ms is null because the probe reads no clock (state().timings has the tick's time).
 */
export interface SystemDump {
  /** Its name. */
  readonly name: string;
  /**
   * Entities its queries match now; null for a system the host did not declare to the probe.
   */
  readonly matched: number | null;
  /** What it declared it reads; null when undeclared. */
  readonly reads: readonly string[] | null;
  /** What it declared it writes; null when undeclared. */
  readonly writes: readonly string[] | null;
  /**
   * Time it took on the last tick; null when not measured.
   *
   * @unit ms (milliseconds)
   */
  readonly ms: number | null;
}

/** The world: tables, systems, resources. */
export interface WorldDump {
  /** The tick. */
  readonly tick: number;
  /** The world hash, hex. */
  readonly hash: string;
  /** Live entities. */
  readonly entities: number;
  /** The archetype tables. */
  readonly archetypes: readonly ArchetypeDump[];
  /** The systems in run order. */
  readonly systems: readonly SystemDump[];
  /** The registered resources. */
  readonly resources: readonly string[];
}

/** One entity. */
export interface EntityDump {
  /** Its probe id. */
  readonly id: string;
  /** Its name. */
  readonly name: string | null;
  /** Each component's field values, by component name. */
  readonly components: Readonly<Record<string, unknown>>;
}

/** One UI element with its bounds. */
export interface UiElement {
  /** Its stable id, as the command channel targets it. */
  readonly id: string;
  /** Its parent's id; empty at the root. */
  readonly parent: string;
  /**
   * What made it: `label`, `button`, `icon`, `image`, `panel` and the other widget names; empty when the UI module does not say.
   */
  readonly kind: string;
  /**
   * A disabled control (its text is exempt from contrast rules, as WCAG exempts inactive controls).
   */
  readonly disabled: boolean;
  /**
   * Only part of it shows: a scroll container or the surface cuts it, and the bounds are the part that shows.
   */
  readonly clipped: boolean;
  /** The text it shows, if any. */
  readonly text: string;
  /**
   * Its accessibility role (og.ui.a11y): button, link, text, heading, image, checkbox, radio, switch, slider, spinbutton, textbox, combobox, listbox, option, tab, tablist, radiogroup, progressbar, meter, timer, status, alert, dialog, tooltip, separator, group; empty for a presentational element (a layout container, a spacer).
   */
  readonly role: string;
  /**
   * Its accessible name: its text, a name the widget gives it (an icon's name, a dialog's title, Decrease and Increase for a stepper's buttons), a control's own text gathered from its children, or else its id.
   */
  readonly name: string;
  /**
   * Its value as text: a slider's or stepper's number, a progress's or meter's amount, a field's text, a select's choice; empty for none.
   */
  readonly value: string;
  /**
   * Its states: disabled, focused, focusable (keys and pads reach it now, in this order), checked, selected, expanded, pressed, busy.
   */
  readonly states: readonly string[];
  /**
   * Left edge.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly x: number;
  /**
   * Top edge.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly y: number;
  /**
   * Width.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly width: number;
  /**
   * Height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly height: number;
}

/** The surface UI bounds are laid out on. */
export interface UiSurface {
  /**
   * Its width.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly width: number;
  /**
   * Its height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly height: number;
  /**
   * Device pixels per CSS pixel: a frame of this surface is width × dpr pixels wide.
   *
   * @default 1
   */
  readonly dpr: number;
}

/** The UI tree, flattened in draw order. */
export interface UiDump {
  /** The tick. */
  readonly tick: number;
  /** The surface the bounds are on; null without a UI module. */
  readonly surface: UiSurface | null;
  /** The elements. */
  readonly elements: readonly UiElement[];
}

/** The first desync of this peer's lockstep session (og.net), as `dump_desync` writes it. */
export interface DesyncDump {
  /** Whether a desync happened; false lists the ticks compared with each peer so far. */
  readonly desync: boolean;
  /** The first divergent tick: og.Time.tick of the two worlds that differ. */
  readonly tick: number | null;
  /** This peer: `{ peer, player, hash }`. */
  readonly local: unknown;
  /** The other peer: `{ peer, player, hash }`. */
  readonly remote: unknown;
  /**
   * What differs: `{ components: [{ entity, component, fields: [{ field, local, remote }] }], entities: { onlyLocal, onlyRemote }, resources, events, blocks, allocator }`, in the local world's entity order (the first entry is the first difference in storage order); `{ pending }` until the other peer's snapshot arrived.
   */
  readonly diff: unknown;
}

/** A dump, tagged by its kind. */
export type DumpResult =
  /** The world dump. */
  | ({ readonly kind: "world" } & WorldDump)
  /** An entity dump. */
  | ({ readonly kind: "entity" } & EntityDump)
  /** The UI dump. */
  | ({ readonly kind: "ui" } & UiDump)
  /** The desync dump. */
  | ({ readonly kind: "desync" } & DesyncDump);

/** A box on the play surface. */
export interface ScreenBox {
  /**
   * Left edge.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly x: number;
  /**
   * Top edge.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly y: number;
  /**
   * Width.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly width: number;
  /**
   * Height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly height: number;
}

/** A point on the play surface. */
export interface ScreenPoint {
  /**
   * Horizontal position.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly x: number;
  /**
   * Vertical position.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly y: number;
}

/**
 * One entity the frame shows, from the entity-id pass (a CPU rasterisation of the frame's meshes, skinned meshes and world sprites at a quarter of its pixels).
 */
export interface DescribedEntity {
  /** Its probe id. */
  readonly id: string;
  /** Its name. */
  readonly name: string | null;
  /** Its depth order: 0 is the nearest. */
  readonly order: number;
  /** Its components, by name. */
  readonly archetype: readonly string[];
  /** What draws it: `model` with its asset path, `mesh`, `skinned`, `sprite`. */
  readonly labels: readonly string[];
  /** The box of its visible pixels. */
  readonly box: ScreenBox;
  /** The box of what it covers with nothing in front of it. */
  readonly drawn: ScreenBox;
  /** A visible point nearest the box's centre: pick answers this entity there. */
  readonly point: ScreenPoint;
  /**
   * From the camera to its nearest visible point, along the view.
   *
   * @unit m (metres)
   */
  readonly depth: number;
  /**
   * Its visible share of the surface.
   *
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly coverage: number;
  /**
   * The share of what it covers alone that something else hides.
   *
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly occlusion: number;
  /**
   * The probe ids of the og.CameraView and og.Mirror entities whose image also shows it, from their own id passes on their own layers; empty when only the main camera does (under DescribedView.entities it is empty).
   */
  readonly views: readonly string[];
}

/**
 * A secondary camera (og.CameraView, og.Mirror) and what its image shows: its own entity-id pass, from its eye and on its layers, at a quarter of its image's pixels, whatever its `every`. An entity only its layers admit (a layer the main camera leaves out) is listed here and not in Description.entities.
 */
export interface DescribedView {
  /** The view entity's probe id. */
  readonly id: string;
  /** Its name. */
  readonly name: string | null;
  /** `camera` (og.CameraView) or `mirror` (og.Mirror). */
  readonly kind: string;
  /** The pixels its image covers: its render target's, or its viewport's rect of the frame. */
  readonly size: PassSize;
  /** What it shows, nearest its eye first, boxes and points in its image's pixels. */
  readonly entities: readonly DescribedEntity[];
}

/** One UI element as describe lists it (the tree dump_ui writes). */
export interface DescribedUi {
  /** Its id. */
  readonly id: string;
  /** Its parent's id. */
  readonly parent: string;
  /** What made it. */
  readonly kind: string;
  /** Its accessibility role. */
  readonly role: string;
  /** The text it shows. */
  readonly text: string;
  /** Its value as text. */
  readonly value: string;
  /** Its states. */
  readonly states: readonly string[];
  /** Its rect. */
  readonly box: ScreenBox;
}

/** The camera as drawn, with its shake. */
export interface DescribedCamera {
  /** Where it is. */
  readonly position: [number, number, number];
  /** What it looks at. */
  readonly target: [number, number, number];
  /** Its up. */
  readonly up: [number, number, number];
  /** `perspective` or `orthographic`. */
  readonly projection: string;
  /**
   * The vertical field of view (perspective).
   *
   * @unit deg (degrees)
   */
  readonly fov: number | null;
  /**
   * The height in view (orthographic).
   *
   * @unit m (metres)
   */
  readonly height: number | null;
  /**
   * The near plane.
   *
   * @unit m (metres)
   */
  readonly near: number;
  /**
   * The far plane.
   *
   * @unit m (metres)
   */
  readonly far: number;
}

/** The entity-id pass's size. */
export interface PassSize {
  /** Width in pixels. */
  readonly width: number;
  /** Height in pixels. */
  readonly height: number;
}

/**
 * The frame as data: deterministic, the same JSON on every host and backend for the same world.
 */
export interface Description {
  /** The tick. */
  readonly tick: number;
  /** The play surface. */
  readonly surface: UiSurface | null;
  /** The entity-id pass; null without a scene. */
  readonly pass: PassSize | null;
  /** The camera; null without a scene. */
  readonly camera: DescribedCamera | null;
  /**
   * The entities the main camera shows, nearest first (each names the views that also show it).
   */
  readonly entities: readonly DescribedEntity[];
  /** The secondary cameras the frame draws, in its view order, each with what it shows. */
  readonly views: readonly DescribedView[];
  /** The UI elements in draw order. */
  readonly ui: readonly DescribedUi[];
}

/**
 * How an expectation came out.
 * - `"pass"`: It held.
 * - `"fail"`: It did not: the result carries the tick and the observed value.
 * - `"unchecked"`: It reads a frame and the host has no renderer.
 */
export type ExpectStatus = "pass" | "fail" | "unchecked";

/** One evaluated expectation. */
export interface ExpectResult {
  /** The tick it was evaluated at. */
  readonly tick: number;
  /**
   * The script step's time, when a script ran it.
   *
   * @unit s (seconds of simulation time)
   */
  readonly at: number | null;
  /** As written after `expect:`. */
  readonly expect: string;
  /** `entity`, `event`, `ui`, `region`, `frame` or `state`. */
  readonly kind: string;
  /** The outcome. */
  readonly status: ExpectStatus;
  /** What was expected. */
  readonly expected: unknown;
  /** What was there. */
  readonly observed: unknown;
  /** Why it failed or was unchecked; empty on a pass. */
  readonly reason: string;
}

/** The expectations evaluated since the last call. */
export interface ExpectReport {
  /** Those that held. */
  readonly passed: number;
  /** Those that did not. */
  readonly failed: number;
  /** Those that needed a renderer. */
  readonly unchecked: number;
  /** Each, in order. */
  readonly results: readonly ExpectResult[];
}

/** A scene written back from the world. */
export interface SavedScene {
  /** The tick of the world it was written from. */
  readonly tick: number;
  /** The scene's source, project-relative. */
  readonly source: string;
  /** Where it was written; empty when only returned. */
  readonly written: string;
  /** The 1-based lines of the new text that changed or were added. */
  readonly lines: readonly number[];
  /**
   * Tuned fields that were not written, each with its reason: a resource, or an entity the scene did not make.
   */
  readonly skipped: readonly string[];
  /** The new text. */
  readonly text: string;
}

/**
 * RuntimeProbe protocol 5, member for member. Every member answers synchronously with JSON-serialisable values.
 */
export interface RuntimeProbe {
  /** PROBE_VERSION. */
  readonly version: number;
  /** Fixed steps the world has taken since load. */
  tick(): number;
  /** A JSON snapshot: the fixed timings head, then the game's own facts. */
  state(): ProbeState;
  /**
   * Take whole fixed steps regardless of the wall clock.
   *
   * @param steps Fixed steps, at most PLAYTEST_MAX_TICKS.
   * @returns The tick after the steps.
   */
  advance(steps: number): number;
  /** Hold the wall-clock loop; advance still steps. */
  pause(): boolean;
  /** Release the wall-clock loop. */
  resume(): boolean;
  /**
   * One event through the input layer's own path.
   *
   * @param event The event.
   * @returns False is a refusal.
   */
  input(event: InjectedInput): boolean;
  /** Page events (keyboard, pointer, wheel) the input layer accepted since load. */
  accepted(): number;
  /**
   * What is under a point of the play surface.
   *
   * @param x Horizontal position in CSS pixels.
   * @param y Vertical position in CSS pixels.
   * @returns Null when nothing is under the point.
   */
  pick(x: number, y: number): PickResult | null;
  /**
   * One entity's published parameters: its tunable fields as `Component.field` keys.
   *
   * @param id The probe id.
   * @returns Null for an unknown entity.
   */
  entity(id: string): Readonly<Record<string, unknown>> | null;
  /**
   * The ranges of one entity's tunable fields, from the schema.
   *
   * @param id The probe id.
   * @returns Null for an unknown entity.
   */
  bounds(id: string): Readonly<Record<string, ProbeTunable>> | null;
  /**
   * Move one tunable field for this run only; it applies within one frame. The probe never writes project files.
   *
   * @param id The probe id.
   * @param parameter The field, as `Health.hp`.
   * @param value The new value.
   * @returns False is a refusal.
   */
  patch(id: string, parameter: string, value: unknown): boolean;
  /**
   * Run a program of the members above in order, stopping at the first refusal.
   *
   * @param program At most PROBE_PROGRAM_MAX_OPERATIONS operations.
   */
  run(program: readonly ProbeOperation[]): ProbeProgramResult;
}

/**
 * The engine's probe: protocol 5 plus the wire members of the loopback WebSocket and the in-page request ring.
 */
export interface EngineProbe extends RuntimeProbe {
  /** Who is speaking: protocol, wire, engine version, tick rate, renderer tier, build. */
  hello(): Hello;
  /**
   * The frame of the current tick, rendered offscreen and read back as a PNG; null without a renderer (hello's tier `none`).
   * Size rule: width and height are both 0 (or absent), which is the render target's size, or both 1 to 4096, which renders that size with the same camera (the aspect follows the requested size); one zero and one not is a malformed call.
   * On the web the readback is asynchronous: through the request ring the answer arrives when the copy completes; a synchronous in-page call answers null on the first tick and the frame from then on (a capture is requested on tick N and answered on N + 1).
   *
   * @param width Width in pixels; 0 is the render target's. Default `0`.
   * @param height Height in pixels; 0 is the render target's. Default `0`.
   * @returns Null without a renderer.
   */
  capture(width?: number, height?: number): Capture | null;
  /**
   * A filmstrip over one tunable: per value, restore, patch, advance, capture.
   *
   * @param desc The parameter, values, steps and size.
   * @returns Null for an unknown entity or parameter, or without a renderer.
   */
  sweep(desc?: SweepDesc): SweepResult | null;
  /**
   * Hot reload: snapshot, swap the game bundle, re-register components by name, restore.
   *
   * @returns Null when the new bundle fails to load; the old one keeps running.
   */
  reload(): ReloadReport | null;
  /** Snapshot the whole world in the running process. */
  snapshot(): SnapshotInfo | null;
  /**
   * Restore a snapshot taken by this process.
   *
   * @param id The snapshot id.
   * @returns False for an unknown id.
   */
  restore(id: string): boolean;
  /**
   * A world, entity or UI dump, as the command channel's dump actions write.
   *
   * @param desc What to dump.
   * @returns Null for an unknown entity.
   */
  dump(desc?: DumpDesc): DumpResult | null;
  /**
   * Schedule a command script on simulation time, as `og run --commands`.
   *
   * @param script `t:action[:args];...`.
   * @returns Null for a malformed script.
   */
  commands(script: string): CommandsReport | null;
  /**
   * The last seconds of audio the host's mixer rendered (a null sink renders too, into a two-second ring), as a WAV with its peak and RMS: what a person would have heard.
   *
   * @param seconds How many seconds, 0 to 2. Default `1`.
   * @returns Null without audio, or before anything was rendered.
   */
  listen(seconds?: number): Heard | null;
  /**
   * What the engine holds: the wasm linear memory on the web or the process's resident memory natively, the engine's counted heap, the world's entities, the streaming budget and the audio bank, in the same shape on both hosts.
   *
   * @returns Null from a host that gives no figures.
   */
  memory(): ProbeMemory | null;
  /**
   * The frame of the current tick as data: the visible entities with screen boxes, depth order and occlusion from the entity-id pass, their names, archetypes and labels; the UI elements with rects and text; the camera. Answers without a renderer too (the pass runs on the CPU).
   */
  describe(): Description;
  /**
   * Evaluate one expectation now (the forms of the command channel's `expect:`); the result is also kept for expectations().
   *
   * @param what The expectation, such as `entity:crate:within:0,0.4,-1,1,0.6,1`.
   * @returns A malformed expectation is a malformed call.
   */
  expect(what: string): ExpectResult;
  /**
   * Every expectation evaluated since the last call (scripts' `expect:` steps, programs, expect()), and the counts.
   */
  expectations(): ExpectReport;
  /**
   * Write the boot scene back from the world: its entities' changed field lines and the fields set and patch tuned, keeping every other line, comment and the order. The probe writes the file only with write; otherwise the text is returned.
   *
   * @param write Replace the scene's source file. Default `false`.
   * @returns Null without a scene to write back (the reason is in the log).
   */
  saveScene(write?: boolean): SavedScene | null;
}

// ── terrain ─────────────────────────────────────────────────────
// Terrain (phase 2): a height field over a rectangle, cooked by `og cook` from a `type terrain` .ogt (a 16-bit grey PNG or an EXR heightmap, an optional control map, up to four layered materials with slope and altitude rules, their maps separate textures or layers of two texture arrays: `layers` and `layerNormals` with each layer's `albedoLayer` and `normalLayer`, two sampled textures instead of up to eight) into tiles; drawn as a geometry clipmap around the camera (nested levels by distance, no cracks) that casts and receives shadows under the fog; collided with as a height field; baked into navmeshes by a navmesh file's `terrain` entry.
// Load it with og.terrain.load, then place it: an og.Terrain on an entity draws it placed by the entity's world transform (centred on its position, turned and scaled with it), and an og.Collider of shape SHAPE_HEIGHTFIELD naming the TerrainInfo's geometry makes it solid. og.TerrainScatter grows grass, flowers or stones over a layer near the camera. og.terrain.sample answers the surface's height and normal anywhere (pass it the entity's position, quaternion and scale); og.terrain.edit raises, lowers, flattens, smooths or paints it at runtime, and og.terrain.setLayer retunes a layer. Edits are world state: a snapshot keeps them, the world hash covers them, and a restore brings the ground back as it was (rebuilt from the cook with the restored edits replayed, for the renderer, physics and sample alike).
// Its tiles stream (phase 3): the cook stores the field as a pyramid of tiles, each compressed on its own, and every frame the renderer keeps on the GPU only the tiles the camera's clipmap wants, within each terrain's tile budget (16 MiB by default), decoded off the frame (a worker thread natively, the browser's DecompressionStream on the Web) a few at a time, the most needed first; until a tile arrives its area draws from the coarser level that holds it, then fades in. Nothing a tick reads depends on which tiles are resident (collision and og.terrain.sample read the whole field), so a run's world hash is the same streamed or not. The probe's `terrain_stream:resident` decodes every wanted tile before each frame instead (`terrain_stream:stream[:<MiB>]` streams again, within that budget), and `dump_stream`'s `terrain` gives the tiles resident, waiting, decoding and cached.

/**
 * A loaded terrain: its heights, control map and layers.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type TerrainData = number & { readonly [handleType]: "TerrainData" };

/**
 * A TerrainData handle field.
 *
 * A `TerrainData` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function TerrainData(value?: TerrainData, options?: FieldOptions): FieldSpec<"TerrainData">;

/**
 * What og.terrain.edit does under its brush.
 * - `"raise"`: Raise the heights by `strength` metres at the centre.
 * - `"lower"`: Lower them by `strength` metres at the centre.
 * - `"flatten"`: Pull them towards the brush centre's y, by `strength` (0 to 1) at the centre.
 * - `"smooth"`: Pull each towards its neighbours' mean, by `strength` (0 to 1) at the centre.
 * - `"paint"`: Paint `layer` into the control map, by `strength` (0 to 1) at the centre.
 */
export type TerrainBrushMode = "raise" | "lower" | "flatten" | "smooth" | "paint";

/**
 * Draws a loaded terrain placed by the entity's world transform (centred on its position, turned and scaled with it), with its layers (as cooked, or as og.terrain.setLayer left them), casting shadows unless told not to.
 *
 * The value of the `Terrain` component (token `og.Terrain`).
 */
export interface Terrain {
  /** The terrain (og.terrain.load's TerrainInfo.data). */
  data: TerrainData;
  /**
   * Cast the directional and local lights' shadows (it always receives them).
   *
   * @default true
   */
  castShadows: boolean;
  /**
   * The finest clipmap level's spacing (0: the terrain file's, or the samples' spacing).
   *
   * @range 0..64
   * @unit m (metres)
   * @tunable
   */
  lodSpacing: number;
  /**
   * Quads per side of every clipmap level (0: the terrain file's, else 64; 16 to 256, rounded to a multiple of 4).
   *
   * @range 0..256
   */
  lodRing: number;
}

/**
 * Scatters copies of a mesh (grass, flowers, stones) over a loaded terrain near the camera, where one of its layers shows: on a jittered grid fixed to the terrain, so each copy keeps its place, size and turn as the camera moves, every cell growing one with the probability of the layer's weight there (its control-map channel times its slope and altitude rules, over all the layers' weights). Copies keep their size and thin out (dithered) over the outer `fade` of `distance`. The renderer places them itself, a chunk of the grid at a time, once, and draws the chunks in range: copies cost about nothing per frame while the camera stays. They grow over the og.Terrain drawing the same data, placed as it is. Drawn only: no colliders, and the copies are not entities.
 *
 * The value of the `TerrainScatter` component (token `og.TerrainScatter`).
 */
export interface TerrainScatter {
  /** The terrain (og.terrain.load's TerrainInfo.data). */
  data: TerrainData;
  /** The mesh, its origin at its root. */
  geometry: Geometry;
  /** Its material. */
  material: Material;
  /**
   * The terrain layer that places it, 0 to 3.
   *
   * @range ..3
   */
  layer: number;
  /**
   * Copies per square metre where the layer's weight is 1.
   *
   * @default 2
   * @range 0..400
   * @tunable
   */
  density: number;
  /**
   * How far from the camera, along x and z, copies are placed.
   *
   * @default 40
   * @range 0..500
   * @unit m (metres)
   * @tunable
   */
  distance: number;
  /**
   * The outer part of `distance` over which copies thin out to nothing.
   *
   * @default 0.3
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  fade: number;
  /**
   * The smallest copy's uniform scale.
   *
   * @default 0.8
   * @range 0..
   * @tunable
   */
  minScale: number;
  /**
   * The largest copy's uniform scale.
   *
   * @default 1.2
   * @range 0..
   * @tunable
   */
  maxScale: number;
  /**
   * How far copies lean with the ground: 0 upright, 1 along its normal.
   *
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   * @tunable
   */
  align: number;
  /**
   * Another seed places the copies elsewhere.
   *
   * @default 1
   */
  seed: number;
}

/** A loaded terrain. */
export interface TerrainInfo {
  /** Its handle, for og.Terrain.data and this module's functions. */
  readonly data: TerrainData;
  /**
   * The height field registered with physics under this handle: the mesh of an og.Collider of shape SHAPE_HEIGHTFIELD.
   */
  readonly geometry: Geometry;
  /**
   * Its extent along x and z, centred on its entity.
   *
   * @unit m (metres)
   */
  readonly size: Vector2;
  /** Height samples along x. */
  readonly columns: number;
  /** Height samples along z. */
  readonly rows: number;
  /**
   * The lowest point above its entity.
   *
   * @unit m (metres)
   */
  readonly minHeight: number;
  /**
   * The highest.
   *
   * @unit m (metres)
   */
  readonly maxHeight: number;
  /** Its material layers (up to 4). */
  readonly layers: number;
  /** Whether its terrain file asks for a collider (`collision`, default true). */
  readonly collision: boolean;
  /** Its terrain file's `castShadows`. */
  readonly castShadows: boolean;
}

/**
 * Where og.terrain.sample looks.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TerrainSampleDesc {
  /**
   * The terrain.
   *
   * @default 0
   */
  readonly data?: TerrainData;
  /**
   * Where it stands: its og.Terrain entity's world position.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly origin?: Vector3;
  /**
   * How it is turned: its og.Terrain entity's world rotation (the zero value: along world x and z).
   *
   * @default { x: 0, y: 0, z: 0, w: 1 }
   */
  readonly quaternion?: Quaternion;
  /**
   * How it is scaled: its og.Terrain entity's world scale.
   *
   * @default { x: 1, y: 1, z: 1 }
   */
  readonly scale?: Vector3;
  /**
   * The world x.
   *
   * @default 0
   * @unit m (metres)
   */
  readonly x?: number;
  /**
   * The world z.
   *
   * @default 0
   * @unit m (metres)
   */
  readonly z?: number;
}

/** The terrain under a point. */
export interface TerrainSample {
  /**
   * The world y of the surface (the triangles drawn at the finest level and collided with) on the terrain's up axis through (x, z): straight down for a terrain turned only about y.
   *
   * @unit m (metres)
   */
  readonly height: number;
  /** The surface normal, smoothed between samples as the renderer shades it. */
  readonly normal: Vector3;
  /**
   * The normal's angle from straight up.
   *
   * @unit deg (degrees)
   */
  readonly slope: number;
  /** Whether the point is over the terrain (outside, the nearest edge answers). */
  readonly inside: boolean;
}

/** A terrain's box in world space. */
export interface TerrainBounds {
  /**
   * The lowest corner.
   *
   * @unit m (metres)
   */
  readonly min: Vector3;
  /**
   * The highest corner.
   *
   * @unit m (metres)
   */
  readonly max: Vector3;
}

/**
 * One runtime edit of a terrain (og.terrain.edit).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TerrainBrush {
  /**
   * What it does.
   *
   * @default "raise"
   */
  readonly mode?: TerrainBrushMode;
  /**
   * Where the terrain stands: its og.Terrain entity's world position.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly origin?: Vector3;
  /**
   * How it is turned: its og.Terrain entity's world rotation.
   *
   * @default { x: 0, y: 0, z: 0, w: 1 }
   */
  readonly quaternion?: Quaternion;
  /**
   * How it is scaled: its og.Terrain entity's world scale.
   *
   * @default { x: 1, y: 1, z: 1 }
   */
  readonly scale?: Vector3;
  /**
   * The brush's centre in world space; its y is flatten's target height.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly center?: Vector3;
  /**
   * The brush's radius in world metres (across the terrain's surface).
   *
   * @default 8
   * @range 0.01..
   * @unit m (metres)
   */
  readonly radius?: number;
  /**
   * World metres at the centre (raise, lower), or how far to go, 0 to 1 (flatten, smooth, paint).
   *
   * @default 1
   * @range 0..
   */
  readonly strength?: number;
  /**
   * The outer part of the radius over which it fades to nothing (0: a hard edge, 1: fading from the centre).
   *
   * @default 0.5
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly falloff?: number;
  /**
   * paint: the layer painted, 0 to 3.
   *
   * @default 0
   * @range ..3
   */
  readonly layer?: number;
}

/** What an edit changed. */
export interface TerrainEdit {
  /** Height or control-map samples it touched. */
  readonly samples: number;
  /**
   * The terrain's lowest point now.
   *
   * @unit m (metres)
   */
  readonly minHeight: number;
  /**
   * Its highest now.
   *
   * @unit m (metres)
   */
  readonly maxHeight: number;
}

/**
 * One material layer of a terrain: its look and the rules that place it. A layer shows where its control-map channel (1 without a map) times its slope band times its altitude band is high; the layers' weights are normalised.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TerrainLayerDesc {
  /**
   * Multiplies the layer's texture (linear).
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  readonly color?: Color;
  /**
   * Its roughness.
   *
   * @default 0.9
   * @range 0.04..1
   */
  readonly roughness?: number;
  /**
   * Metres per repeat of its textures.
   *
   * @default 4
   * @range 0.01..1000
   * @unit m (metres)
   */
  readonly tile?: number;
  /**
   * Sample by three projections blended by the normal (for cliffs).
   *
   * @default false
   */
  readonly triplanar?: boolean;
  /**
   * How strongly its normal map bends the surface.
   *
   * @default 1
   * @range 0..4
   */
  readonly normalStrength?: number;
  /**
   * The flattest slope it shows on.
   *
   * @default 0
   * @range 0..90
   * @unit deg (degrees)
   */
  readonly slopeMin?: number;
  /**
   * The steepest.
   *
   * @default 90
   * @range 0..90
   * @unit deg (degrees)
   */
  readonly slopeMax?: number;
  /**
   * How far past its slopes it fades out.
   *
   * @default 4
   * @range 0..90
   * @unit deg (degrees)
   */
  readonly slopeFade?: number;
  /**
   * The lowest height above the terrain's origin it shows at.
   *
   * @default -1000000
   * @unit m (metres)
   */
  readonly altitudeMin?: number;
  /**
   * The highest.
   *
   * @default 1000000
   * @unit m (metres)
   */
  readonly altitudeMax?: number;
  /**
   * How far past its heights it fades out.
   *
   * @default 4
   * @range 0..
   * @unit m (metres)
   */
  readonly altitudeFade?: number;
}







// ── stream ──────────────────────────────────────────────────────
// Asset streaming groups (phase 3): a scene's `stream "<name>" { asset ...; distance x y z r [unload d] | volume "<entity>"; priority n }` blocks name glTF models (without skins or morph targets), textures, texture arrays, flipbooks (an `array true` sheet brings its array), sounds, environment maps, palettes, looks and colour LUTs (`.cube`) that load and unload together, and what loads them: the camera within `r` of the anchor (kept until it is past `unload`, 1.25 r by default), the camera inside the named entity's box (its position ± half its scale, turned with it), or neither (og.stream.request only). `og cook` packs each group's files into `cooked/stream/<name>.ogb`; `og build --publish` ships the packs beside the boot bundle, which the shell and the service worker fetch first, and fetches a pack only when its group loads.
// At every tick boundary the engine decides which groups are wanted, admits them by priority (then name) within the memory budget (256 MiB by default; GPU textures and meshes and decoded audio, as the cook measured them), evicting lower-priority resident groups to make room for a higher one, and reads and puts in place at most one group's assets per boundary. A group that cannot fit waits, queued; one larger than the whole budget fails. Until its group is resident, a model draws its proxy (its coarsest LOD, untextured), a texture is a 1x1 grey placeholder and a sound plays as silence of its own length; then the real ones take over the same handles. Physics always collides with the proxy's meshes and a sound's length never changes, so nothing a tick reads depends on what is resident: a run's world hash is the same streamed or not.
// Texture arrays, flipbooks, environment maps, palettes, looks and colour LUTs have no stand-in: they load once their group is resident, and when it unloads the renderer lets them go and their handles wait (the sky lights the scene, Stylise.palette quantises to none, PostEffects.lut grades nothing, og.render.sprite of the array and applying the look are refused, an og.Sprite, a flipbook's sprite or a particle emitter of the array draws nothing) until the group is resident again, when they come back under the same handles. One the frame still uses when its group unloads (the map Environment lights it with, the palette Stylise quantises to, an array a sprite, a screen sprite or a particle emitter draws from, the look last applied, the LUT PostEffects grades with) is never freed under the frame: it is kept, with a warning naming it in the engine's log, until the frame stops using it. What is kept still takes its memory: its bytes count in the budget's `used` (and `kept`, and its group's StreamStatus.kept) until it is let go, so a group that would not fit beside it waits.
// Every change of a group's state sends StreamEvent at the start of the next tick; og.stream.status and og.stream.budget answer at once, and the probe's `dump_stream` lists the budget, the memory used and its peak, every group (trigger, state, why, since which tick, what it keeps for the frame after unloading with each asset's bytes), the recent changes, the terrain streamer's tiles and the fetch queue.
// What streams is read through one queue: first the chunks of the sounds cooked in chunks that a playhead needs soonest (the one it is playing, then by the seconds until it gets there at its pitch, a loop's start by the time to its wrap), then the loading groups' packs by priority (then name). At most four reads are in flight at once (on the web, fetches; natively a read completes at once, so only the order shows); the rest wait in that order. `dump_stream`'s `fetch` lists what is in flight and what waits, each with its path and what it is for (a sound's chunk and its seconds, a group's pack and its priority).

/**
 * Where a streaming group is.
 * - `"unloaded"`: Not wanted; its assets draw and play their stand-ins.
 * - `"queued"`: Wanted, waiting for room in the memory budget.
 * - `"loading"`: Admitted: its pack is being read (its memory already counts).
 * - `"resident"`: Its assets are in place of their stand-ins.
 * - `"failed"`: Its pack could not be read, or it alone needs more than the budget (StreamStatus.reason says which).
 */
export type StreamState = "unloaded" | "queued" | "loading" | "resident" | "failed";

/** `StreamEvent.state`: the group is unloaded (StreamState unloaded). */
export declare const STREAM_UNLOADED: 0;

/** `StreamEvent.state`: the group waits for room in the budget (StreamState queued). */
export declare const STREAM_QUEUED: 1;

/** `StreamEvent.state`: the group's pack is being read (StreamState loading). */
export declare const STREAM_LOADING: 2;

/** `StreamEvent.state`: the group's assets are in place (StreamState resident). */
export declare const STREAM_RESIDENT: 3;

/** `StreamEvent.state`: the group failed (StreamState failed; og.stream.status says why). */
export declare const STREAM_FAILED: 4;

/**
 * A group changed state: sent by og.stream at the start of the tick after the change.
 *
 * The payload of the `StreamEvent` event (token `og.StreamEvent`).
 */
export interface StreamEvent {
  /** The group's name (at most 63 bytes). */
  readonly group: string;
  /**
   * Its new state: STREAM_UNLOADED, STREAM_QUEUED, STREAM_LOADING, STREAM_RESIDENT or STREAM_FAILED (StreamState's order).
   */
  readonly state: number;
}

/** One streaming group. */
export interface StreamStatus {
  /** Its name. */
  readonly name: string;
  /** The scene that declares it. */
  readonly scene: string;
  /** Where it is. */
  readonly state: StreamState;
  /** Whether its trigger or a request wants it now. */
  readonly wanted: boolean;
  /** Whether og.stream.request asked for it (until og.stream.release). */
  readonly requested: boolean;
  /** Higher loads first and is evicted last. */
  readonly priority: number;
  /** manual, distance or volume. */
  readonly trigger: string;
  /**
   * What its assets take once loaded, in bytes.
   *
   * @unit bytes (bytes)
   */
  readonly memory: number;
  /**
   * Its pack's size, in bytes.
   *
   * @unit bytes (bytes)
   */
  readonly bytes: number;
  /** Its asset ids. */
  readonly assets: readonly string[];
  /** Why it is queued or failed; empty otherwise. */
  readonly reason: string;
  /** The tick of its last change. */
  readonly since: number;
  /** How many times it became resident. */
  readonly loads: number;
  /**
   * The bytes of its assets kept for the frame after it unloaded (counted in StreamBudget.used until let go); 0 otherwise.
   *
   * @unit bytes (bytes)
   */
  readonly kept: number;
}

/** The streaming memory budget and every group. */
export interface StreamBudget {
  /**
   * The bytes resident and loading groups may take.
   *
   * @unit bytes (bytes)
   */
  readonly budget: number;
  /**
   * The bytes they take now, with what unloaded groups keep for the frame.
   *
   * @unit bytes (bytes)
   */
  readonly used: number;
  /**
   * Of `used`, the bytes of assets kept for the frame after their group unloaded (an array a sprite still draws, the map the frame is lit by, ...).
   *
   * @unit bytes (bytes)
   */
  readonly kept: number;
  /**
   * The most they ever took (never above the budget at the time).
   *
   * @unit bytes (bytes)
   */
  readonly peak: number;
  /** Every group, by name. */
  readonly groups: readonly StreamStatus[];
}






// ── video ───────────────────────────────────────────────────────
// Footage in a game (docs/design.md §2.9): a `type video` asset cooked by ffmpeg into a texture array of its frames and its sound, loaded with og.load.video and played by og.VideoPlayer. The frame shown is a Texture (VideoInfo.texture, also VideoPlayer.texture) any material map, og.Sprite or UI image draws, and the sound plays on the entity's og.AudioSource in step with it.
// Playback is simulation: the og.video system (after the game's systems, before og.audio) advances VideoPlayer.time by the tick's dt times `speed` (not the time scale, so the picture stays with its sound), so the frame at a tick is the same on both hosts, in every run and after a snapshot is restored. With a 30 Hz tick a player started at 0 shows the source's frame 3 s in at tick 90.

/**
 * Cooked footage: its frames as a texture array, its sound (og.load.video).
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Video = number & { readonly [handleType]: "Video" };

/**
 * A Video handle field (0 is none).
 *
 * A `Video` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Video(value?: Video, options?: FieldOptions): FieldSpec<"Video">;

/** What og.load.video loaded. */
export interface VideoInfo {
  /** The video. */
  readonly video: Video;
  /** The texture that shows the frame its og.VideoPlayer is at (frame 0 until one plays it). */
  readonly texture: Texture;
  /**
   * Its sound track (og.load.sound's kind of handle, for an og.AudioSource); 0 when it has none or was cooked with `audio false`.
   */
  readonly sound: Sound;
  /** Each frame's width in texels. */
  readonly width: number;
  /** Each frame's height in texels. */
  readonly height: number;
  /** Frames kept (at most 256). */
  readonly frames: number;
  /** Frames per second kept. */
  readonly fps: number;
  /**
   * frames / fps.
   *
   * @unit s (seconds of simulation time)
   */
  readonly seconds: number;
  /** The frames' encoding on the GPU (`rgba8`, `bc7`, `astc4x4`, `etc2`). */
  readonly format: string;
  /**
   * Bytes its frames take on the GPU.
   *
   * @unit bytes (bytes)
   */
  readonly bytes: number;
}

/**
 * Plays a loaded video: VideoInfo.texture shows its current frame (frame n is the source at n / fps seconds), and when the entity has an og.AudioSource the og.video system sets its sound, playing, pitch and loop each tick so the sound plays in step: a seek or a restart starts the sound again at the new time on the next tick (AudioSource.startOffset). Several players of one video share its texture, which shows the first player's frame in query order; cook a second asset to show two moments of one clip.
 *
 * The value of the `VideoPlayer` component (token `og.VideoPlayer`).
 */
export interface VideoPlayer {
  /** The video (og.load.video's `video`); 0 plays nothing. */
  video: Video;
  /**
   * Whether time advances.
   *
   * @default true
   */
  playing: boolean;
  /**
   * Seconds into the video; written every tick, set it to seek.
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  time: number;
  /**
   * The playback rate: 1 as cooked, 0.5 half speed (the sound's pitch follows).
   *
   * @default 1
   * @range 0..4
   * @tunable
   */
  speed: number;
  /** Start again at 0 past the end; otherwise hold the last frame and set finished. */
  loop: boolean;
  /**
   * Written: the frame shown, floor(time * fps).
   *
   * @readonly written by the engine
   */
  readonly frame: number;
  /**
   * Written: a player without loop reached the end.
   *
   * @readonly written by the engine
   */
  readonly finished: boolean;
  /**
   * Written: the video's texture (VideoInfo.texture), for a material or sprite set up from the player.
   *
   * @readonly written by the engine
   */
  readonly texture: Texture;
}

// ── debug ───────────────────────────────────────────────────────
// Debug draw: lines, boxes, spheres, arrows, axes and labels in the world, each for a number of ticks, drawn over the finished frame (after every post effect, before the UI) so they read in every capture and never hide behind geometry; and the debug views (physics, navigation, bounds) the command channel's `debug:` actions switch.
// Development only: not in publish builds, where every member does nothing and answers its default, so a game may leave its debug draws in. Debug draw is rendering: it never feeds back into the simulation, and its draws are not world state (a snapshot or a replay does not carry them).

/**
 * How a debug shape looks and how long it lasts.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DebugDrawDesc {
  /**
   * The linear colour.
   *
   * @default { r: 1, g: 1, b: 0 }
   */
  readonly color?: Color;
  /**
   * Line width in CSS pixels.
   *
   * @default 2
   * @range 0..
   */
  readonly width?: number;
  /**
   * How many ticks after this one it stays; 0 is this frame only (draw it every tick it should show).
   *
   * @default 0
   * @unit ticks (fixed simulation steps)
   */
  readonly duration?: number;
  /**
   * The shape's rotation: a box's or a sphere's circles', the axes' frame.
   *
   * @default { x: 0, y: 0, z: 0, w: 1 }
   */
  readonly rotation?: Quaternion;
}












// ── path ────────────────────────────────────────────────────────
// Paths: a curve through or controlled by points, measured by arc length so a parameter t from 0 to 1 moves along it at an even speed however its points are spaced. catmullRom passes through every point (three.js CatmullRomCurve3, tension 0.5), bezier chains cubic segments `p0 c0 c1 p1 c2 c3 p2 ...` (three.js CubicBezierCurve3 in a CurvePath), linear joins the points with straight segments. A closed path joins its last point to its first and wraps t.
// Make one with og.path.create, or declare it in a scene (`path "<name>" { kind catmullRom; closed false; point x y z ... }`) and find it by name with og.path.find; a timeline's camera rides one with `rail "<name>"`. og.PathFollower moves an entity along one, og.CameraRail.path moves the camera along one, og.render's tube geometry can take its og.path.sample points. Paths are assets: they are not world state, and a hot reload registers them again.

/**
 * A path: its points, kind and arc-length table.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Path = number & { readonly [handleType]: "Path" };

/**
 * A Path handle field (0 is none).
 *
 * A `Path` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Path(value?: Path, options?: FieldOptions): FieldSpec<"Path">;

/**
 * How the points make the curve.
 * - `"catmullRom"`: Through every point, as three.js CatmullRomCurve3: tension scales the tangents (0.5 is Catmull-Rom).
 * - `"bezier"`: Cubic Bezier segments: anchor, two controls, anchor, ... (3n + 1 points open, 3n closed).
 * - `"linear"`: Straight segments between the points.
 */
export type PathKind = "catmullRom" | "bezier" | "linear";

/**
 * A PathKind field: the variant's index.
 *
 * A `PathKind` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function PathKind(value?: PathKind, options?: FieldOptions): FieldSpec<"PathKind">;

/**
 * What makes a path (three.js's Curve; PathDesc is og.nav's path query).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CurveDesc {
  /**
   * The points, in order: at least 2 (catmullRom, linear), 3n + 1 open or 3n closed (bezier).
   *
   * @default []
   */
  readonly points?: readonly Vector3[];
  /**
   * Join the last point to the first; t wraps.
   *
   * @default false
   */
  readonly closed?: boolean;
  /**
   * How the points make the curve.
   *
   * @default "catmullRom"
   */
  readonly kind?: PathKind;
  /**
   * catmullRom's tangent scale, as three.js's tension.
   *
   * @default 0.5
   * @range 0..1
   */
  readonly tension?: number;
}

/** A point of a path. */
export interface PathSample {
  /**
   * Where it is.
   *
   * @unit m (metres)
   */
  readonly position: Vector3;
  /** The unit direction of travel. */
  readonly tangent: Vector3;
  /**
   * A unit up perpendicular to the tangent: world +Y made perpendicular (world +Z where the path runs straight up or down).
   */
  readonly up: Vector3;
}

/** The point of a path nearest another point. */
export interface PathNearest {
  /**
   * Its arc-length parameter, 0 to 1.
   *
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly t: number;
  /**
   * Where it is.
   *
   * @unit m (metres)
   */
  readonly position: Vector3;
  /**
   * How far it is from the point asked about.
   *
   * @unit m (metres)
   */
  readonly distance: number;
}

/**
 * Moves the entity along a path at `speed`, by the engine's og.path system after the game's systems and before physics: og.Transform.position is the path's point at `distance` plus `offset` (in the path's frame with align, else in world axes). Advanced by scaled time (slow motion, hit stop). The distance is the component's own field, so a snapshot, a restore and the world hash carry it, and writing it moves the entity there.
 *
 * The value of the `PathFollower` component (token `og.PathFollower`).
 */
export interface PathFollower {
  /** The path (og.path.create or og.path.find); 0 does nothing. */
  path: Path;
  /**
   * Metres per second along the path; negative goes backwards.
   *
   * @default 1
   * @unit mps (metres per second)
   * @tunable
   */
  speed: number;
  /**
   * How far along the path it is, from its start; written every tick, set it to place the entity.
   *
   * @unit m (metres)
   */
  distance: number;
  /**
   * Wrap at the end (and the start, going backwards); otherwise stop there and set `finished`.
   *
   * @default true
   */
  loop: boolean;
  /**
   * Turn og.Transform.quaternion to face along the tangent (local -Z forward, +Y the path's up).
   */
  align: boolean;
  /**
   * Added to the path's point: along the path's right, up and backward axes with align, else world x, y and z.
   *
   * @unit m (metres)
   */
  offset: Vector3;
  /**
   * Moves while true.
   *
   * @default true
   */
  playing: boolean;
  /**
   * Written: a path without loop reached its end.
   *
   * @readonly written by the engine
   */
  readonly finished: boolean;
}






// ── timeline ────────────────────────────────────────────────────
// Timelines: cutscenes and sequences as data, Unity's Timeline and Godot's AnimationPlayer as an `.ogt` the agent reads and diffs. A `type timeline` file has a `duration` in seconds, `path "<name>" { ... }` blocks (as og.path's) and `track <kind> ["<entity>"] { key <seconds> { ... } }` blocks; `og cook` cooks it (importing caption files) and og.load.timeline loads it. Tracks: `camera` (position and target keys, or `rail "<path>"` with `at` 0..1 along it, or `follow "<entity>"` with offsets; fov; eased), `transform "<entity>"` (position, rotation in degrees or quaternion, scale; eased), `animation "<entity>"` (clip, speed, blendIn, loop; `model` names the glTF whose clips they are, else the entity's og.Model), `audio` (sound, volume, pitch, bus), `music` (sound, volume, fadeIn, fadeOut, end, loop, bus), `caption` (text from `key` to `end` with a UI type step `style` and an `anchor`; `captions "narration.srt" [offset]` imports a SubRip file or word timestamps `.json`), `event` (`do "<command-channel action>"`: teleport, set, a game's registered command; `retroactive true` runs it when a seek passes it), `transition` (kind cut, fade, wipe, dissolve, iris or slide; mode through, in or out; duration, color, from, over og.ui's screen cover) and `marker` (`name`, sending og.TimelineMarker when played past).
// Between two keys that set a value the later key's `ease` (og.tween's Ease names, linear when it gives none) mixes them; before the first and after the last the value is held. Every value is evaluated at the timeline's time, never accumulated over ticks, and that time is kept exactly (in steps of a thousandth of a tick), so seeking to a time and playing to it give the same world: a seek is how a video render starts at any frame.
// An og.Timeline component plays one on its entity, run by the engine's og.timeline system after the game's systems (and tweens) and before og.anim and physics, so what it sets is what animation, physics and the frame see that tick. While one with a camera track plays (or is paused) it owns og.Camera, and hands back the camera it found when it stops or ends. Its event keys' actions run at the start of the next tick, as og.ai's commands do. The command channel plays them too: `timeline:play:<name>` (`timelines/<name>.ogt`, or an asset id), `timeline:seek:<seconds>[:<name>]`, `timeline:pause[:<name>]`, `timeline:resume[:<name>]`, `timeline:stop[:<name>]`.

/**
 * A loaded timeline: its tracks, paths, sounds and clips.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type TimelineAsset = number & { readonly [handleType]: "TimelineAsset" };

/**
 * A TimelineAsset handle field (0 is none).
 *
 * A `TimelineAsset` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function TimelineAsset(value?: TimelineAsset, options?: FieldOptions): FieldSpec<"TimelineAsset">;

/**
 * Plays a loaded timeline. Set `playing` (or call og.timeline.play) to run it from `time`; write `time` (or call og.timeline.seek) to jump; og.timeline.stop goes back to 0 and lets go of what it owns.
 *
 * The value of the `Timeline` component (token `og.Timeline`).
 */
export interface Timeline {
  /** The timeline (og.load.timeline); 0 plays nothing. */
  asset: TimelineAsset;
  /** Advances while true. */
  playing: boolean;
  /**
   * Where it is, in seconds; written every tick it plays, set it to seek.
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  time: number;
  /**
   * Playback rate: 1 as authored, 0.5 half speed (0 holds the time while playing).
   *
   * @default 1
   * @range 0..16
   * @tunable
   */
  speed: number;
  /**
   * Start over at the end; otherwise stop playing there, hand the camera back and send og.TimelineEnded.
   */
  loop: boolean;
}

/**
 * A timeline started playing from stopped or ended (not on a resume from pause, nor on a loop).
 *
 * The payload of the `TimelineStarted` event (token `og.TimelineStarted`).
 */
export interface TimelineStarted {
  /** The og.Timeline's entity. */
  readonly entity: Entity;
  /** The timeline. */
  readonly asset: TimelineAsset;
}

/**
 * Playback passed a marker key (a seek over one does not send it).
 *
 * The payload of the `TimelineMarker` event (token `og.TimelineMarker`).
 */
export interface TimelineMarker {
  /** The og.Timeline's entity. */
  readonly entity: Entity;
  /** The timeline. */
  readonly asset: TimelineAsset;
  /** The marker's name (at most 63 bytes). */
  readonly name: string;
  /**
   * The marker's time.
   *
   * @unit s (seconds of simulation time)
   */
  readonly time: number;
}

/**
 * A timeline without loop reached its duration and stopped playing.
 *
 * The payload of the `TimelineEnded` event (token `og.TimelineEnded`).
 */
export interface TimelineEnded {
  /** The og.Timeline's entity. */
  readonly entity: Entity;
  /** The timeline. */
  readonly asset: TimelineAsset;
}

/** One track of a loaded timeline. */
export interface TimelineTrackInfo {
  /** camera, transform, animation, audio, music, caption, event, transition or marker. */
  readonly kind: string;
  /**
   * The entity a transform or animation track drives, the path a rail rides, the entity a camera follows; empty otherwise.
   */
  readonly target: string;
  /** Its keys. */
  readonly keys: number;
}

/** A loaded timeline. */
export interface TimelineInfo {
  /** Its handle. */
  readonly asset: TimelineAsset;
  /** Its `name`. */
  readonly name: string;
  /**
   * Its length.
   *
   * @unit s (seconds of simulation time)
   */
  readonly duration: number;
  /** Its tracks, in file order. */
  readonly tracks: readonly TimelineTrackInfo[];
  /** The paths it declares. */
  readonly paths: readonly string[];
  /** Its markers' names, in time order. */
  readonly markers: readonly string[];
}






// ── net ─────────────────────────────────────────────────────────
// Deterministic lockstep over a room (docs/design.md §2.10): every peer runs the whole world, and what crosses the room is each player's input, one per tick, and the world's hash. Turn it on with og.Lockstep (`enabled`, `players`); join a room (og.platform.roomJoin, or `og run --room ws://127.0.0.1:P/<room>` with `og relay` natively). Until the room holds `players` peers the world runs freely on each (a lobby, the local input driving the local player); then the authority (the lowest peer number) starts: every other peer takes its world's snapshot, the seats go to the peers in number order, and from then on tick t runs only when every seated player's input for t is in. A peer that joins a running room takes a free seat (or watches) from a snapshot a running peer hands it.
// Inputs are og.PlayerInput: one entity per player carries it with `player` set to its seat; the engine writes every other field at the start of each tick from that seat's input for the tick. A system reads the inputs from og.PlayerInput, never from og.Input, and gives this peer's own with og.net.submit (from a system that reads og.Input: what the local player holds now, sent for the tick inputDelay ticks on). Without a room, or with lockstep off, og.net.submit drives the local player's og.PlayerInput the next tick, so the same game plays alone.
// Every tick each peer hashes its world without its own session (og.Input, og.Pointers, og.Gamepad, og.Audio, og.Camera, og.Lockstep, the input, room, platform, asset and stream events, the UI's focus and hover) and every hashEvery ticks sends the hashes since its last batch; the first tick whose hashes differ is reported once as og.Desync, and the probe's `desync` dump (`dump_desync`) names it with the component-level diff of the two peers' snapshots of that tick: the answer to what is not deterministic (Math.random, Date.now, a wall clock, iteration over a Set of objects, a module variable, input read from og.Input).

/**
 * Lockstep's settings and this peer's place. The game sets enabled, players, inputDelay and hashEvery (the authority's are the room's once it starts: changes while running wait for the next start); the engine sets localPlayer and running. It is this peer's own: never hashed, never compared.
 *
 * The `Lockstep` resource (token `og.Lockstep`).
 */
export interface Lockstep {
  /** Whether the world takes its ticks in lockstep when in a room. */
  enabled: boolean;
  /**
   * The seats: the peers lockstep waits for before it starts, and whose inputs each tick waits for.
   *
   * @default 2
   * @range 1..16
   */
  players: number;
  /**
   * This peer's seat, the og.PlayerInput.player its input drives; `players` while it watches (a peer beyond the seats). 0 before the start.
   *
   * @readonly written by the engine
   */
  readonly localPlayer: number;
  /**
   * Ticks between a player's input and the tick it acts on: the time the input has to cross the room. 2 (33 ms at 60 Hz) on a LAN, 4 to 6 over the internet.
   *
   * @default 2
   * @range 1..30
   * @unit ticks (fixed simulation steps)
   */
  inputDelay: number;
  /**
   * How often the peers exchange their world hashes, in ticks (every tick is hashed, so the first divergent tick is exact); 0 compares nothing.
   *
   * @default 10
   * @range 0..600
   * @unit ticks (fixed simulation steps)
   */
  hashEvery: number;
  /**
   * Whether lockstep runs: the room filled the seats and the authority started (until this peer leaves the room or turns it off).
   *
   * @readonly written by the engine
   */
  readonly running: boolean;
}

/**
 * One player's input for this tick, the same on every peer: the engine writes every field but `player` at the start of each tick from the seat's input (zero for a seat nobody holds, and before its holder's first input). Its packed bytes are what crosses the room, read through this layout.
 *
 * The value of the `PlayerInput` component (token `og.PlayerInput`).
 */
export interface PlayerInput {
  /**
   * The seat whose input this entity carries (0 to og.Lockstep.players - 1); the game sets it when it spawns the player.
   */
  player: number;
  /** The buttons held, one bit each, as the game numbers them (bit 0 fire, bit 1 dash, ...). */
  buttons: number;
  /**
   * The move axis, left -1 to right 1.
   *
   * @range -1..1
   */
  x: number;
  /**
   * The move axis, back -1 to forward 1.
   *
   * @range -1..1
   */
  y: number;
  /** A second axis (aim, a cursor's world x), as the game uses it. */
  aimX: number;
  /** A second axis (aim, a cursor's world y), as the game uses it. */
  aimY: number;
  /**
   * The tick these values are for: the tick now, once the engine wrote them.
   *
   * @readonly written by the engine
   */
  readonly tick: number;
}

/**
 * This peer's world and another's differ: sent once, at the start of the tick after the hashes showed it. The probe's `desync` dump (`dump_desync`) names the component and field that differ, once the other peer's snapshot of that tick arrived.
 *
 * The payload of the `Desync` event (token `og.Desync`).
 */
export interface Desync {
  /**
   * The first divergent tick: the world after it differs (og.Time.tick of the snapshots compared).
   */
  readonly tick: number;
  /** The first peer whose world differs from this one. */
  readonly peer: number;
  /** How many peers' worlds differ from this one. */
  readonly peers: number;
}

/**
 * This peer's player's input: og.PlayerInput's fields but `player` and `tick`.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface InputDesc {
  /**
   * The buttons held, one bit each.
   *
   * @default 0
   */
  readonly buttons?: number;
  /**
   * The move axis, -1 to 1.
   *
   * @default 0
   */
  readonly x?: number;
  /**
   * The move axis, -1 to 1.
   *
   * @default 0
   */
  readonly y?: number;
  /**
   * A second axis.
   *
   * @default 0
   */
  readonly aimX?: number;
  /**
   * A second axis.
   *
   * @default 0
   */
  readonly aimY?: number;
}

/** One peer's hold on a seat. */
export interface LockstepSeat {
  /** The seat. */
  readonly seat: number;
  /** The peer that holds (or held) it. */
  readonly peer: number;
  /**
   * The first tick of its input.
   *
   * @unit ticks (fixed simulation steps)
   */
  readonly since: number;
  /**
   * The last tick of its input, once it left.
   *
   * @unit ticks (fixed simulation steps)
   */
  readonly until: number | null;
}

/** Lockstep on this peer, as the probe and a HUD read it. */
export interface LockstepStatus {
  /** Whether lockstep runs. */
  readonly running: boolean;
  /** This peer's number; null in no room. */
  readonly peer: number | null;
  /** The peers present, by number. */
  readonly peers: readonly number[];
  /**
   * The tick lockstep started (or this peer joined it) at.
   *
   * @unit ticks (fixed simulation steps)
   */
  readonly startTick: number | null;
  /** The seats, while running. */
  readonly players: number | null;
  /** Every seat's holds, in order. */
  readonly seats: readonly LockstepSeat[];
  /** The running peers without a seat. */
  readonly spectators: readonly number[];
  /** The seats whose input for the next tick has not arrived. */
  readonly waitingFor: readonly number[];
  /** Tick boundaries this peer waited at for an input. */
  readonly waits: number;
  /** The first divergent tick, once og.Desync was sent. */
  readonly desync: number | null;
}




// ── og ──────────────────────────────────────────────────────────

/**
 * The engine's root object: root functions (`og.game`), every engine
 * component, resource and event token (`og.Transform`, `og.Time`), and one
 * namespace per module (`og.load`, `og.commands`).
 */
export declare namespace og {
  /**
   * A world.restore applied: the tick after it (the first whole tick after a save taken mid-tick, whose next step completes the saved tick) delivers this, read through world.events like any event. Everything in the world came back as saved, request numbers and resources the session filled included; the session did not (og-game §17), so a system that reads this resets what belongs to the session: it forgets requests answered long ago, asks the store again for the settings and progress, and closes a menu the player had open. Sent as that tick starts, not when the restore applies, so the world between the two hashes as the save did.
   */
  const WorldRestored: EventType<WorldRestored>;
  /**
   * Declare the game. The bundle's default export is its result.
   *
   * @param desc Tick, scenes, systems and commands.
   */
  function game(desc?: GameDesc): Game;
  /** Where an entity is, relative to its parent (or the world). */
  const Transform: ComponentType<{ readonly position: FieldSpec<"Vector3">; readonly quaternion: FieldSpec<"Quaternion">; readonly scale: FieldSpec<"Vector3"> }, Transform>;
  /** The entity this one's Transform is relative to. Set by SpawnDesc.parent. */
  const Parent: ComponentType<{ readonly entity: FieldSpec<"Entity"> }, Parent>;
  /**
   * The world matrix the engine composes after the systems run, as three.js Object3D.matrixWorld.
   */
  const MatrixWorld: ComponentType<{ readonly matrix: FieldSpec<"Matrix4"> }, MatrixWorld>;
  /**
   * Transforms in the three.js vocabulary: position, quaternion and scale, composed into matrixWorld down the parent chain.
   */
  namespace transform {
    /**
     * A matrix from position, rotation and scale, as three.js Matrix4.compose.
     *
     * @param position The translation.
     * @param quaternion The rotation.
     * @param scale The scale.
     */
    function compose(position: Vector3, quaternion: Quaternion, scale: Vector3): Matrix4;
    /**
     * Position, rotation and scale from a matrix, as three.js Matrix4.decompose.
     *
     * @param matrix An affine matrix.
     */
    function decompose(matrix: Matrix4): Transform;
    /**
     * The rotation that points -Z from an eye towards a target, as three.js Object3D.lookAt for cameras.
     *
     * @param eye Where the viewer is.
     * @param target What it looks at.
     * @param up The up direction; the zero vector means +Y.
     */
    function lookAt(eye: Vector3, target: Vector3, up?: Vector3): Quaternion;
    /**
     * A rotation about an axis, as three.js Quaternion.setFromAxisAngle.
     *
     * @param axis A unit axis.
     * @param angle The angle in radians.
     */
    function fromAxisAngle(axis: Vector3, angle: number): Quaternion;
  }
  /**
   * The fixed tick's clock. Every value is a function of the tick count, so a replay sees the same numbers. The world keeps the time scale in a second resource, og.TimeScale (`scale`, `hitStop`, and `elapsed`, which is scaledElapsed), set through og.time.setScale and og.time.hitStop; read og.Time, which carries all of it. The probe's `get:og.Time` answers every field here, as a system's `time` does; `set:og.TimeScale.scale=0.5` slows time from a script.
   */
  const Time: ResourceType<{ readonly tick: FieldSpec<"u32">; readonly dt: FieldSpec<"f32">; readonly elapsed: FieldSpec<"f64">; readonly rate: FieldSpec<"u32">; readonly scale: FieldSpec<"f32">; readonly hitStop: FieldSpec<"u32">; readonly scaledDt: FieldSpec<"f32">; readonly scaledElapsed: FieldSpec<"f64"> }, Time>;
  /** Simulation time. Game code never reads the wall clock; it reads Time. */
  namespace time {
    /**
     * Set the global time scale (slow motion): physics, og.anim, flipbooks and tweens advance by dt * scale from the system after the call; systems may call it.
     *
     * @param world The world.
     * @param scale 0 to 10: 1 is normal speed.
     * @returns Refused outside 0..10.
     */
    function setScale(world: World, scale: number): Result<void>;
    /**
     * Freeze scaled time for `ticks` ticks from now (this tick included): physics, og.anim, flipbooks and scaled tweens hold while input, the UI, unscaled tweens and camera shake keep moving. A longer hit stop in progress is never shortened. Typical values at 60 Hz: 2-3 ticks for a light hit, 4-6 medium, 8-12 heavy, 12-20 a finishing blow (paired with slow motion).
     *
     * @param world The world.
     * @param ticks Ticks to hold.
     */
    function hitStop(world: World, ticks: number): Result<void>;
  }
  /**
   * The world's main random stream, seeded from GameDesc.seed (or the replay seed). Its state is part of every snapshot.
   */
  const Rng: ResourceType<{ readonly seed: FieldSpec<"u32">; readonly state: FieldSpec<"u32"> }, Rng>;
  /**
   * The input state at the start of this tick (og-input's og.Input, version 2). Edges (pressed, released) last exactly one tick; read the state through the methods.
   * Key sets are 128 bits over og-input's KEY_CODES (KeyboardEvent.code names): code i is bit i % 32 of word i / 32. Button masks: left 1, middle 2, right 4.
   */
  const Input: ResourceType<{ readonly pointer: FieldSpec<"Vector2">; readonly pointerInside: FieldSpec<"bool">; readonly wheel: FieldSpec<"Vector2">; readonly buttons: FieldSpec<"u32">; readonly buttonsPressed: FieldSpec<"u32">; readonly buttonsReleased: FieldSpec<"u32">; readonly buttonsReleaseNext: FieldSpec<"u32">; readonly keys0: FieldSpec<"u32">; readonly keys1: FieldSpec<"u32">; readonly keys2: FieldSpec<"u32">; readonly keys3: FieldSpec<"u32">; readonly pressed0: FieldSpec<"u32">; readonly pressed1: FieldSpec<"u32">; readonly pressed2: FieldSpec<"u32">; readonly pressed3: FieldSpec<"u32">; readonly released0: FieldSpec<"u32">; readonly released1: FieldSpec<"u32">; readonly released2: FieldSpec<"u32">; readonly released3: FieldSpec<"u32">; readonly releaseNext0: FieldSpec<"u32">; readonly releaseNext1: FieldSpec<"u32">; readonly releaseNext2: FieldSpec<"u32">; readonly releaseNext3: FieldSpec<"u32"> }, Input>;
  /**
   * Every pointer on the play surface at once, at the start of this tick (og-input's og.Pointers): a touch stick and a touch button can be held together. Pointer 0 is og.Input's pointer; ids 1 to 4 are further touches, in the order they landed (the lowest free id).
   * Edges last exactly one tick, as og.Input's do. Masks are bit `id`; read them through the methods.
   */
  const Pointers: ResourceType<{ readonly heldMask: FieldSpec<"u32">; readonly pressedMask: FieldSpec<"u32">; readonly releasedMask: FieldSpec<"u32">; readonly releaseNextMask: FieldSpec<"u32">; readonly position0: FieldSpec<"Vector2">; readonly position1: FieldSpec<"Vector2">; readonly position2: FieldSpec<"Vector2">; readonly position3: FieldSpec<"Vector2">; readonly position4: FieldSpec<"Vector2">; readonly start0: FieldSpec<"Vector2">; readonly start1: FieldSpec<"Vector2">; readonly start2: FieldSpec<"Vector2">; readonly start3: FieldSpec<"Vector2">; readonly start4: FieldSpec<"Vector2"> }, Pointers>;
  /**
   * The first four gamepads in the W3C standard mapping, at the start of this tick (og-input's og.Gamepad). On the web the shell polls the pads each display frame and sends their state through the input ring, so a pad is world state and replays; it also maps pads to keys for games that only read keys.
   * Button masks are bit `GamepadButton` index per pad; edges last exactly one tick, and a press and a release between two ticks leave both edges set. Read them through the methods.
   */
  const Gamepad: ResourceType<{ readonly connectedMask: FieldSpec<"u32">; readonly buttons0: FieldSpec<"u32">; readonly buttons1: FieldSpec<"u32">; readonly buttons2: FieldSpec<"u32">; readonly buttons3: FieldSpec<"u32">; readonly pressed0: FieldSpec<"u32">; readonly pressed1: FieldSpec<"u32">; readonly pressed2: FieldSpec<"u32">; readonly pressed3: FieldSpec<"u32">; readonly released0: FieldSpec<"u32">; readonly released1: FieldSpec<"u32">; readonly released2: FieldSpec<"u32">; readonly released3: FieldSpec<"u32">; readonly leftStick0: FieldSpec<"Vector2">; readonly leftStick1: FieldSpec<"Vector2">; readonly leftStick2: FieldSpec<"Vector2">; readonly leftStick3: FieldSpec<"Vector2">; readonly rightStick0: FieldSpec<"Vector2">; readonly rightStick1: FieldSpec<"Vector2">; readonly rightStick2: FieldSpec<"Vector2">; readonly rightStick3: FieldSpec<"Vector2">; readonly triggers0: FieldSpec<"Vector2">; readonly triggers1: FieldSpec<"Vector2">; readonly triggers2: FieldSpec<"Vector2">; readonly triggers3: FieldSpec<"Vector2"> }, Gamepad>;
  /**
   * A key went down (a down for a held key is a repeat and sets no edge). Input events are sent between ticks and read by the next tick's systems.
   */
  const KeyDown: EventType<KeyDown>;
  /**
   * Text typed or composed (an engine extension): the shell sends it from the page's `beforeinput` and composition events, the probe's `type:`, `compose:` and `commit:` from scripts. A text longer than 255 bytes arrives as several events in order. Key events still arrive for the keys; a text field types from these once a world has had one.
   */
  const TextInput: EventType<TextInput>;
  /** A key went up. */
  const KeyUp: EventType<KeyUp>;
  /** A pointer button went down (or a touch landed). */
  const PointerDown: EventType<PointerDown>;
  /** A pointer button went up (or a touch lifted). */
  const PointerUp: EventType<PointerUp>;
  /** A pointer moved. */
  const PointerMove: EventType<PointerMove>;
  /** A wheel or trackpad scroll. */
  const Wheel: EventType<Wheel>;
  /**
   * A screen reader's request on a UI control (an engine extension): the web shell sends it from its mirror of og.ui.a11y(); the UI takes it at the start of the next frame. A path longer than 255 bytes is refused.
   */
  const A11yAction: EventType<A11yAction>;
  /**
   * The camera the frame is rendered from. Rendering reads it after the tick; it never feeds back into the simulation.
   */
  const Camera: ResourceType<{ readonly position: FieldSpec<"Vector3">; readonly quaternion: FieldSpec<"Quaternion">; readonly fov: FieldSpec<"f32">; readonly near: FieldSpec<"f32">; readonly far: FieldSpec<"f32">; readonly zoom: FieldSpec<"f32">; readonly orthographic: FieldSpec<"bool">; readonly height: FieldSpec<"f32">; readonly background: FieldSpec<"Color">; readonly exposure: FieldSpec<"f32">; readonly trauma: FieldSpec<"f32">; readonly traumaDecay: FieldSpec<"f32">; readonly shakeOffset: FieldSpec<"f32">; readonly shakeRoll: FieldSpec<"f32">; readonly shakeFrequency: FieldSpec<"f32">; readonly punch: FieldSpec<"f32">; readonly punchDecay: FieldSpec<"f32">; readonly kick: FieldSpec<"Vector3">; readonly kickDecay: FieldSpec<"f32">; readonly zoneActive: FieldSpec<"bool">; readonly zoneMin: FieldSpec<"Vector2">; readonly zoneMax: FieldSpec<"Vector2">; readonly zoneFromActive: FieldSpec<"bool">; readonly zoneFromMin: FieldSpec<"Vector2">; readonly zoneFromMax: FieldSpec<"Vector2">; readonly zoneScroll: FieldSpec<"f32">; readonly zoneScrollTime: FieldSpec<"f32">; readonly layers: FieldSpec<"u32"> }, Camera>;
  /**
   * Orbits og.Camera around the entity carrying it, as three.js OrbitControls: a left-button drag on the play surface turns it, the wheel zooms, autoRotate turns it on its own.
   * Camera controllers are game-level systems the engine ships in TypeScript (og.systems.cameras); list them in GameDesc.systems: `systems: [...og.systems.cameras, ...yours]`. The first entity carrying one, in query order, drives the camera.
   * The camera sits `distance` from the orbited point (the entity's position plus `target`) in the direction given by `azimuth` about +Y and `polar` from +Y, and looks at the point. Every angle is computed with deterministic arithmetic, so a replay moves the camera identically on every host.
   */
  const OrbitCamera: ComponentType<{ readonly target: FieldSpec<"Vector3">; readonly distance: FieldSpec<"f32">; readonly minDistance: FieldSpec<"f32">; readonly maxDistance: FieldSpec<"f32">; readonly azimuth: FieldSpec<"f32">; readonly polar: FieldSpec<"f32">; readonly minPolarAngle: FieldSpec<"f32">; readonly maxPolarAngle: FieldSpec<"f32">; readonly rotateSpeed: FieldSpec<"f32">; readonly zoomSpeed: FieldSpec<"f32">; readonly autoRotate: FieldSpec<"bool">; readonly autoRotateSpeed: FieldSpec<"f32">; readonly enabled: FieldSpec<"bool">; readonly dragging: FieldSpec<"bool">; readonly pointer: FieldSpec<"Vector2"> }, OrbitCamera>;
  /**
   * Chases the entity carrying it with og.Camera: a third-person 3D view that eases towards a point offset from the entity and looks at it. Driven by og.systems.cameras; the first entity carrying one, in query order, drives the camera.
   */
  const FollowCamera: ComponentType<{ readonly offset: FieldSpec<"Vector3">; readonly lookOffset: FieldSpec<"Vector3">; readonly followRotation: FieldSpec<"bool">; readonly smoothTime: FieldSpec<"f32"> }, FollowCamera>;
  /**
   * Keeps og.Camera on the entity carrying it in the xy plane, for 2D games: the camera moves only when the entity leaves the dead zone, eases by smoothTime (smoothTimeY for y when set) and, with clamp, stays inside the bounds; lookAhead leads it in the direction the entity moves. The camera keeps its z, rotation and projection. Driven by og.systems.cameras; og.CameraZone entities then keep the view inside the zone the entity is in.
   */
  const Follow2D: ComponentType<{ readonly offset: FieldSpec<"Vector2">; readonly deadZone: FieldSpec<"Vector2">; readonly smoothTime: FieldSpec<"f32">; readonly smoothTimeY: FieldSpec<"f32">; readonly lookAhead: FieldSpec<"f32">; readonly lookAheadMax: FieldSpec<"Vector2">; readonly lookAheadSmoothing: FieldSpec<"f32">; readonly lead: FieldSpec<"Vector2">; readonly last: FieldSpec<"Vector2">; readonly clamp: FieldSpec<"bool">; readonly boundsMin: FieldSpec<"Vector2">; readonly boundsMax: FieldSpec<"Vector2"> }, Follow2D>;
  /**
   * A rectangle of the xy plane (min to max, world units, not moved by the entity's Transform) that keeps a 2D view inside it while the entity followed by og.Follow2D is in it: rooms, arenas, a level's bounds. Of overlapping zones the highest priority wins, then the first in query order. The engine's og.camera system picks the zone after the game's systems (into og.Camera's zone fields); entering another zone scrolls the view across over the new zone's scrollTime, or jumps when it is 0. A zone smaller than the view centres it.
   */
  const CameraZone: ComponentType<{ readonly min: FieldSpec<"Vector2">; readonly max: FieldSpec<"Vector2">; readonly priority: FieldSpec<"i32">; readonly scrollTime: FieldSpec<"f32"> }, CameraZone>;
  /**
   * Moves og.Camera along a spline through p0 to p(points - 1) (Catmull-Rom, through every point) to the point nearest the entity carrying it, easing by smoothTime: a corridor, a side-scroller's track, a 3D chase on rails. A perspective camera looks at the entity; an orthographic one keeps its z and rotation. Applied by the engine's og.camera system after the game's systems.
   */
  const CameraRail: ComponentType<{ readonly points: FieldSpec<"u32">; readonly p0: FieldSpec<"Vector3">; readonly p1: FieldSpec<"Vector3">; readonly p2: FieldSpec<"Vector3">; readonly p3: FieldSpec<"Vector3">; readonly p4: FieldSpec<"Vector3">; readonly p5: FieldSpec<"Vector3">; readonly p6: FieldSpec<"Vector3">; readonly p7: FieldSpec<"Vector3">; readonly smoothTime: FieldSpec<"f32">; readonly path: FieldSpec<"Path"> }, CameraRail>;
  /**
   * The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names.
   * Camera controllers are components (OrbitCamera, FollowCamera, Follow2D) on the entity the camera orbits or follows; the systems that drive them ship in TypeScript as og.systems.cameras.
   */
  namespace camera {
    /**
     * Add screen shake: trauma, clamped to 1, which og.Camera decays by traumaDecay. The shake grows with trauma squared, so 0.1-0.2 is a light hit, 0.25-0.4 a medium one, 0.5-0.8 an explosion or a boss slam; 1 is almost never right (nauseating). Visual only. Systems may call it; it takes effect at the engine's og.camera system this tick.
     *
     * @param world The world.
     * @param trauma Trauma to add, 0 to 1.
     */
    function shake(world: World, trauma: number): Result<void>;
    /**
     * A zoom punch: the view narrows by `amount` (0.05 is a tap, 0.15 a slam) and springs back by Camera.punchDecay. Visual only.
     *
     * @param world The world.
     * @param amount The fraction to zoom in (negative zooms out), -0.9 to 4.
     */
    function punch(world: World, amount: number): Result<void>;
    /**
     * A directional nudge of the view (recoil, an impact from one side): offset by `distance` along `direction`, returning by Camera.kickDecay. Visual only.
     *
     * @param world The world.
     * @param direction The direction to push the view (normalised here; zero does nothing).
     * @param distance How far, in world units.
     */
    function kick(world: World, direction: Vector3, distance: number): Result<void>;
  }
  /**
   * Draws a geometry with a material at the entity's MatrixWorld, as a three.js Mesh.
   * The material is a handle from og.render.standardMaterial or from a loaded glTF (GltfInfo.materials); its StandardMaterialDesc maps to the renderer as that struct's docs say.
   */
  const Mesh: ComponentType<{ readonly geometry: FieldSpec<"Geometry">; readonly material: FieldSpec<"Material">; readonly castShadow: FieldSpec<"bool">; readonly receiveShadow: FieldSpec<"bool">; readonly visible: FieldSpec<"bool">; readonly flash: FieldSpec<"f32">; readonly flashColor: FieldSpec<"Color">; readonly materialLayer: FieldSpec<"i32">; readonly layers: FieldSpec<"u32"> }, Mesh>;
  /**
   * The sun, as three.js DirectionalLight: parallel light along the entity's local -Z (aim it with transform.lookAt, as a camera). The first one found is drawn, with three shadow cascades.
   * Without one, the renderer's default sun (from above and to the side, intensity 3) lights the scene.
   */
  const DirectionalLight: ComponentType<{ readonly color: FieldSpec<"Color">; readonly intensity: FieldSpec<"f32">; readonly castShadow: FieldSpec<"bool">; readonly shadowDistance: FieldSpec<"f32">; readonly shadowSoftness: FieldSpec<"f32">; readonly cascadeBlend: FieldSpec<"f32"> }, DirectionalLight>;
  /**
   * A light shining in every direction from the entity's position, as three.js PointLight. Its radiance at distance d is color * intensity / d^2, faded smoothly to zero at distance.
   * Lights are clustered (Forward+): each pixel shades only the lights whose distance reaches it, so hundreds are cheap. Lights out of view are skipped; up to MAX_LIGHTS in view are drawn, and past that (or past what one screen cluster holds, MAX_CLUSTER_LIGHTS) the ones brightest at the camera are kept. Of those in view with castShadow, RenderCapabilities.maxShadowedLights (8 on the full tier, 4 on WebGL2) get shadows, the brightest at the camera first.
   */
  const PointLight: ComponentType<{ readonly color: FieldSpec<"Color">; readonly intensity: FieldSpec<"f32">; readonly distance: FieldSpec<"f32">; readonly castShadow: FieldSpec<"bool">; readonly cookie: FieldSpec<"Texture"> }, PointLight>;
  /** A cone of light along the entity's local -Z, as three.js SpotLight. */
  const SpotLight: ComponentType<{ readonly color: FieldSpec<"Color">; readonly intensity: FieldSpec<"f32">; readonly distance: FieldSpec<"f32">; readonly angle: FieldSpec<"f32">; readonly penumbra: FieldSpec<"f32">; readonly castShadow: FieldSpec<"bool">; readonly cookie: FieldSpec<"Texture"> }, SpotLight>;
  /**
   * The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping. Read after each tick; it never feeds back into the simulation. Camera.exposure is the manual exposure.
   */
  const RenderSettings: ResourceType<{ readonly environmentIntensity: FieldSpec<"f32">; readonly sky: FieldSpec<"bool">; readonly bloom: FieldSpec<"bool">; readonly bloomThreshold: FieldSpec<"f32">; readonly bloomKnee: FieldSpec<"f32">; readonly bloomIntensity: FieldSpec<"f32">; readonly autoExposure: FieldSpec<"bool">; readonly adaptSpeedUp: FieldSpec<"f32">; readonly adaptSpeedDown: FieldSpec<"f32">; readonly toneMapped: FieldSpec<"bool">; readonly toneMapping: FieldSpec<"ToneMapping">; readonly exposure: FieldSpec<"f32">; readonly decalAtlas: FieldSpec<"Texture">; readonly lodBias: FieldSpec<"f32">; readonly bakedLighting: FieldSpec<"bool"> }, RenderSettings>;
  /**
   * Image-based light from a cooked HDR environment map instead of the sky, as three.js scene.environment, and the map behind the scene, as scene.background = the environment. Load the map with og.load.environment: the cook has already prefiltered it (GGX levels for reflections from mirror to rough, harmonics for the diffuse light), so setting it costs a few copy passes on any tier, WebGL2 included. Without a map the sky lights the scene as before (og.Sky, a DayNight, or the built-in one). Read after each tick; it never feeds back into the simulation.
   */
  const Environment: ResourceType<{ readonly map: FieldSpec<"EnvironmentMap">; readonly intensity: FieldSpec<"f32">; readonly rotation: FieldSpec<"f32">; readonly background: FieldSpec<"bool">; readonly backgroundIntensity: FieldSpec<"f32"> }, Environment>;
  /**
   * The post stack beyond RenderSettings' bloom, exposure and tone mapping, and camera shake. Every effect is off by default (its switch false or its amount 0) and then costs nothing: no pass is recorded. Read after each tick; it never feeds back into the simulation.
   * Screen-space ambient occlusion, outlines, depth of field, motion blur and god rays run a depth-normal prepass, turned on with them. Grading, vignette, aberration, grain and the flash apply to the display colour after tone mapping; the UI and screen-space sprites (the overlay) come after all of it.
   */
  const PostEffects: ResourceType<{ readonly ao: FieldSpec<"bool">; readonly aoRadius: FieldSpec<"f32">; readonly aoIntensity: FieldSpec<"f32">; readonly aoMethod: FieldSpec<"AoMethod">; readonly outline: FieldSpec<"bool">; readonly outlineColor: FieldSpec<"Color">; readonly outlineWidth: FieldSpec<"f32">; readonly outlineDepthThreshold: FieldSpec<"f32">; readonly outlineNormalThreshold: FieldSpec<"f32">; readonly depthOfField: FieldSpec<"bool">; readonly dofFocusDistance: FieldSpec<"f32">; readonly dofFocusRange: FieldSpec<"f32">; readonly dofMaxBlur: FieldSpec<"f32">; readonly dofFalloff: FieldSpec<"f32">; readonly dofQuality: FieldSpec<"DofQuality">; readonly motionBlur: FieldSpec<"bool">; readonly motionBlurShutter: FieldSpec<"f32">; readonly motionBlurSamples: FieldSpec<"u32">; readonly motionBlurMaxPixels: FieldSpec<"f32">; readonly godRays: FieldSpec<"bool">; readonly godRaysIntensity: FieldSpec<"f32">; readonly godRaysDecay: FieldSpec<"f32">; readonly godRaysDensity: FieldSpec<"f32">; readonly godRaysTint: FieldSpec<"Color">; readonly godRaysSamples: FieldSpec<"u32">; readonly godRaysThreshold: FieldSpec<"f32">; readonly glow: FieldSpec<"bool">; readonly glowIntensity: FieldSpec<"f32">; readonly lut: FieldSpec<"Lut">; readonly lutIntensity: FieldSpec<"f32">; readonly vignette: FieldSpec<"bool">; readonly vignetteIntensity: FieldSpec<"f32">; readonly vignetteRadius: FieldSpec<"f32">; readonly vignetteSmoothness: FieldSpec<"f32">; readonly vignetteColor: FieldSpec<"Color">; readonly chromaticAberration: FieldSpec<"f32">; readonly filmGrain: FieldSpec<"f32">; readonly filmGrainSize: FieldSpec<"f32">; readonly flash: FieldSpec<"f32">; readonly flashColor: FieldSpec<"Color">; readonly shakeTrauma: FieldSpec<"f32">; readonly shakeMaxOffset: FieldSpec<"f32">; readonly shakeMaxRoll: FieldSpec<"f32">; readonly shakeFrequency: FieldSpec<"f32">; readonly shakeSeed: FieldSpec<"u32">; readonly ssr: FieldSpec<"bool">; readonly ssrMaxDistance: FieldSpec<"f32">; readonly ssrThickness: FieldSpec<"f32">; readonly ssrSteps: FieldSpec<"u32">; readonly ssrFade: FieldSpec<"f32">; readonly ssrIntensity: FieldSpec<"f32">; readonly custom1: FieldSpec<"Post">; readonly custom2: FieldSpec<"Post">; readonly custom3: FieldSpec<"Post">; readonly custom4: FieldSpec<"Post">; readonly custom5: FieldSpec<"Post">; readonly custom6: FieldSpec<"Post">; readonly custom7: FieldSpec<"Post">; readonly custom8: FieldSpec<"Post"> }, PostEffects>;
  /**
   * Stylisation primitives that move the frame away from the photographic: a low-resolution pixel grid scaled up by whole numbers, fewer tones or a fixed palette through an ordered or blue-noise dither, halftone dots, cross-hatching, painterly smoothing over a canvas, ink lines on tonal edges, flat colour bands by depth, and the vertex snapping, affine textures and coarse depth of early 3D hardware. They compose: a look (og.load.look) sets this with PostEffects and RenderSettings, and the game's art direction is in that data, not in the engine. Every primitive is off at its zero value (0, false, none) and then costs nothing: no pass is recorded and no pipeline is built. RenderCapabilities.stylise names the primitives this renderer draws; a field it does not name is carried and read back but not drawn. Read after each tick; it never feeds back into the simulation.
   * Order in the frame: depth bands colour the lit scene with the fog, before tone mapping; after tone mapping and PostEffects' grading come paint, hatching, halftone and ink, then posterize and palette quantise through the dither, all at the logical resolution, and the pixel grid is scaled up last. The UI and screen-space sprites are drawn over the scaled frame at the output's resolution, unstylised.
   */
  const Stylise: ResourceType<{ readonly pixelHeight: FieldSpec<"u32">; readonly pixelUpscale: FieldSpec<"u32">; readonly pixelSnap: FieldSpec<"bool">; readonly posterize: FieldSpec<"u32">; readonly palette: FieldSpec<"Palette">; readonly dither: FieldSpec<"DitherPattern">; readonly ditherStrength: FieldSpec<"f32">; readonly ditherScale: FieldSpec<"u32">; readonly halftone: FieldSpec<"f32">; readonly halftoneSize: FieldSpec<"f32">; readonly halftoneAngle: FieldSpec<"f32">; readonly halftoneColor: FieldSpec<"Color">; readonly halftoneSpace: FieldSpec<"StyliseSpace">; readonly hatching: FieldSpec<"f32">; readonly hatchingScale: FieldSpec<"f32">; readonly hatchingLevels: FieldSpec<"u32">; readonly hatchingAngle: FieldSpec<"f32">; readonly hatchingColor: FieldSpec<"Color">; readonly hatchingSpace: FieldSpec<"StyliseSpace">; readonly hatchingDepth: FieldSpec<"f32">; readonly paintRadius: FieldSpec<"f32">; readonly paintAnisotropy: FieldSpec<"f32">; readonly paintEdgeDarkening: FieldSpec<"f32">; readonly paintBleed: FieldSpec<"f32">; readonly paintCanvas: FieldSpec<"Texture">; readonly paintCanvasStrength: FieldSpec<"f32">; readonly paintCanvasScale: FieldSpec<"f32">; readonly paintCanvasSpace: FieldSpec<"StyliseSpace">; readonly paintCanvasWorldScale: FieldSpec<"f32">; readonly ink: FieldSpec<"bool">; readonly inkThreshold: FieldSpec<"f32">; readonly inkWidth: FieldSpec<"f32">; readonly inkColor: FieldSpec<"Color">; readonly inkDepth: FieldSpec<"f32">; readonly inkDepthThreshold: FieldSpec<"f32">; readonly inkNormalThreshold: FieldSpec<"f32">; readonly depthBands: FieldSpec<"u32">; readonly depthBandsNear: FieldSpec<"f32">; readonly depthBandsFar: FieldSpec<"f32">; readonly depthBandsNearColor: FieldSpec<"Color">; readonly depthBandsFarColor: FieldSpec<"Color">; readonly depthBandsFlatten: FieldSpec<"f32">; readonly vertexSnap: FieldSpec<"f32">; readonly affineTexturing: FieldSpec<"f32">; readonly depthBits: FieldSpec<"u32"> }, Stylise>;
  /**
   * Stylised defaults for every lit material, so a whole scene takes one treatment without editing each material: toon shading and its ramp, a coloured shadow side, rim light, outlines whose width falls off with distance, and flat facets. Each applies to a material (og.render.standardMaterial or a loaded glTF's) that leaves that field at its own default; unlit and shader materials, materials with StandardMaterialDesc.styled false, sprites and the UI are untouched, and toon passes by a material with a MeshPhysicalMaterial lobe (clearcoat, sheen, transmission, iridescence, anisotropy, ior), which keeps its physical shading. All off (false, 0) is no change. A look sets it; RenderCapabilities.stylise names what this renderer draws. Read after each tick; it never feeds back into the simulation.
   */
  const MaterialStyle: ResourceType<{ readonly toon: FieldSpec<"bool">; readonly gradientMap: FieldSpec<"Texture">; readonly shadowTint: FieldSpec<"Color">; readonly shadowTintAmount: FieldSpec<"f32">; readonly rimColor: FieldSpec<"Color">; readonly rimIntensity: FieldSpec<"f32">; readonly rimPower: FieldSpec<"f32">; readonly outlineColor: FieldSpec<"Color">; readonly outlineWidth: FieldSpec<"f32">; readonly outlineDistance: FieldSpec<"f32">; readonly flatShading: FieldSpec<"bool"> }, MaterialStyle>;
  /**
   * 2D lighting of the sprite layers minLayer to maxLayer (sprites, 2D particles and og.Tilemap layers there, a map layer unless its `lit` property is false): their colour is multiplied by ambient plus every Light2D reaching the pixel, shadowed by Occluder2Ds and sprites with castShadow. Other layers (and the overlay) are drawn as authored. Off by default.
   */
  const Lighting2D: ResourceType<{ readonly enabled: FieldSpec<"bool">; readonly ambient: FieldSpec<"Color">; readonly minLayer: FieldSpec<"i32">; readonly maxLayer: FieldSpec<"i32">; readonly resolution: FieldSpec<"f32"> }, Lighting2D>;
  /**
   * A light in the 2D plane at the entity's position (z ignored): color times intensity at its centre, falling to zero at radius with (1 - d/r)^falloff, shadowed by occluders when castShadows. Lighting2D.enabled turns 2D lighting on; at most 64 are drawn.
   */
  const Light2D: ComponentType<{ readonly color: FieldSpec<"Color">; readonly intensity: FieldSpec<"f32">; readonly radius: FieldSpec<"f32">; readonly falloff: FieldSpec<"f32">; readonly sourceRadius: FieldSpec<"f32">; readonly castShadows: FieldSpec<"bool"> }, Light2D>;
  /**
   * A rectangle that blocks 2D light (walls, crates, characters), centred on the entity's position and turned with it about z. Pixels inside it are not shadowed by it. At most 128 are drawn.
   */
  const Occluder2D: ComponentType<{ readonly width: FieldSpec<"f32">; readonly height: FieldSpec<"f32"> }, Occluder2D>;
  /**
   * A projected decal (scorch marks, blood, posters, bullet holes): the entity's unit box (-0.5 to 0.5 on each axis, scaled by its Transform) projected along its local -Z onto every lit opaque surface inside it, shaded with the surface. Its image is the uvMin to uvMax region of RenderSettings.decalAtlas (a solid colour without one). At most 32 are drawn.
   */
  const Decal: ComponentType<{ readonly color: FieldSpec<"Color">; readonly opacity: FieldSpec<"f32">; readonly uvMin: FieldSpec<"Vector2">; readonly uvMax: FieldSpec<"Vector2">; readonly emissive: FieldSpec<"Color">; readonly roughness: FieldSpec<"f32">; readonly angleFade: FieldSpec<"f32">; readonly visible: FieldSpec<"bool"> }, Decal>;
  /**
   * Emits particles from an og.load.emitter asset at the entity's MatrixWorld. Particles are simulated by the renderer from (seed, startTick, the tick), so they are deterministic and replay, and never feed back into the simulation: compute on the full tier, the CPU on WebGL2, identical either way.
   */
  const ParticleSystem: ComponentType<{ readonly emitter: FieldSpec<"ParticleEmitter">; readonly seed: FieldSpec<"u32">; readonly startTick: FieldSpec<"u32">; readonly stopTick: FieldSpec<"u32">; readonly visible: FieldSpec<"bool">; readonly layers: FieldSpec<"u32"> }, ParticleSystem>;
  /**
   * A streak behind the entity: its world positions over the last `lifetime` seconds as a camera-facing strip, from widthStart and color at the head to widthEnd and colorEnd at the tail.
   */
  const Trail: ComponentType<{ readonly lifetime: FieldSpec<"f32">; readonly minDistance: FieldSpec<"f32">; readonly widthStart: FieldSpec<"f32">; readonly widthEnd: FieldSpec<"f32">; readonly color: FieldSpec<"Color">; readonly opacity: FieldSpec<"f32">; readonly colorEnd: FieldSpec<"Color">; readonly opacityEnd: FieldSpec<"f32">; readonly texture: FieldSpec<"Texture">; readonly uvTile: FieldSpec<"f32">; readonly blend: FieldSpec<"StripBlend">; readonly layered: FieldSpec<"bool">; readonly layer: FieldSpec<"i32"> }, Trail>;
  /**
   * A strip from the entity's position to `end` whose texture scrolls with the tick: lasers, tethers, lightning (jitter).
   */
  const Beam: ComponentType<{ readonly end: FieldSpec<"Vector3">; readonly width: FieldSpec<"f32">; readonly color: FieldSpec<"Color">; readonly opacity: FieldSpec<"f32">; readonly texture: FieldSpec<"Texture">; readonly tile: FieldSpec<"f32">; readonly scroll: FieldSpec<"f32">; readonly segments: FieldSpec<"u32">; readonly jitter: FieldSpec<"f32">; readonly seed: FieldSpec<"u32">; readonly blend: FieldSpec<"StripBlend"> }, Beam>;
  /**
   * A sprite drawn every frame at the entity's MatrixWorld (its xy and z, its rotation about z; its scale multiplies width and height, so squash and stretch is a tween of og.Transform.scale). With an og.Flipbook on the entity, the texture (or an `array true` sheet's texture array layer), region and anchor come from the flipbook's current frame.
   * Sorted with the immediate sprites and text by layer, then (sortY) by y, then texture: one draw per run of one texture. Tile layers draw before the sprites of their layer.
   */
  const Sprite: ComponentType<{ readonly texture: FieldSpec<"Texture">; readonly array: FieldSpec<"TextureArray">; readonly arrayLayer: FieldSpec<"u32">; readonly width: FieldSpec<"f32">; readonly height: FieldSpec<"f32">; readonly anchor: FieldSpec<"Vector2">; readonly color: FieldSpec<"Color">; readonly opacity: FieldSpec<"f32">; readonly uvMin: FieldSpec<"Vector2">; readonly uvMax: FieldSpec<"Vector2">; readonly layer: FieldSpec<"i32">; readonly sortY: FieldSpec<"bool">; readonly flipX: FieldSpec<"bool">; readonly flipY: FieldSpec<"bool">; readonly flash: FieldSpec<"f32">; readonly flashColor: FieldSpec<"Color">; readonly glow: FieldSpec<"f32">; readonly blend: FieldSpec<"SpriteBlend">; readonly castShadow: FieldSpec<"bool">; readonly sliceX: FieldSpec<"Vector2">; readonly sliceY: FieldSpec<"Vector2">; readonly sliceScale: FieldSpec<"f32">; readonly shadow: FieldSpec<"f32">; readonly shadowSize: FieldSpec<"Vector2">; readonly shadowOffset: FieldSpec<"Vector2">; readonly dropShadow: FieldSpec<"f32">; readonly dropShadowHeight: FieldSpec<"f32">; readonly toneMapped: FieldSpec<"bool">; readonly visible: FieldSpec<"bool">; readonly layers: FieldSpec<"u32"> }, Sprite>;
  /**
   * Ghosts of the entity's og.Sprite: every `interval` ticks its pose (position, rotation, scale, frame, flips) is kept, and the last `count` are drawn behind it, fading from `opacity` to 0, tinted by color. A dash, a speed-up, a teleport. The history is world state (the og.sprite state block).
   */
  const Afterimage: ComponentType<{ readonly count: FieldSpec<"u32">; readonly interval: FieldSpec<"u32">; readonly opacity: FieldSpec<"f32">; readonly color: FieldSpec<"Color">; readonly additive: FieldSpec<"bool">; readonly active: FieldSpec<"bool"> }, Afterimage>;
  /**
   * A local reflection: a cube of what is around the entity's position, sampled by every lit surface inside its box with box projection (the reflected ray meets the box, so a room's walls reflect where they are, not at infinity) and blended with the environment map over blendDistance at the box's faces. Interiors that the sky must not light: a chrome sphere in a room shows the room. Overlapping probes: the smallest wins. The cube is rendered from the probe's position without it (nothing is drawn twice) and without the probes' own reflections.
   */
  const ReflectionProbe: ComponentType<{ readonly size: FieldSpec<"Vector3">; readonly resolution: FieldSpec<"u32">; readonly mode: FieldSpec<"ReflectionProbeMode">; readonly interval: FieldSpec<"u32">; readonly intensity: FieldSpec<"f32">; readonly blendDistance: FieldSpec<"f32">; readonly visible: FieldSpec<"bool"> }, ReflectionProbe>;
  /**
   * Draws a geometry's vertices as lines at the entity's MatrixWorld, every frame (three.js Line, LineSegments or LineLoop over a BufferGeometry): a static set of lines kept as data, a level's outline, a track, a constellation. The vertices are taken in order (the geometry's indices are not read), with its vertex colours times color.
   */
  const Lines: ComponentType<{ readonly geometry: FieldSpec<"Geometry">; readonly mode: FieldSpec<"LineMode">; readonly color: FieldSpec<"Color">; readonly opacity: FieldSpec<"f32">; readonly width: FieldSpec<"f32">; readonly worldUnits: FieldSpec<"bool">; readonly depthTest: FieldSpec<"bool">; readonly dashSize: FieldSpec<"f32">; readonly gapSize: FieldSpec<"f32">; readonly visible: FieldSpec<"bool">; readonly layers: FieldSpec<"u32"> }, Lines>;
  /**
   * Draws a geometry's vertices as a point cloud at the entity's MatrixWorld, every frame (three.js Points over a BufferGeometry): star fields, scanned surfaces, sparse markers kept as data. Vertex colours times color.
   */
  const Points: ComponentType<{ readonly geometry: FieldSpec<"Geometry">; readonly color: FieldSpec<"Color">; readonly opacity: FieldSpec<"f32">; readonly size: FieldSpec<"f32">; readonly sizeAttenuation: FieldSpec<"bool">; readonly depthTest: FieldSpec<"bool">; readonly visible: FieldSpec<"bool">; readonly layers: FieldSpec<"u32"> }, Points>;
  /**
   * A secondary camera, as a second three.js camera rendered with setRenderTarget or setViewport: it draws the scene from the entity's MatrixWorld (looking down its local -Z, as og.Camera) into a render target (target), or, without one, into a rect of the frame (viewport: split-screen, picture-in-picture) over the finished image and under the UI.
   * It draws what the main camera draws, from its own eye and on its own layers: the meshes and models (skinned and with any material: hooks and full shaders run their vertex and fragment entries), opaque then transparent back to front, the terrain and the sky, the particle systems, og.Lines and og.Points, world sprites (og.Sprite, og.render.sprite) and world text, lit by the frame's lights and shadowed by its shadow maps. Each view is a draw of everything its layers admit: keep a view cheap with its layers and `every`. Screen sprites, the UI and debug draw stay on the main frame. Of the post chain, an ldr target and a viewport carry the view's exposure (its camera's) and the frame's tone curve; an hdr target carries linear radiance before exposure, exposed by the frame that shows it. Neither runs the frame's bloom, AO, SSR, depth of field, colour grading or custom post passes, whose inputs are the main camera's. Sprites in a view are not 2D-lit and do not glow; trails and 2D fluids draw on the main frame only. Views draw lowest order first, all before the frame.
   */
  const CameraView: ComponentType<{ readonly target: FieldSpec<"RenderTarget">; readonly viewportX: FieldSpec<"f32">; readonly viewportY: FieldSpec<"f32">; readonly viewportWidth: FieldSpec<"f32">; readonly viewportHeight: FieldSpec<"f32">; readonly fov: FieldSpec<"f32">; readonly near: FieldSpec<"f32">; readonly far: FieldSpec<"f32">; readonly zoom: FieldSpec<"f32">; readonly orthographic: FieldSpec<"bool">; readonly height: FieldSpec<"f32">; readonly exposure: FieldSpec<"f32">; readonly layers: FieldSpec<"u32">; readonly order: FieldSpec<"i32">; readonly clear: FieldSpec<"bool">; readonly clearColor: FieldSpec<"Color">; readonly every: FieldSpec<"u32">; readonly active: FieldSpec<"bool"> }, CameraView>;
  /**
   * A planar mirror: the main camera reflected in the entity's xy plane (its local +Z the mirror's facing side), drawn into target fitted to a width by height rect centred on the entity, as a mirrored og.CameraView. Give the mirror's mesh (an og.render.planeGeometry of the same size) a material that reads the target: an hdr target as emissiveMap with color black, or an ldr target as an unlit map. Nothing behind the glass is drawn; seen from behind, it draws nothing.
   */
  const Mirror: ComponentType<{ readonly target: FieldSpec<"RenderTarget">; readonly width: FieldSpec<"f32">; readonly height: FieldSpec<"f32">; readonly layers: FieldSpec<"u32">; readonly every: FieldSpec<"u32">; readonly active: FieldSpec<"bool"> }, Mirror>;
  /**
   * A rectangle of light, as three.js RectAreaLight: width by height metres centred on the entity in its local xy plane, facing along its local -Z (aim it with transform.lookAt, as a spot), emitting color * intensity from its front, or from both faces with twoSided. Shaded with linearly transformed cosines (Heitz et al. 2016, three.js's tables), so its highlight on a glossy floor is the rectangle's shape and its light on a matte wall falls off as a real panel's does. With castShadow it casts a soft shadow in the frame; og bake lightmaps traces its exact soft shadows into the bounce light either way.
   * Area lights are clustered with the point and spot lights and share their MAX_LIGHTS budget; each reaches as far as its brightness carries (where it would light a white surface facing it at one hundredth of one), and fades smoothly to zero there.
   */
  const RectLight: ComponentType<{ readonly color: FieldSpec<"Color">; readonly intensity: FieldSpec<"f32">; readonly width: FieldSpec<"f32">; readonly height: FieldSpec<"f32">; readonly twoSided: FieldSpec<"bool">; readonly castShadow: FieldSpec<"bool"> }, RectLight>;
  /**
   * A box of baked light probes (an irradiance volume): og bake lightmaps traces the light arriving from the scene (bounced off its static models, from the sky, from emissive surfaces) at a grid of points spacing metres apart inside it, as first-order spherical harmonics. Every mesh, model, skinned character and particle-lit surface drawn inside it that has no lightmap of its own takes its ambient light from the probes instead of the sky's, blended at the box's edge over one spacing: a ball rolling past a red wall turns red on that side.
   * The box is size metres centred on the entity, axis-aligned in world space (its rotation and scale are ignored). Its probes are the scene's: unbaked (no bake since the volume was added, moved or resized) it lights nothing and og doctor --project names the scene.
   */
  const LightVolume: ComponentType<{ readonly size: FieldSpec<"Vector3">; readonly spacing: FieldSpec<"f32"> }, LightVolume>;
  /**
   * What is drawn: meshes with materials. Draws are instanced by (geometry, material) automatically; there is no per-object draw call.
   */
  namespace render {
    /**
     * Draw a ribbon this frame. Systems and commands may call it; the list is cleared at every tick.
     *
     * @param world The world whose renderer draws it.
     * @param desc The points and the look.
     * @returns Refused for a dead texture or fewer than two points.
     */
    function ribbon(world: World, desc?: RibbonDesc): Result<void>;
    /**
     * Draw a sprite this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the sprite should show.
     *
     * @param world The world whose renderer draws it.
     * @param desc The texture, placement, region and colour.
     * @returns Refused for a dead texture.
     */
    function sprite(world: World, desc?: SpriteDesc): Result<void>;
    /**
     * Spawn a floating text (damage numbers, +coins, `blocked`). It lives `lifetime` seconds of scaled time on its own; systems and commands may call it.
     *
     * @param world The world whose renderer draws it.
     * @param desc The text, the point and the style.
     * @returns Refused for a dead font, or no font when the UI has none.
     */
    function floatingText(world: World, desc?: FloatingTextDesc): Result<void>;
    /**
     * The size text takes when og.render.text draws it, for centring, wrapping and layout.
     *
     * @param world The world whose renderer owns the font.
     * @param desc The font, text and size.
     * @returns Refused for a dead font.
     */
    function measureText(world: World, desc?: TextMeasureDesc): Result<TextMetrics>;
    /**
     * Make a box geometry for og.Mesh. Physics can use it as a convex or triangle-mesh Collider.mesh too.
     *
     * @param world The world whose renderer owns the geometry.
     * @param desc Its extents and texture repeat.
     */
    function boxGeometry(world: World, desc?: BoxGeometryDesc): Result<Geometry>;
    /**
     * Make a plane geometry for og.Mesh. Physics can use it as a triangle-mesh Collider.mesh too.
     *
     * @param world The world whose renderer owns the geometry.
     * @param desc Its extents and texture repeat.
     */
    function planeGeometry(world: World, desc?: PlaneGeometryDesc): Result<Geometry>;
    /**
     * Make a sphere geometry for og.Mesh (give its body a sphere Collider, not this mesh).
     *
     * @param world The world whose renderer owns the geometry.
     * @param desc Its radius, tessellation and texture repeat.
     */
    function sphereGeometry(world: World, desc?: SphereGeometryDesc): Result<Geometry>;
    /**
     * Create a standard material.
     *
     * @param world The world whose renderer owns the material.
     * @param desc The parameters; omitted ones take three.js defaults.
     */
    function standardMaterial(world: World, desc?: StandardMaterialDesc): Result<Material>;
    /**
     * Change some parameters of a standard material.
     *
     * @param world The world whose renderer owns the material.
     * @param material The material.
     * @param desc The parameters to change; omitted ones keep their values.
     * @returns Refused for a dead material.
     */
    function updateMaterial(world: World, material: Material, desc?: StandardMaterialDesc): Result<void>;
    /**
     * Create a standard material that draws with a material shader's hooks (vertex_offset, surface_color, surface_normal, surface_emissive; the og-game skill's Material shaders section has their WGSL contract), its parameters at their defaults. The hooks apply to skinned meshes too, and shadows and the depth prepass (ambient occlusion, outlines, depth of field) see the mesh where vertex_offset moved it and cut where surface_color's alpha falls below alphaTest.
     *
     * @param world The world whose renderer owns the material.
     * @param shader The shader.
     * @param desc The base material; omitted parameters take three.js defaults. A dissolve sets alphaTest so that alpha below it is cut.
     * @returns Refused for a shader not loaded in this world.
     */
    function shaderMaterial(world: World, shader: Shader, desc?: StandardMaterialDesc): Result<Material>;
    /**
     * Give an existing material a material shader, or take it away with null: a loaded model's material (GltfInfo.materials) draws its skinned or static meshes through the hooks, with the shader's parameters at their defaults. Every mesh drawn with that material changes.
     *
     * @param world The world whose renderer owns the material.
     * @param material The material, from og.render.standardMaterial, og.render.shaderMaterial or a loaded glTF.
     * @param shader The shader, or null to draw the material plain again.
     * @returns Refused for a dead material or a shader not loaded in this world.
     */
    function setMaterialShader(world: World, material: Material, shader: Shader | null): Result<void>;
    /**
     * Set a parameter of a shader material, by the name its shader declares: a number for float, two to four for vec2, vec3, vec4 and color.
     *
     * @param world The world whose renderer owns the material.
     * @param material A material made by og.render.shaderMaterial or given a shader by og.render.setMaterialShader.
     * @param name The parameter's name.
     * @param value Its value; missing components are 0.
     * @returns Refused for a material without a shader or a parameter its shader does not declare.
     */
    function setShaderParam(world: World, material: Material, name: string, value: readonly number[]): Result<void>;
    /**
     * Apply a look, in GameDesc.start (or name it in GameDesc.look) or at any tick for a level's mood, a flashback or a photo mode: every resource the look names is replaced whole (the fields it sets take its values, the rest their defaults; PostEffects.flash and PostEffects.shakeTrauma, which game code animates, keep theirs), and the UI is configured with its style, keeping the rest of the last UiDesc. Resources it does not name are left as they are. Inside a system, the system must declare those resources among its writes. A look that switches the stylisation (a Stylise pass on or off, the pixel grid, retro 3D, the material style) starts TAA's history over, as a camera cut does, so its first frame holds no ghost of the old look.
     *
     * @param world The world whose look changes.
     * @param look The look, from og.load.look.
     * @returns Refused for a look not loaded in this world, one in a streaming group that is not resident (the reason names it), or a resource the system did not declare.
     */
    function applyLook(world: World, look: Look): Result<void>;
    /**
     * The look last applied (by og.render.applyLook or GameDesc.look), as og.load.lookInfo describes it; null before any.
     *
     * @param world The world asked.
     */
    function look(world: World): LookInfo | null;
    /**
     * The renderer's tier and limits on this device.
     *
     * @param world The world whose renderer is asked.
     */
    function capabilities(world: World): RenderCapabilities;
    /**
     * Draw text this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the text should show. Multi-page fonts are drawn page by page with one layout.
     *
     * @param world The world whose renderer draws it.
     * @param desc The font, text, placement and colour.
     * @returns Refused for a dead font.
     */
    function text(world: World, desc?: TextDesc): Result<void>;
    /**
     * Make a geometry from attribute arrays (three.js BufferGeometry): procedural levels, 2D polygons, meshes a tool made. It draws as any geometry (og.Mesh), its BVH is built for og.render.raycast, and (desc.physics) it is a Collider.mesh at once.
     *
     * @param world The world whose renderer owns the geometry.
     * @param desc The attributes.
     * @returns Refused for an attribute whose length does not fit the vertex count, an index past the vertices, or a number that is not finite (the reason names the attribute).
     */
    function bufferGeometry(world: World, desc?: BufferGeometryDesc): Result<Geometry>;
    /**
     * Rewrite a dynamic geometry's attributes in place (desc as og.render.bufferGeometry's, every attribute given again; `dynamic` and `physics` keep their values from when it was made). The same vertex and index counts rewrite its GPU buffers in place; other counts reallocate them, which costs more and is fine now and then. Its BVH is rebuilt, and its physics mesh re-registered (colliders made from it are rebuilt at the next step). Every og.Mesh, og.Lines and og.Points showing it changes.
     *
     * @param world The world whose renderer owns the geometry.
     * @param geometry A geometry og.render.bufferGeometry made with dynamic true.
     * @param desc The new attributes.
     * @returns Refused for a dead or static geometry, or attributes og.render.bufferGeometry refuses.
     */
    function updateGeometry(world: World, geometry: Geometry, desc?: BufferGeometryDesc): Result<void>;
    /**
     * A geometry's counts and bounds.
     *
     * @param world The world whose renderer owns the geometry.
     * @param geometry Any geometry: made, a primitive or a loaded glTF's.
     * @returns Refused for a dead geometry.
     */
    function geometryInfo(world: World, geometry: Geometry): Result<GeometryInfo>;
    /**
     * Make a cylinder geometry for og.Mesh (give its body a cylinder-like Collider: a capsule, a convex Collider.mesh of it, or the geometry itself).
     *
     * @param world The world whose renderer owns the geometry.
     * @param desc Its radii, height and tessellation.
     */
    function cylinderGeometry(world: World, desc?: CylinderGeometryDesc): Result<Geometry>;
    /**
     * Make a cone geometry for og.Mesh. Physics can use it as a convex Collider.mesh.
     *
     * @param world The world whose renderer owns the geometry.
     * @param desc Its radius, height and tessellation.
     */
    function coneGeometry(world: World, desc?: ConeGeometryDesc): Result<Geometry>;
    /**
     * Make a capsule geometry for og.Mesh (its body's Collider is a capsule of the same radius and half the height).
     *
     * @param world The world whose renderer owns the geometry.
     * @param desc Its radius, length and tessellation.
     */
    function capsuleGeometry(world: World, desc?: CapsuleGeometryDesc): Result<Geometry>;
    /**
     * Make a torus geometry for og.Mesh. Physics can use it as a triangle-mesh Collider.mesh.
     *
     * @param world The world whose renderer owns the geometry.
     * @param desc Its radii and tessellation.
     */
    function torusGeometry(world: World, desc?: TorusGeometryDesc): Result<Geometry>;
    /**
     * Make a ring geometry for og.Mesh (one-sided: give it a double-sided material to see it from behind).
     *
     * @param world The world whose renderer owns the geometry.
     * @param desc Its radii and tessellation.
     */
    function ringGeometry(world: World, desc?: RingGeometryDesc): Result<Geometry>;
    /**
     * Make a circle geometry for og.Mesh (one-sided, as ringGeometry).
     *
     * @param world The world whose renderer owns the geometry.
     * @param desc Its radius and tessellation.
     */
    function circleGeometry(world: World, desc?: CircleGeometryDesc): Result<Geometry>;
    /**
     * Make a tube along a curve through points for og.Mesh: pipes, cables, rails, a path shown as a ribbon of geometry. Physics can use it as a triangle-mesh Collider.mesh.
     *
     * @param world The world whose renderer owns the geometry.
     * @param desc The points, radius and tessellation.
     * @returns Refused for fewer than two points or a point that is not finite.
     */
    function tubeGeometry(world: World, desc?: TubeGeometryDesc): Result<Geometry>;
    /**
     * Draw lines this frame. Systems and commands may call it; the list is cleared at every tick, so submit them every tick they should show (og.Lines keeps a static set).
     *
     * @param world The world whose renderer draws them.
     * @param desc The points and the look.
     * @returns Refused for fewer than two points or a colours list whose length is not the points'.
     */
    function lines(world: World, desc?: LinesDesc): Result<void>;
    /**
     * Draw a point cloud this frame. Systems and commands may call it; the list is cleared at every tick.
     *
     * @param world The world whose renderer draws it.
     * @param desc The positions and the look.
     * @returns Refused for a colours list whose length is not the points'.
     */
    function points(world: World, desc?: PointsDesc): Result<void>;
    /**
     * Every drawn triangle a ray crosses, nearest first, as three.js Raycaster.intersectObjects: what is drawn, not the colliders (og.physics.raycast answers those). It tests og.Mesh and og.Model entities at their world transforms through a BVH per geometry (built when the geometry is made, loaded or updated), both faces of each triangle, skinned models in their bind pose. Read the world as of the call; inside a system, its transforms as earlier systems left them.
     *
     * @param world The world asked.
     * @param ray The ray; its direction need not be normalised and must not be zero.
     * @param desc Reach, nearest only and layers.
     * @returns Refused for a zero or non-finite direction.
     */
    function raycast(world: World, ray: Ray, desc?: RaycastGeometryDesc): Result<readonly GeometryHit[]>;
    /**
     * Make a render target: a texture an og.CameraView or og.Mirror draws into, its colour (RenderTargetInfo.texture) usable by any map, sprite or UI image, as three.js WebGLRenderTarget and its texture. Targets are drawn before the frame, so a material shows this frame's image; one a camera draws while it sees itself (a monitor in the room it films) shows the frame before.
     *
     * @param world The world whose renderer owns it.
     * @param desc Its size, format and depth.
     * @returns Refused for a side past RenderCapabilities.maxRenderTargetSize.
     */
    function renderTarget(world: World, desc?: RenderTargetDesc): Result<RenderTargetInfo>;
    /**
     * Set a parameter of a custom post pass, by the name its asset declares: a number for float, two to four for vec2, vec3, vec4 and color. Every frame it runs in reads it.
     *
     * @param world The world whose renderer owns the pass.
     * @param post The pass, from og.load.post.
     * @param name The parameter's name.
     * @param value Its value; missing components are 0.
     * @returns Refused for a pass not loaded in this world or a parameter it does not declare.
     */
    function setPostParam(world: World, post: Post, name: string, value: readonly number[]): Result<void>;
    /**
     * The custom post passes PostEffects.custom1 to custom8 name that failed to compile on this device, each skipped (the frame draws as without it). The cook validates every pass for each back end, so this is a device's own refusal.
     *
     * @param world The world asked.
     */
    function postErrors(world: World): readonly PostError[];
    /**
     * The baked lighting of every scene instantiated in the world, in instantiation order: whether it is drawn, and why not. A scene with no static model and no LightVolume is not listed.
     *
     * @param world The world asked.
     */
    function bakedLighting(world: World): readonly BakedScene[];
  }
  /**
   * Draws a loaded glTF's default scene at the entity's world transform: every mesh node with its glTF material. Scene entities with a `model` line get one; world.instantiate of a glTF asset spawns one.
   * The glTF's materials are shared by every instance of it; `tint` and `flash` are this instance's own, carried with its transform to the lit shaders (one draw for every instance, whatever each shows), so one of a crowd can flash when struck.
   */
  const Model: ComponentType<{ readonly asset: FieldSpec<"Asset">; readonly tint: FieldSpec<"Color">; readonly flash: FieldSpec<"f32">; readonly flashColor: FieldSpec<"Color">; readonly castShadow: FieldSpec<"bool">; readonly receiveShadow: FieldSpec<"bool">; readonly visible: FieldSpec<"bool">; readonly lodBias: FieldSpec<"f32">; readonly layers: FieldSpec<"u32">; readonly static: FieldSpec<"bool"> }, Model>;
  /** A load completed. */
  const AssetReady: EventType<AssetReady>;
  /** A load failed. */
  const AssetFailed: EventType<AssetFailed>;
  /**
   * Asset loading from cooked output: `og cook` turns assets/ into cooked/ (.ogb blobs and the manifest cooked/.cook.ogt), and the runtime reads only cooked/.
   * Paths are asset ids: the path under assets/ (`models/ship.glb`, `scenes/main.ogt`); an `assets/` prefix is accepted. Every blob is checked against the manifest's type, schema version and cook stamp.
   * The game's bundle (`cooked`) is opened before the first tick and GameDesc.scene is spawned then. Loads from an open bundle complete at once on both hosts (the web host fetches the bundle's files before the engine starts); only og.load.bundle of another bundle can be pending, and it completes at a tick boundary.
   */
  namespace load {
    /**
     * Open another cooked bundle (the game's own `cooked` is open before the first tick). Its loads read through it once it is ready.
     *
     * @param world The world that owns the asset.
     * @param path The cooked directory, relative to the project (or the page), such as `dlc/cooked`.
     * @returns The bundle's handle: pending until its manifest is read, then ready or failed.
     */
    function bundle(world: World, path: string): Result<Asset>;
    /**
     * What an open bundle holds.
     *
     * @param world The world that owns the asset.
     * @param asset A bundle handle; the game's own bundle is og.load.bundle's first answer, also given by `defaultBundle`.
     * @returns Refused for a pending, failed or non-bundle asset.
     */
    function bundleInfo(world: World, asset: Asset): Result<BundleInfo>;
    /**
     * The game's own bundle, opened before the first tick from `cooked/`.
     *
     * @param world The world that owns the asset.
     * @returns Refused when cooked/ was missing or unreadable (run og cook); the reason names the path.
     */
    function defaultBundle(world: World): Result<Asset>;
    /**
     * Load a cooked glTF (.gltf or .glb), as three.js GLTFLoader.load: its meshes, materials and textures join the renderer once.
     * A MagicaVoxel `.vox` cooks to a glTF and loads here too: greedy-meshed, flat-shaded, its palette as vertex colours, z-up turned y-up, each model about its centre (MagicaVoxel's pivot, with a scene graph or without). Its import settings (`models/hut.vox.ogt`, `type import`) size and place it: `model { voxScale 0.1 voxOrigin bottom }` is 0.1 m a voxel (default 1) and the lowest voxel on y = 0 (default `center`, the editor's origin). A `.vox` with frames (MagicaVoxel's animation) cooks to clips that swap the frames' models, played by og.AnimationPlayer as any skinned clip: `voxFps 8` (default 10) and `voxClip walk 0 3` (frames 0 to 3; default one clip `frames` of every frame).
     *
     * @param world The world that owns the asset.
     * @param path The asset id, such as `models/ship.glb`.
     * @param desc Scene and shadow options.
     * @returns The handle, or why it was refused (an uncooked path, a cook stamp mismatch, no open bundle).
     */
    function gltf(world: World, path: string, desc?: GltfDesc): Result<Asset>;
    /**
     * Load a cooked scene (.ogt). world.instantiate spawns its entities: depth first, parents before children, each component table matched to a registered component by its reflected name (`Transform` is og.Transform) and migrated field by field; a field the scene lacks keeps its default, one the component lacks is dropped with a warning.
     *
     * @param world The world that owns the asset.
     * @param path The scene's asset id, such as `scenes/main.ogt`.
     */
    function scene(world: World, path: string): Result<Asset>;
    /**
     * Load a cooked MSDF font for og.render.text, or an icon set (an `.ogt` of `type icons`: SVG icons cooked into a distance-field atlas like a font) for UiDesc.icons.font.
     *
     * @param world The world that owns the font.
     * @param path The font's asset id, such as `fonts/NotoSansSC-Regular-subset.ttf`.
     */
    function font(world: World, path: string): Result<Font>;
    /**
     * A loaded font's metrics.
     *
     * @param world The world that owns the font.
     * @param font The font.
     * @returns Refused for a dead font.
     */
    function fontInfo(world: World, font: Font): Result<FontInfo>;
    /**
     * Load a cooked sound for the audio module.
     * A long sound (music, ambience) can be cooked in chunks of whole QOA frames: `sound { stream true chunk 4 }` in its `.wav.ogt` import settings (chunk in seconds, 1 to 60, default 4), and every sound whose QOA file would pass 1 MiB unless `stream false`. Its first chunk loads with it and the others follow the playheads (SoundInfo.chunks), so music is never resident whole; it plays sample for sample as the whole file would, loops and crossfades included. A sound cooked in chunks cannot join a streaming group.
     *
     * @param world The world that owns the sound.
     * @param path The sound's asset id, such as `audio/blip.wav`.
     */
    function sound(world: World, path: string): Result<Sound>;
    /**
     * A colour lookup table for PostEffects.lut, from a cooked `.cube` file (Adobe/Resolve 3D LUT; other sizes are resampled to 32). Loaded once per path. One in a streaming group's pack loads once its group is resident (refused before, naming the group); while the group is unloaded PostEffects.lut grades nothing with it, and it comes back under the same handle.
     *
     * @param world The world whose renderer owns it.
     * @param path The asset id, such as `luts/warm.cube`.
     * @returns Refused for a path not in the bundle (the reason says to run og cook).
     */
    function lut(world: World, path: string): Result<Lut>;
    /**
     * A prefiltered environment map for Environment.map, from a cooked `.ogt` of `type environment`: `source` an equirectangular Radiance `.hdr` or OpenEXR `.exr` (the centre column faces +X, as three.js maps it) or a KTX2 cube map (six square faces +X -X +Y -Y +Z -Z with rows from the top; RGBA16F, RGB9E5, BC6H and the other HDR formats, decoded at cook time), or `faces` with six `.hdr`/`.exr` images in that order; `size` level 0's face size (a power of two, 16 to 2048, default 128), `intensity`, `rotation` (degrees about +Y) and `samples`. The cook prefilters it into a cube of five GGX levels (roughness 0 to 1) in RGB9E5 and the diffuse irradiance as nine harmonics, so loading only uploads it; `prefiltered true` takes a KTX2 cube's own mip chain as those levels instead (one already GGX-prefiltered by cmgen, IBLBaker or the glTF IBL sampler: level k of L at roughness k / (L - 1)), at the file's own size, without filtering it again. On the web it is fetched with the bundle like any blob; one in a streaming group's pack loads when its group is resident, and the sky lights the scene until then. Loaded once per path.
     *
     * @param world The world whose renderer owns it.
     * @param path The asset id, such as `env/studio.ogt`.
     * @returns Refused for a path not in the bundle (the reason says to run og cook) or not an environment map.
     */
    function environment(world: World, path: string): Result<EnvironmentMap>;
    /**
     * A particle emitter asset for ParticleSystem.emitter, from a cooked `.ogt` of `type particles` (the og-game skill's Effects section gives every line with its default and an example; og-cook's fx module parses it): shape, rate and bursts, lifetime, speed, size and colour over life, gravity, drag, noise, a texture and sprite sheet, blending, collision, billboard or 2D sprite rendering. A sprite-mode emitter may draw from a texture array instead (`array "<id>" [<layers>] [off | life <cycles> | fps <n> | random]`: each particle shows a layer by its age, at a rate, or one at random for its life; one draw per array whatever the layers; `round true` fades each particle to the soft dot's round edge). The textures and the array it names are loaded with it; while the array's streaming group is not resident its systems draw nothing. Loaded once per path.
     *
     * @param world The world whose renderer owns it.
     * @param path The asset id, such as `fx/sparks.ogt`.
     * @returns Refused for a path not in the bundle, or a texture or texture array it cannot load.
     */
    function emitter(world: World, path: string): Result<ParticleEmitter>;
    /**
     * A material shader for og.render.shaderMaterial, from a cooked `.ogt` of `type shader` (its WGSL compiled by the cook for every back end). `mode hooks` (the default) defines hooks of the lit shader; `mode full lit` or `mode full unlit` defines `fn vertex(v: VertexInput, model: mat4x4<f32>, p: Params) -> VertexOutput` and `fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput` over the binding contract (camera(), the model with its skin, Params, tex0 to tex7 for the material's eight texture slots; the og-game skill's Full custom shaders section), lit by the frame's lights or drawn as returned, the prepass and every shadow pass drawn through the same entries. Loaded once per path.
     *
     * @param world The world whose renderer owns it.
     * @param path The asset id, such as `shaders/dissolve.ogt`.
     * @returns Refused for a path not in the bundle or not a shader.
     */
    function shader(world: World, path: string): Result<Shader>;
    /**
     * A custom post pass for PostEffects.custom1 to custom8, from a cooked `.ogt` of `type post`: `code` or `source` (a `.wgsl` beside it) defining `fn post(f: PostInput, p: Params) -> vec4<f32>` and `param <name> float|vec2|vec3|vec4|color <default...>` lines (at most eight). The function reads previous(uv), scene_color(uv), depth(uv), normal(uv) and f.uv, f.pixel, f.size, f.time, and returns linear HDR colour (the og-game skill's Custom post passes section has the contract). The cook compiles it for every back end. Loaded once per path.
     *
     * @param world The world whose renderer owns it.
     * @param path The asset id, such as `post/crt.ogt`.
     * @returns Refused for a path not in the bundle or not a post pass.
     */
    function post(world: World, path: string): Result<Post>;
    /**
     * Load a cooked texture for og.render.sprite and og.ui.image, as three.js TextureLoader.load.
     * Sources are PNG, JPEG or KTX2 files under assets/ (`texture { kind color|normal|linear|pixel }` in `<file>.ogt`). A KTX2's `KTXorientation` and `KTXswizzle` are honoured: one that is not `rd` and `rgba` is decoded, flipped to rows top to bottom and swizzled, and cooked again; one that is (Basis ETC1S or UASTC, or RGBA8, of a colour or linear kind) is taken as it is.
     *
     * @param world The world whose renderer owns the texture.
     * @param path The texture's asset id, such as `textures/stone.png`.
     */
    function texture(world: World, path: string): Result<Texture>;
    /**
     * Load a cooked texture array (an `.ogt` of `type texture-array`: `kind color|normal|linear|pixel` and one `layer "<file>"` per layer, 1 to 256 images of one size, each path beside the file or under assets/) and upload it as one 2D-array texture, its layers in file order, each in the encoding the device takes for its kind. Each layer is also a texture of its own, `<path>#layer<n>`, for og.load.texture. Loading the same path again answers the same array.
     * SpriteDesc.array (world and screen sprites), og.Sprite.array and 2D particle emitters (an emitter's `array` line) draw from its layers, one draw per array whatever layer each shows, with glow and the additive blend; an `array true` flipbook sheet cooks its frames into one (materials and terrain layers do not sample arrays yet). TextureArrayInfo and the frame's texture bytes report it. One in a streaming group's pack loads once its group is resident (refused before, naming the group); when the group unloads the renderer lets it go, og.render.sprite of it is refused and og.Sprite and particles of it draw nothing until the group is resident again, when it comes back under the same handle.
     *
     * @param world The world whose renderer owns the array.
     * @param path The array's asset id, such as `tiles/ground.ogt`.
     * @returns Refused for a path not in the bundle (run og cook), not a texture array, or layers that do not agree.
     */
    function textureArray(world: World, path: string): Result<TextureArrayInfo>;
    /**
     * Load a cooked UI style (an `.ogt` of `type style`: every palette role, the radii, type ramp, durations, and its art: the atlas and its nine-slice frames, ornaments, fonts, icons, motion and sounds; the grammar is og-ui's `parse_style`) for UiDesc.style. Its atlas, fonts, icons and sounds are loaded with it. Loaded once per path. A built-in style a look names is in the bundle as `@styles/<name>/style.ogt`.
     *
     * @param world The world whose UI uses it.
     * @param path The asset id, such as `styles/pixel/style.ogt` or `@styles/pixel/style.ogt`.
     * @returns Refused for a path not in the bundle, a style that does not parse (the reason names the line) or an asset it names that cannot load.
     */
    function style(world: World, path: string): Result<Style>;
    /**
     * Load a look: an `.ogt` of `type look` binding a frame's art direction as data. Its top-level `name`, `extends` (a built-in preset's slug, such as `anime-cel-3d`, or another look's asset id: the game's own `assets/looks/<slug>.ogt` wins over the built-in preset of that slug), `ui` (a UI style: a built-in style's slug, such as `cartoon`, or the game's own style file's asset id) and `antialias`, then one block per resource it sets (RenderSettings, PostEffects, Stylise, MaterialStyle, Fog, Sky, Environment) with that resource's fields by name, an asset id where a field is a handle (`lut "luts/warm.cube"`, `palette "palettes/dusk.hex"`). The cook validates every field against the resources' reflected schema and flattens `extends`, so the blob is the whole look. The textures, LUTs, palettes, environment map and UI style it names are loaded with it. Loaded once per path.
     *
     * @param world The world whose renderer owns it.
     * @param path The asset id, such as `looks/main.ogt`.
     * @returns Refused for a path not in the bundle (the reason says to run og cook) or an asset it names that cannot load.
     */
    function look(world: World, path: string): Result<Look>;
    /**
     * What a loaded look holds and sets.
     *
     * @param world The world that owns the look.
     * @param look The look.
     * @returns Refused for a look not loaded in this world.
     */
    function lookInfo(world: World, look: Look): Result<LookInfo>;
    /**
     * A colour palette for Stylise.palette, from a cooked `.hex` (one RRGGBB per line, as palette sites publish them) or GIMP `.gpl` file: up to 256 sRGB colours, in file order. Loaded once per path. One in a streaming group's pack loads once its group is resident; while the group is unloaded Stylise.palette quantises to no palette, and the palette comes back under the same handle.
     *
     * @param world The world whose renderer owns it.
     * @param path The asset id, such as `palettes/dusk.hex`.
     * @returns Refused for a path not in the bundle (the reason says to run og cook).
     */
    function palette(world: World, path: string): Result<Palette>;
    /**
     * A loaded sound's format and length.
     *
     * @param world The world that owns the sound.
     * @param sound The sound.
     * @returns Refused for a dead sound.
     */
    function soundInfo(world: World, sound: Sound): Result<SoundInfo>;
    /**
     * Where a load is.
     *
     * @param world The world that owns the asset.
     * @param asset The handle.
     */
    function status(world: World, asset: Asset): LoadStatus;
    /**
     * Why a load failed.
     *
     * @param world The world that owns the asset.
     * @param asset The handle.
     * @returns The reason, naming the fix; null unless the load failed.
     */
    function error(world: World, asset: Asset): string | null;
    /**
     * What a ready glTF holds.
     *
     * @param world The world that owns the asset.
     * @param asset A ready glTF handle.
     * @returns Refused for a pending, failed or non-glTF asset.
     */
    function gltfInfo(world: World, asset: Asset): Result<GltfInfo>;
    /**
     * Load a cooked glTF's animation (its `<model>#anim` blob: the rig, the clips, the skin and morph streams) and register its skeleton and clips with og.anim. Loading the same model again answers the same handles.
     * Clips are cooked as f32 samples, or with `animation { compress true tolerance 0.0001 }` in the model's import settings as quantised keys (rotations as 48-bit smallest-three, translations, scales and weights as 16 bits over each channel's range) with every frame dropped that interpolation rebuilds within the tolerance; either way every frame is rebuilt here and the clip plays as f32. Morph targets are cooked sparse (the vertices each moves).
     *
     * @param world The world whose animation owns the assets.
     * @param path The model's asset id, such as `models/walker.glb` (a `#anim` suffix is accepted).
     * @returns Refused for an uncooked path, a model the cook wrote no `#anim` blob for (no skins, clips or morph targets), a cook stamp mismatch or an invalid rig.
     */
    function animation(world: World, path: string): Result<Animation>;
    /**
     * Load a cooked humanoid map (an `.ogt` of `type rig`: a naming scheme and bones named outright) for og.anim.retarget and og.anim.humanoid. Loading the same path again answers the same handle.
     *
     * @param world The world whose animation uses it.
     * @param path The rig's asset id, such as `rigs/strider.ogt`.
     * @returns Refused for a path not in the bundle (run og cook) or a cook stamp mismatch.
     */
    function rig(world: World, path: string): Result<Rig>;
    /**
     * Load a cooked flipbook sheet (an `.ogt` of `type flipbook`, with its Aseprite or TexturePacker atlas resolved by the cook) and its texture, and register it with og.flipbook. A sheet with `array true` is cooked into a texture array instead (its frames, one size, cut out of the texture as layers): its sprites draw frame n as layer n, every sprite of the sheet one draw whichever frame each shows, and FlipbookInfo.array names the array. A sheet in a streaming group (its array and layers with it) loads once its group is resident (refused naming the group before), and its sprites draw nothing while the group is away. Loading the same path again answers the same handles.
     *
     * @param world The world whose animation and renderer own it.
     * @param path The sheet's asset id, such as `sprites/hero.ogt`.
     * @returns Refused for a path not in the bundle (run og cook) or a texture it cannot load.
     */
    function flipbook(world: World, path: string): Result<FlipbookInfo>;
    /**
     * Load a cooked map (a tilemap `.ogt`, a Tiled `.tmj` or an LDtk `.ldtk`), its tilesets and their textures. Loading the same path again answers the same handles.
     *
     * @param world The world whose renderer and tilemaps own it.
     * @param path The map's asset id, such as `maps/room.tmj`.
     * @returns Refused for a path not in the bundle, a missing tileset or texture, or a terrain its tileset lacks.
     */
    function tilemap(world: World, path: string): Result<TilemapInfo>;
    /**
     * Load cooked footage (an `.ogt` of `type video`: `source` an .mp4, .webm or .mov beside it or under assets/, `fps`, `maxSeconds`, `size W H`, `kind color|pixel`, `audio`), which og cook turns with ffmpeg into a texture array of its frames and its sound track. Uploads the frames as one 2D-array texture and makes the video's Texture (VideoInfo.texture), which shows the frame its og.VideoPlayer is at: a material map, an og.Sprite or a UI image draws it like any texture. Load it in GameDesc.start (every process of an og render --jobs run loads it the same way, so its handles agree). Loading the same path again answers the same handles.
     *
     * @param world The world whose renderer and audio own it.
     * @param path The video's asset id: the `.ogt`, such as `clips/intro.ogt`.
     * @returns Refused for a path not in the bundle (run og cook: a video needs ffmpeg there, og doctor) or not a video.
     */
    function video(world: World, path: string): Result<VideoInfo>;
    /**
     * A loaded video's size, frames, rate, length, texture and sound.
     *
     * @param world The world that loaded it.
     * @param video The video.
     * @returns Refused for a handle og.load.video did not answer.
     */
    function videoInfo(world: World, video: Video): Result<VideoInfo>;
    /**
     * Load a timeline: an `.ogt` of `type timeline` (the og.timeline module's file: duration, paths and tracks of keys), cooked with its caption files imported. Loaded once per path; play it with an og.Timeline whose asset is the handle, or `timeline:play:<name>` on the command channel.
     *
     * @param world The world.
     * @param path The timeline's asset id, such as `timelines/intro.ogt`.
     * @returns Refused for a path not in the bundle (run og cook) or a file that is not a timeline.
     */
    function timeline(world: World, path: string): Result<TimelineAsset>;
    /**
     * Load a cooked 2D rig (an `.ogt` of `type rig2d`, written by hand or imported by the cook from DragonBones JSON) and its texture, and register it with og.skeleton2d. Loading the same path again answers the same handles.
     *
     * @param world The world whose animation and renderer own it.
     * @param path The rig's asset id, such as `rigs/hero.ogt`.
     * @returns Refused for a path not in the bundle (run og cook) or a texture it cannot load.
     */
    function rig2d(world: World, path: string): Result<Rig2DInfo>;
  }
  /**
   * A body the physics step simulates. It needs an og.Transform and usually has an og.Collider; a dynamic body without an og.Velocity gets one.
   * Changing a RigidBody, a Collider or Transform.scale rebuilds the body (it loses its contacts' warm starts); a Transform or Velocity that game code changed moves it (a teleport).
   */
  const RigidBody: ComponentType<{ readonly kind: FieldSpec<"u32">; readonly mass: FieldSpec<"f32">; readonly linearDamping: FieldSpec<"f32">; readonly angularDamping: FieldSpec<"f32">; readonly gravityScale: FieldSpec<"f32">; readonly lock: FieldSpec<"u32">; readonly ccd: FieldSpec<"u32">; readonly centerOfMass: FieldSpec<"Vector3"> }, RigidBody>;
  /**
   * The shape an entity collides with, attached to its RigidBody (or, without one, a static collider at its Transform). Sizes are scaled by og.Transform.scale.
   */
  const Collider: ComponentType<{ readonly shape: FieldSpec<"u32">; readonly size: FieldSpec<"Vector3">; readonly radius: FieldSpec<"f32">; readonly height: FieldSpec<"f32">; readonly mesh: FieldSpec<"Geometry">; readonly offset: FieldSpec<"Vector3">; readonly rotation: FieldSpec<"Quaternion">; readonly friction: FieldSpec<"f32">; readonly restitution: FieldSpec<"f32">; readonly density: FieldSpec<"f32">; readonly sensor: FieldSpec<"u32">; readonly layers: FieldSpec<"u32">; readonly mask: FieldSpec<"u32"> }, Collider>;
  /**
   * Linear and angular velocity. Written for dynamic bodies every tick (added to them by the step when missing); a kinematic body moves by it each tick (never added, never written). Set it to launch or stop a body; push one with og.physics.applyForce, applyImpulse, applyTorque or applyTorqueImpulse.
   */
  const Velocity: ComponentType<{ readonly linear: FieldSpec<"Vector3">; readonly angular: FieldSpec<"Vector3"> }, Velocity>;
  /**
   * The physics settings of the world. The profile is the project's; the same components serve both profiles.
   */
  const Physics: ResourceType<{ readonly profile: FieldSpec<"u32">; readonly gravity: FieldSpec<"Vector3">; readonly substeps: FieldSpec<"u32">; readonly solverIterations: FieldSpec<"u32">; readonly restSpeed: FieldSpec<"f32"> }, Physics>;
  /** Two solid colliders started touching during the tick. */
  const CollisionStarted: EventType<CollisionStarted>;
  /** Two solid colliders stopped touching, or one of them went away. */
  const CollisionEnded: EventType<CollisionEnded>;
  /** A collider started overlapping a sensor. */
  const SensorEntered: EventType<SensorEntered>;
  /** A collider stopped overlapping a sensor, or one of them went away. */
  const SensorExited: EventType<SensorExited>;
  /**
   * One pair in contact at the end of a step, sent every tick for every touching pair: how a game asks what is touching what right now.
   */
  const Contact: EventType<Contact>;
  /**
   * A joint between two bodies (or a body and a point of the world). The component may sit on one of the bodies (`body` 0) or on an entity of its own, so one body can have several joints. Anchors are in each body's frame (rotated and moved with it, not scaled); without `other` the second anchor is a world point. A joint whose bodies are not both `og.RigidBody` entities does nothing until they are.
   */
  const Joint: ComponentType<{ readonly kind: FieldSpec<"u32">; readonly body: FieldSpec<"Entity">; readonly other: FieldSpec<"Entity">; readonly anchor: FieldSpec<"Vector3">; readonly otherAnchor: FieldSpec<"Vector3">; readonly axis: FieldSpec<"Vector3">; readonly limits: FieldSpec<"bool">; readonly min: FieldSpec<"f32">; readonly max: FieldSpec<"f32">; readonly motor: FieldSpec<"u32">; readonly motorTarget: FieldSpec<"f32">; readonly motorStiffness: FieldSpec<"f32">; readonly motorDamping: FieldSpec<"f32">; readonly motorMaxForce: FieldSpec<"f32">; readonly restLength: FieldSpec<"f32">; readonly stiffness: FieldSpec<"f32">; readonly damping: FieldSpec<"f32">; readonly length: FieldSpec<"f32">; readonly collide: FieldSpec<"bool">; readonly value: FieldSpec<"f32"> }, Joint>;
  /**
   * A kinematic character (3D, or top-down 2D): the entity's capsule (or any og.Collider) moved by `move` each tick, sliding along walls, climbing slopes up to `maxSlope` and steps up to `stepHeight`, snapped to the ground, carried by the moving platform it stands on and falling by gravity. It keeps `skin` off every surface, standing that far over the ground; a step is climbed when the ground beyond it is at most `stepHeight` over the ground it stands on. What blocks it is every solid collider its og.Collider's `layers` and `mask` interact with (each one's layers meeting the other's mask), as for bodies. The entity needs an og.Transform and an og.Collider; the step makes it a kinematic body and pushes the dynamic bodies it walks into (`pushBodies`) with at most `pushStrength`, so a light one is shoved and a heavy one holds.
   */
  const CharacterController: ComponentType<{ readonly move: FieldSpec<"Vector3">; readonly jump: FieldSpec<"bool">; readonly jumpSpeed: FieldSpec<"f32">; readonly gravityScale: FieldSpec<"f32">; readonly maxSlope: FieldSpec<"f32">; readonly stepHeight: FieldSpec<"f32">; readonly snapDistance: FieldSpec<"f32">; readonly skin: FieldSpec<"f32">; readonly coyoteTime: FieldSpec<"f32">; readonly maxFallSpeed: FieldSpec<"f32">; readonly pushBodies: FieldSpec<"bool">; readonly pushStrength: FieldSpec<"f32">; readonly grounded: FieldSpec<"bool">; readonly sliding: FieldSpec<"bool">; readonly velocity: FieldSpec<"Vector3">; readonly groundNormal: FieldSpec<"Vector3">; readonly coyote: FieldSpec<"f32">; readonly jumpWasHeld: FieldSpec<"bool">; readonly ground: FieldSpec<"Entity"> }, CharacterController>;
  /**
   * A 2D platformer character (the 2D profile): run, jump and fall with designer units (jump height and time to apex, not gravity), coyote time, jump buffering, variable jump height (releasing jump while rising cuts the jump), apex hang, air jumps, wall slide and wall jump, one-way platforms (colliders on `oneWayLayers` are solid from above only; hold `down` to drop through), moving platforms (it rides what it stands on) and corner correction (a jump that clips a ceiling corner by less than `cornerCorrection` slides past it). It keeps `skin` off every surface; what blocks it is every solid collider its og.Collider's `layers` and `mask` interact with (each one's layers meeting the other's mask), as for bodies. The entity needs an og.Transform and an og.Collider (a box or capsule); it becomes a kinematic body and pushes the dynamic bodies it runs into (`pushBodies`) with at most `pushStrength`, so a crate's mass tells: a light one is shoved at the run speed, a heavy one holds (set its og.RigidBody `mass`).
   */
  const PlatformerController: ComponentType<{ readonly moveX: FieldSpec<"f32">; readonly jump: FieldSpec<"bool">; readonly down: FieldSpec<"bool">; readonly impulse: FieldSpec<"Vector2">; readonly maxSpeed: FieldSpec<"f32">; readonly acceleration: FieldSpec<"f32">; readonly deceleration: FieldSpec<"f32">; readonly airAcceleration: FieldSpec<"f32">; readonly airDeceleration: FieldSpec<"f32">; readonly jumpHeight: FieldSpec<"f32">; readonly timeToApex: FieldSpec<"f32">; readonly fallGravity: FieldSpec<"f32">; readonly jumpCut: FieldSpec<"f32">; readonly apexGravity: FieldSpec<"f32">; readonly apexThreshold: FieldSpec<"f32">; readonly maxFallSpeed: FieldSpec<"f32">; readonly coyoteTime: FieldSpec<"f32">; readonly jumpBuffer: FieldSpec<"f32">; readonly airJumps: FieldSpec<"u32">; readonly wallJumps: FieldSpec<"bool">; readonly wallSlideSpeed: FieldSpec<"f32">; readonly wallJump: FieldSpec<"Vector2">; readonly wallJumpLock: FieldSpec<"f32">; readonly oneWayLayers: FieldSpec<"u32">; readonly cornerCorrection: FieldSpec<"f32">; readonly maxSlope: FieldSpec<"f32">; readonly snapDistance: FieldSpec<"f32">; readonly skin: FieldSpec<"f32">; readonly pushBodies: FieldSpec<"bool">; readonly pushStrength: FieldSpec<"f32">; readonly grounded: FieldSpec<"bool">; readonly onWall: FieldSpec<"i32">; readonly facing: FieldSpec<"i32">; readonly velocity: FieldSpec<"Vector3">; readonly jumping: FieldSpec<"bool">; readonly airJumpsLeft: FieldSpec<"u32">; readonly coyote: FieldSpec<"f32">; readonly buffer: FieldSpec<"f32">; readonly wallLock: FieldSpec<"f32">; readonly dropTime: FieldSpec<"f32">; readonly jumpWasHeld: FieldSpec<"bool">; readonly ground: FieldSpec<"Entity"> }, PlatformerController>;
  /**
   * Something a CharacterController or PlatformerController did this step: the hooks for dust, squash, sounds and camera kicks. Read the next tick: the controllers move in the physics step, after the game's systems, so a jump pressed in tick T moves the character in T and its CHARACTER_JUMPED is read in T + 1 (og-feel §12 has the same-tick alternative).
   */
  const CharacterEvent: EventType<CharacterEvent>;
  /**
   * A car on a dynamic chassis (the entity's og.RigidBody and og.Collider). 3D: a raycast car with four wheels (suspension, grip, steering on the front axle); its forward is the chassis's local +z (three.js Object3D.lookAt aims +z). 2D: two wheel bodies on sprung wheel joints (front at +x), driven by a motor. The wheels are not entities: og.physics.vehicleWheels gives their poses (and each tyre's slip) for drawing, smoke and skid marks.
   * The defaults drive a road car of about 1,000 kg: give the chassis that mass (og.RigidBody.mass 1000), since a car-sized box at the collider's density 1 weighs a few kilograms and the engine's and brakes' newtons would throw it. The suspension and the grip scale with the chassis's mass (Bullet's units), the engine and the brakes do not: a heavier car wants proportionally more engineForce and brakeForce. og-feel §12 has the numbers a game starts from.
   * The wheel rays meet what the chassis's collider would (its og.Collider layers against the other's mask, and the other's layers against its mask), so two cars on layers that do not meet (a ghost beside the player) pass through each other.
   */
  const Vehicle: ComponentType<{ readonly throttle: FieldSpec<"f32">; readonly steer: FieldSpec<"f32">; readonly brake: FieldSpec<"f32">; readonly wheelBase: FieldSpec<"f32">; readonly track: FieldSpec<"f32">; readonly wheelRadius: FieldSpec<"f32">; readonly mountHeight: FieldSpec<"f32">; readonly suspensionRest: FieldSpec<"f32">; readonly suspensionTravel: FieldSpec<"f32">; readonly suspensionStiffness: FieldSpec<"f32">; readonly suspensionDamping: FieldSpec<"f32">; readonly grip: FieldSpec<"f32">; readonly engineForce: FieldSpec<"f32">; readonly maxSpeed: FieldSpec<"f32">; readonly brakeForce: FieldSpec<"f32">; readonly maxSteer: FieldSpec<"f32">; readonly drive: FieldSpec<"u32">; readonly wheelMass: FieldSpec<"f32">; readonly rollInfluence: FieldSpec<"f32">; readonly speed: FieldSpec<"f32">; readonly contacts: FieldSpec<"u32">; readonly slip: FieldSpec<"f32"> }, Vehicle>;
  /**
   * A Verlet rope: `segments` links from the entity's world position (its og.Transform after its parents'; plus `offset`) to `end` (an entity, plus `endOffset`) or hanging free, swung by gravity and wind, colliding with every solid collider at `radius`. It moves nothing (one-way: a rope joint ties bodies together, og.Joint JOINT_ROPE). Its points are world state (the og.physics state block); og.physics.ropePoints reads them and the renderer draws the rope as a strip of `width` and `color`.
   */
  const Rope: ComponentType<{ readonly segments: FieldSpec<"u32">; readonly length: FieldSpec<"f32">; readonly offset: FieldSpec<"Vector3">; readonly end: FieldSpec<"Entity">; readonly endOffset: FieldSpec<"Vector3">; readonly iterations: FieldSpec<"u32">; readonly damping: FieldSpec<"f32">; readonly gravityScale: FieldSpec<"f32">; readonly wind: FieldSpec<"Vector3">; readonly radius: FieldSpec<"f32">; readonly width: FieldSpec<"f32">; readonly color: FieldSpec<"Vector3">; readonly visible: FieldSpec<"bool"> }, Rope>;
  /**
   * A Verlet cloth: a `columns` × `rows` grid of points, `spacing` apart, hanging from the entity's world position (its og.Transform after its parents': its top-left corner, the grid laid out along the entity's local x and down -y), pinned by `pins` (pinned points follow the entity), with stretch and shear constraints, gravity, wind, collision against every solid collider and, with `thickness`, against itself. One-way like og.Rope; its points are world state. Drawn as a double-sided mesh with `material` (og.render.standardMaterial; 0: a plain cloth colour).
   */
  const Cloth: ComponentType<{ readonly columns: FieldSpec<"u32">; readonly rows: FieldSpec<"u32">; readonly spacing: FieldSpec<"f32">; readonly pins: FieldSpec<"u32">; readonly iterations: FieldSpec<"u32">; readonly damping: FieldSpec<"f32">; readonly gravityScale: FieldSpec<"f32">; readonly wind: FieldSpec<"Vector3">; readonly gust: FieldSpec<"f32">; readonly radius: FieldSpec<"f32">; readonly thickness: FieldSpec<"f32">; readonly substeps: FieldSpec<"u32">; readonly friction: FieldSpec<"f32">; readonly material: FieldSpec<"Material">; readonly visible: FieldSpec<"bool"> }, Cloth>;
  /**
   * A 2D particle fluid (any profile, in the xy plane): `count` particles spawned in a `size` box centred on the entity, simulated with double density relaxation (Clavet et al. 2005: pressure, near pressure, viscosity) on a fixed grid in a fixed order, so it is deterministic and part of the og.physics state block. Particles collide with every solid collider (pushed out of bodies; they push nothing back) and stay inside `bounds` around the entity when it is set. Fluids of one `group` interact: by `density` they layer, by `mixing` they blend. Drawn as metaballs: the particles blurred into a field the renderer thresholds into one surface of `color`, on sprite layer `layer`.
   */
  const Fluid2D: ComponentType<{ readonly count: FieldSpec<"u32">; readonly size: FieldSpec<"Vector2">; readonly bounds: FieldSpec<"Vector2">; readonly radius: FieldSpec<"f32">; readonly restDensity: FieldSpec<"f32">; readonly stiffness: FieldSpec<"f32">; readonly nearStiffness: FieldSpec<"f32">; readonly viscosity: FieldSpec<"f32">; readonly group: FieldSpec<"u32">; readonly density: FieldSpec<"f32">; readonly mixing: FieldSpec<"f32">; readonly gravityScale: FieldSpec<"f32">; readonly substeps: FieldSpec<"u32">; readonly seed: FieldSpec<"u32">; readonly color: FieldSpec<"Vector3">; readonly opacity: FieldSpec<"f32">; readonly blobScale: FieldSpec<"f32">; readonly threshold: FieldSpec<"f32">; readonly layer: FieldSpec<"i32">; readonly visible: FieldSpec<"bool"> }, Fluid2D>;
  /**
   * A 3D particle fluid (the 3D profile): `count` particles spawned in a `size` box centred on the entity, simulated with double density relaxation (Clavet et al. 2005: pressure, near pressure that gives surface tension, viscosity) over a spatial hash in a fixed order, so it is deterministic and part of the og.physics state block. Particles collide with every solid collider, the terrain's height field included (pushed out; they push nothing back), and stay inside `bounds` around the entity when it is set. An emitter adds particles at `emitRate` up to `maxCount`; a sink box removes those that enter it. Fluids of one `group` interact: by `density` they layer, by `mixing` they blend. Drawn as a screen-space surface: the particles' depth smoothed into one surface, refracting what is behind by its thickness and reflecting the sky, lit by the scene's lights. Cost: about 0.9 ms of CPU a tick for 2000 particles at 1 substep, 1.8 ms at 2 (native and wasm alike), growing with the particles; keep a scene's fluids to a few thousand particles. The surface costs about 1.1 ms of GPU (an Apple M4 Pro, the fluid covering a third of a 1280×720 frame) whatever the particle count, growing with the pixels it covers.
   */
  const Fluid3D: ComponentType<{ readonly count: FieldSpec<"u32">; readonly size: FieldSpec<"Vector3">; readonly bounds: FieldSpec<"Vector3">; readonly radius: FieldSpec<"f32">; readonly restDensity: FieldSpec<"f32">; readonly stiffness: FieldSpec<"f32">; readonly nearStiffness: FieldSpec<"f32">; readonly viscosity: FieldSpec<"f32">; readonly group: FieldSpec<"u32">; readonly density: FieldSpec<"f32">; readonly mixing: FieldSpec<"f32">; readonly gravityScale: FieldSpec<"f32">; readonly substeps: FieldSpec<"u32">; readonly seed: FieldSpec<"u32">; readonly maxCount: FieldSpec<"u32">; readonly emitRate: FieldSpec<"f32">; readonly emitOffset: FieldSpec<"Vector3">; readonly emitVelocity: FieldSpec<"Vector3">; readonly emitRadius: FieldSpec<"f32">; readonly sinkOffset: FieldSpec<"Vector3">; readonly sinkSize: FieldSpec<"Vector3">; readonly color: FieldSpec<"Vector3">; readonly opacity: FieldSpec<"f32">; readonly reflectivity: FieldSpec<"f32">; readonly refraction: FieldSpec<"f32">; readonly drawScale: FieldSpec<"f32">; readonly smoothing: FieldSpec<"f32">; readonly visible: FieldSpec<"bool"> }, Fluid3D>;
  /**
   * Height-field water (3D): a `resolution` × `resolution` grid of heights over a `size` square centred on the entity (its y is the rest level), waves travelling at `waveSpeed` and fading by `damping`. Dynamic bodies crossing it push waves (splashes; og.Splash events) and float by `buoyancy`. The heights are world state; an og.Water on the same entity draws them.
   */
  const WaterSurface: ComponentType<{ readonly resolution: FieldSpec<"u32">; readonly size: FieldSpec<"f32">; readonly waveSpeed: FieldSpec<"f32">; readonly damping: FieldSpec<"f32">; readonly splash: FieldSpec<"f32">; readonly buoyancy: FieldSpec<"f32">; readonly drag: FieldSpec<"f32"> }, WaterSurface>;
  /**
   * 2D water for side-scrollers: a `width` × `depth` pool whose top edge (at the entity's y, centred on its x) is `columns` springs, each pulled to its rest height by `tension`, damped by `damping` and passing waves to its neighbours by `spread`. Bodies (and characters) crossing the surface splash it (og.Splash events) and float by `buoyancy`. The springs are world state; an og.Water2D on the same entity draws it.
   */
  const WaterLine: ComponentType<{ readonly width: FieldSpec<"f32">; readonly depth: FieldSpec<"f32">; readonly columns: FieldSpec<"u32">; readonly tension: FieldSpec<"f32">; readonly damping: FieldSpec<"f32">; readonly spread: FieldSpec<"f32">; readonly splash: FieldSpec<"f32">; readonly buoyancy: FieldSpec<"f32">; readonly drag: FieldSpec<"f32"> }, WaterLine>;
  /**
   * Something crossed a water surface (og.WaterSurface or og.WaterLine): for splash particles and sounds. Read the next tick.
   */
  const Splash: EventType<Splash>;
  /**
   * A ragdoll over an og.Skeleton: a capsule body per bone with a parent, joints with limits between them (ball joints with cone limits in 3D; in 2D hinges about z, the bones rapier2d bodies in the xy plane at their animated z, for a skeleton laid out in that plane), built from the skeleton's rest pose. `weight` is where the pose comes from: 0 the animation (the bodies follow it kinematically), 1 the physics (the bodies are dynamic, starting with the animation's velocity); the pose drawn blends between them over `blendTime`, both ways. The bodies are world state (the og.physics state block), not entities.
   */
  const Ragdoll: ComponentType<{ readonly weight: FieldSpec<"f32">; readonly blendTime: FieldSpec<"f32">; readonly mass: FieldSpec<"f32">; readonly thickness: FieldSpec<"f32">; readonly jointLimit: FieldSpec<"f32">; readonly damping: FieldSpec<"f32">; readonly impulse: FieldSpec<"Vector3">; readonly blend: FieldSpec<"f32"> }, Ragdoll>;
  /**
   * Rigid bodies over rapier (rapier3d in the 3D profile, rapier2d in 2D, both with enhanced determinism): game code owns the components, the engine's og.physics system steps them after the game's systems and writes the dynamic bodies back.
   * The profile is the project's (`profile "2d"` or `"3d"` in project.ogt). In 2D, bodies move in the xy plane and rotate about z; a transform keeps its z. A collider without a RigidBody is static at its transform.
   * The rapier world persists between ticks and is world state: snapshots, restore and the world hash cover the whole simulation. Events sent in tick N are read in tick N + 1.
   */
  namespace physics {
    /**
     * The nearest collider along a ray. Inside a system it sees the colliders as they were at the start of the tick (og.physics.queries records them before the game's systems); between ticks, as the world holds them.
     *
     * @param world The world.
     * @param desc The ray and the filter.
     * @returns The hit, or null when nothing is hit; refused for a zero direction.
     */
    function raycast(world: World, desc?: RaycastDesc): Result<RaycastHit | null>;
    /**
     * The first collider a shape meets when swept along a direction; the same view of the colliders as raycast.
     *
     * @param world The world.
     * @param desc The shape, its sweep and the filter.
     * @returns The hit, or null; refused for a zero direction or an unknown mesh.
     */
    function shapeCast(world: World, desc?: ShapeCastDesc): Result<ShapeCastHit | null>;
    /**
     * Every collider overlapping a shape, sorted by entity id; the same view of the colliders as raycast.
     *
     * @param world The world.
     * @param desc The shape, its placement and the filter.
     */
    function overlap(world: World, desc?: OverlapDesc): Result<readonly Entity[]>;
    /**
     * Push a dynamic body with a force (N, world axes) over the next physics step, at a world point or its centre of mass: call it every tick for a steady force (thrust, wind, a car's downforce or an anti-roll assist). Applied by the og.physics step after the systems, in the order given, then gone; a hit stop drops it. Kinematic and static bodies ignore it. Pushes not yet applied are world state (in snapshots and the hash).
     *
     * @param world The world.
     * @param entity The body's entity (an og.RigidBody).
     * @param force The force in newtons (2D uses x and y).
     * @param point Where it acts, in world space; omitted is the centre of mass (no spin).
     * @returns Refused for an entity without an og.RigidBody, or a force or point that is not finite.
     */
    function applyForce(world: World, entity: Entity, force: Vector3, point?: Vector3): Result<void>;
    /**
     * Give a dynamic body an impulse (N·s, world axes) at a world point or its centre of mass: a sudden change of momentum (a kick, a hit, a jump: its velocity changes by impulse / mass), applied by the next physics step (after a hit stop ends). As applyForce otherwise.
     *
     * @param world The world.
     * @param entity The body's entity (an og.RigidBody).
     * @param impulse The impulse in newton-seconds (2D uses x and y).
     * @param point Where it acts, in world space; omitted is the centre of mass.
     * @returns Refused for an entity without an og.RigidBody, or an impulse or point that is not finite.
     */
    function applyImpulse(world: World, entity: Entity, impulse: Vector3, point?: Vector3): Result<void>;
    /**
     * Turn a dynamic body with a torque (N·m about each world axis; 2D uses z) over the next physics step, as applyForce: call it every tick for a steady twist.
     *
     * @param world The world.
     * @param entity The body's entity (an og.RigidBody).
     * @param torque The torque in newton-metres.
     * @returns Refused for an entity without an og.RigidBody, or a torque that is not finite.
     */
    function applyTorque(world: World, entity: Entity, torque: Vector3): Result<void>;
    /**
     * Give a dynamic body an angular impulse (N·m·s about each world axis; 2D uses z): its spin changes at once, as applyImpulse.
     *
     * @param world The world.
     * @param entity The body's entity (an og.RigidBody).
     * @param torque The angular impulse in newton-metre-seconds.
     * @returns Refused for an entity without an og.RigidBody, or a value that is not finite.
     */
    function applyTorqueImpulse(world: World, entity: Entity, torque: Vector3): Result<void>;
    /**
     * Register (or replace) the geometry a convex or triangle-mesh Collider names by its mesh handle. Loaded glTF geometries are registered under their own handles already; meshes are assets, not world state: world.snapshot carries a mesh, convex-hull or height-field collider by its handle and scale, not its triangles, and world.restore rebuilds the collider from the geometry registered then (so load or register it before restoring; a save naming one that is not is refused).
     *
     * @param world The world.
     * @param mesh The handle colliders name in Collider.mesh.
     * @param desc Positions and triangles.
     * @returns Refused for an empty, non-finite or out-of-range mesh.
     */
    function setMesh(world: World, mesh: Geometry, desc?: MeshDesc): Result<void>;
    /**
     * An og.Rope's points after the last step, from its entity to its end (empty before its first step). The engine draws visible ropes itself; this is for game logic and custom drawing.
     *
     * @param world The world.
     * @param entity The rope's entity.
     * @returns Refused for an entity without an og.Rope.
     */
    function ropePoints(world: World, entity: Entity): Result<readonly Vector3[]>;
    /**
     * An og.Cloth's points after the last step, row by row (`columns` per row, rows downwards from the top edge).
     *
     * @param world The world.
     * @param entity The cloth's entity.
     * @returns Refused for an entity without an og.Cloth.
     */
    function clothPoints(world: World, entity: Entity): Result<readonly Vector3[]>;
    /**
     * An og.Fluid2D's particle positions after the last step, in world space.
     *
     * @param world The world.
     * @param entity The fluid's entity.
     * @returns Refused for an entity without an og.Fluid2D.
     */
    function fluidParticles(world: World, entity: Entity): Result<readonly Vector2[]>;
    /**
     * An og.Fluid3D's particle positions after the last step, in world space (emitted particles last, in the order they appeared).
     *
     * @param world The world.
     * @param entity The fluid's entity.
     * @returns Refused for an entity without an og.Fluid3D.
     */
    function fluid3dParticles(world: World, entity: Entity): Result<readonly Vector3[]>;
    /**
     * The surface height of an og.WaterSurface (world y) or an og.WaterLine (world y at `x`; `z` ignored) at a point, as the last step left it: for boats that bob and spray that starts on the surface.
     *
     * @param world The world.
     * @param entity The water's entity.
     * @param x World x, in metres.
     * @param z World z, in metres (3D).
     * @returns Refused for an entity without water; outside the water, its rest level.
     */
    function waterHeight(world: World, entity: Entity, x: number, z: number): Result<number>;
    /**
     * An og.Vehicle's wheels after the last step (front left, front right, rear left, rear right), for drawing wheel models.
     *
     * @param world The world.
     * @param entity The vehicle's entity.
     * @returns Refused for an entity without a vehicle.
     */
    function vehicleWheels(world: World, entity: Entity): Result<readonly WheelPose[]>;
    /**
     * A ragdoll rig from a loaded skeleton (og.load.animation's `skeleton`): a bone per joint of its first skin, parents first, each from its joint's rest pose to its first child joint, every bone naming its joint and the rig naming the skeleton. Pass it to og.physics.setRagdollRig, edited or not.
     *
     * @param world The world.
     * @param skeleton The skeleton.
     * @returns Refused for a skeleton that is not loaded or has no skin.
     */
    function ragdollRigFromSkeleton(world: World, skeleton: SkeletonAsset): Result<RagdollRigDesc>;
    /**
     * Give an entity with an og.Ragdoll its bones (and, from og.physics.ragdollRigFromSkeleton, the skeleton they stand for: its og.Model is then drawn blended from the animation towards the bones by og.Ragdoll.blend). While weight is 0 the bones follow the rig's rest pose at the entity. Rigs are assets, not world state: kept by the engine and given again to a reloaded world.
     *
     * @param world The world.
     * @param entity The ragdoll's entity.
     * @param desc The bones, parents first.
     * @returns Refused for an empty rig or a parent that does not come before its child.
     */
    function setRagdollRig(world: World, entity: Entity, desc?: RagdollRigDesc): Result<void>;
    /**
     * An og.Ragdoll's bones after the last step, in rig order (empty before its first step).
     *
     * @param world The world.
     * @param entity The ragdoll's entity.
     * @returns Refused for an entity without a ragdoll.
     */
    function ragdollPose(world: World, entity: Entity): Result<readonly BonePose[]>;
    /**
     * What the last physics step cost: each phase's time and whether it ran on the lanes pool. All zero before the first step.
     *
     * @param world The world.
     * @returns Refused for an engine without physics.
     */
    function stats(world: World): Result<PhysicsStats>;
  }
  /**
   * Distance and height fog over the 3D scene and world sprites: distance fog thickens exponentially by density from start (or linearly from start to end when end > start), height fog is densest below height and thins by falloff per metre above it; looking into the sun it turns towards sunColor. The overlay is never fogged.
   */
  const Fog: ResourceType<{ readonly enabled: FieldSpec<"bool">; readonly color: FieldSpec<"Color">; readonly density: FieldSpec<"f32">; readonly start: FieldSpec<"f32">; readonly end: FieldSpec<"f32">; readonly heightDensity: FieldSpec<"f32">; readonly height: FieldSpec<"f32">; readonly falloff: FieldSpec<"f32">; readonly sunColor: FieldSpec<"Color">; readonly sunPower: FieldSpec<"f32">; readonly maxOpacity: FieldSpec<"f32">; readonly sky: FieldSpec<"f32">; readonly volumetric: FieldSpec<"bool">; readonly scattering: FieldSpec<"f32">; readonly anisotropy: FieldSpec<"f32">; readonly volumetricDistance: FieldSpec<"f32"> }, Fog>;
  /**
   * A procedural sky of chosen colours, drawn behind the scene per pixel and lighting it (the image-based light follows it). Without it, RenderSettings.sky draws the built-in sky. A DayNight that is enabled sets the sky itself.
   */
  const Sky: ResourceType<{ readonly enabled: FieldSpec<"bool">; readonly zenith: FieldSpec<"Color">; readonly horizon: FieldSpec<"Color">; readonly ground: FieldSpec<"Color">; readonly sunDirection: FieldSpec<"Vector3">; readonly sunColor: FieldSpec<"Color">; readonly sunSize: FieldSpec<"f32">; readonly sunIntensity: FieldSpec<"f32">; readonly glow: FieldSpec<"f32">; readonly glowPower: FieldSpec<"f32">; readonly stars: FieldSpec<"f32">; readonly brightness: FieldSpec<"f32">; readonly moonSize: FieldSpec<"f32">; readonly moonPhase: FieldSpec<"f32">; readonly moonIntensity: FieldSpec<"f32">; readonly moonColor: FieldSpec<"Color">; readonly moonDirection: FieldSpec<"Vector3"> }, Sky>;
  /**
   * A day-night cycle: the time of day drives the sun's direction, the sky's colours, stars and the moon disc, the first DirectionalLight (the sun by day, a dim moon opposite it by night), the ambient light and the fog's colour. The clock is start plus the simulated seconds times speed, wrapped to 24 hours; the image-based light is regenerated every `step` minutes of game time while the sky drawn per pixel moves smoothly. The drive happens as the frame is drawn and never writes the world: the DirectionalLight keeps the colour, intensity and transform it was given. The probe's `get` answers what is drawn, a pure function of the tick: `get:og.DayNight` adds `hours`, `toSun`, `toLight` (towards the sun by day, the moon by night), `lightColor` and `lightIntensity`, and `get:<sun>.og.DirectionalLight` on the driven light answers the drawn `color` and `intensity` with `toLight` and `drivenBy`.
   */
  const DayNight: ResourceType<{ readonly enabled: FieldSpec<"bool">; readonly start: FieldSpec<"f32">; readonly speed: FieldSpec<"f32">; readonly tilt: FieldSpec<"f32">; readonly azimuth: FieldSpec<"f32">; readonly lightIntensity: FieldSpec<"f32">; readonly tintFog: FieldSpec<"bool">; readonly step: FieldSpec<"f32">; readonly moonSize: FieldSpec<"f32">; readonly moonPhase: FieldSpec<"f32"> }, DayNight>;
  /**
   * A weather preset around the camera: particles (rain, snow, dust), its fog (unless og.Fog is enabled, which takes precedence) and dimmed sunlight. Particles are simulated by the renderer from the tick, so they look the same on every run; they never touch the simulation.
   */
  const Weather: ResourceType<{ readonly kind: FieldSpec<"u32">; readonly intensity: FieldSpec<"f32">; readonly wind: FieldSpec<"Vector3">; readonly radius: FieldSpec<"f32">; readonly tint: FieldSpec<"Color">; readonly dim: FieldSpec<"f32">; readonly layer: FieldSpec<"i32"> }, Weather>;
  /**
   * A planar water surface on the entity's xz plane (centred on it, turned with it): colour by the water's depth over what is below, foam where it meets geometry, animated waves, refraction and reflections (screen-space where the tier allows, the sky's cubemap on WebGL2, or a planar reflection), the sun's shadows, caustics below it and an underwater view. With an og.WaterSurface on the same entity its wave heights displace the surface (centred on the entity, unrotated).
   */
  const Water: ComponentType<{ readonly size: FieldSpec<"Vector2">; readonly segments: FieldSpec<"u32">; readonly shallow: FieldSpec<"Color">; readonly deep: FieldSpec<"Color">; readonly depthFade: FieldSpec<"f32">; readonly opacity: FieldSpec<"f32">; readonly foam: FieldSpec<"Color">; readonly foamWidth: FieldSpec<"f32">; readonly waveScale: FieldSpec<"f32">; readonly waveSpeed: FieldSpec<"f32">; readonly waveStrength: FieldSpec<"f32">; readonly refraction: FieldSpec<"f32">; readonly reflectivity: FieldSpec<"f32">; readonly planarReflection: FieldSpec<"f32">; readonly shadows: FieldSpec<"f32">; readonly underwaterColor: FieldSpec<"Color">; readonly underwaterDensity: FieldSpec<"f32">; readonly caustics: FieldSpec<"f32">; readonly causticsScale: FieldSpec<"f32"> }, Water>;
  /**
   * Draws the og.WaterLine on the same entity in the sprite order: the body from its surface down, a gradient from the top colour to the bottom one, and a bright lip along the surface.
   */
  const Water2D: ComponentType<{ readonly top: FieldSpec<"Color">; readonly topOpacity: FieldSpec<"f32">; readonly bottom: FieldSpec<"Color">; readonly bottomOpacity: FieldSpec<"f32">; readonly lip: FieldSpec<"Color">; readonly lipWidth: FieldSpec<"f32">; readonly layer: FieldSpec<"i32"> }, Water2D>;
  /**
   * Fog, a procedural sky, the day-night cycle, weather and water, declared as resources and components (the E5 round). Everything is off by default: a world that sets none of them draws exactly as before.
   * The day-night clock is a function of the tick (DayNight.start plus the simulated time times speed), so it is the same on every run and after a restore; rendering reads the world and never writes it.
   */
  namespace atmosphere {
    /**
     * The DayNight clock at the current tick, in hours from 0 to 24 (its start when the cycle is off): for lamps that light at dusk and shops that close at night. Inside a system, declare og.DayNight among its reads.
     *
     * @param world The world.
     */
    function timeOfDay(world: World): number;
  }
  /**
   * A posed skeleton: the entity is drawn skinned by the joint palette og.anim writes each tick. The pose itself (every node's local transform) is world state kept by the `og.anim` state block.
   */
  const Skeleton: ComponentType<{ readonly asset: FieldSpec<"SkeletonAsset">; readonly skin: FieldSpec<"u32">; readonly rootMotion: FieldSpec<"RootMotion">; readonly rootJoint: FieldSpec<"u32">; readonly jointCount: FieldSpec<"u32"> }, Skeleton>;
  /**
   * Plays one clip on a skeleton. Alone it poses the skeleton from its rest pose; with an og.AnimationGraph it plays over the graph's result with `weight`. Changing `clip` cross-fades from the previous one over `fade`; a clip change starts the new clip at 0 (or at `time` if the game wrote it in the same tick).
   */
  const AnimationPlayer: ComponentType<{ readonly clip: FieldSpec<"ClipAsset">; readonly time: FieldSpec<"f32">; readonly speed: FieldSpec<"f32">; readonly loop: FieldSpec<"bool">; readonly weight: FieldSpec<"f32">; readonly fade: FieldSpec<"f32">; readonly stepFps: FieldSpec<"f32">; readonly finished: FieldSpec<"bool"> }, AnimationPlayer>;
  /**
   * Runs a blend-tree graph (og.anim.graph) on a skeleton: its layers' state machines pick motions by the parameters p0...p7, which the graph names in `param` order (`param speed` is p0, the next `param` is p1, ...).
   */
  const AnimationGraph: ComponentType<{ readonly asset: FieldSpec<"GraphAsset">; readonly weight: FieldSpec<"f32">; readonly speed: FieldSpec<"f32">; readonly p0: FieldSpec<"f32">; readonly p1: FieldSpec<"f32">; readonly p2: FieldSpec<"f32">; readonly p3: FieldSpec<"f32">; readonly p4: FieldSpec<"f32">; readonly p5: FieldSpec<"f32">; readonly p6: FieldSpec<"f32">; readonly p7: FieldSpec<"f32">; readonly state: FieldSpec<"u32">; readonly stateTime: FieldSpec<"f32"> }, AnimationGraph>;
  /**
   * Morph-target weights of the skeleton entity's mesh, in the order of the glTF mesh's targets. With `source` clips the playing clips' weight channels write them; with `component` game code does.
   */
  const Blendshapes: ComponentType<{ readonly source: FieldSpec<"BlendshapeSource">; readonly mesh: FieldSpec<"u32">; readonly w0: FieldSpec<"f32">; readonly w1: FieldSpec<"f32">; readonly w2: FieldSpec<"f32">; readonly w3: FieldSpec<"f32">; readonly w4: FieldSpec<"f32">; readonly w5: FieldSpec<"f32">; readonly w6: FieldSpec<"f32">; readonly w7: FieldSpec<"f32"> }, Blendshapes>;
  /**
   * An inverse-kinematics constraint on a skeleton, applied after its clips and graph: the chain of `bones` bones ending at joint `tip` turns so the tip reaches `target`. Put it on its own entity (a skeleton can have many chains); chains apply in query order.
   */
  const IkChain: ComponentType<{ readonly skeleton: FieldSpec<"Entity">; readonly tip: FieldSpec<"u32">; readonly bones: FieldSpec<"u32">; readonly solver: FieldSpec<"IkSolver">; readonly target: FieldSpec<"Vector3">; readonly pole: FieldSpec<"Vector3">; readonly space: FieldSpec<"IkSpace">; readonly weight: FieldSpec<"f32">; readonly iterations: FieldSpec<"u32">; readonly tolerance: FieldSpec<"f32">; readonly reached: FieldSpec<"bool">; readonly error: FieldSpec<"f32"> }, IkChain>;
  /**
   * A named marker of a clip that playback passed in the last tick (read in the tick after it fired, like every event). Cross-fades and blends fire every contributing clip's markers.
   */
  const AnimationEvent: EventType<AnimationEvent>;
  /**
   * Plays a clip of a flipbook sheet on the entity's og.Sprite: extraction draws the sprite with the sheet's texture and the current frame's region and pivot, or, for an `array true` sheet, the frame's layer of its texture array whole (the sprite keeps its size, colour, flips and layer). Played by the engine's og.flipbook system by scaled time (Time.scaledDt: slow motion, hit stop), after the game's systems.
   * Changing clip restarts at the clip's first frame (or at `time`, written in the same tick); writing a different time is a seek. Frame events are og.FlipbookEvent. The playback position is world state (the og.flipbook state block).
   */
  const Flipbook: ComponentType<{ readonly sheet: FieldSpec<"FlipbookSheet">; readonly clip: FieldSpec<"u32">; readonly speed: FieldSpec<"f32">; readonly playing: FieldSpec<"bool">; readonly frame: FieldSpec<"u32">; readonly time: FieldSpec<"f32">; readonly finished: FieldSpec<"bool">; readonly loops: FieldSpec<"u32"> }, Flipbook>;
  /**
   * A flipbook entered a frame its clip names in an `event` line (`event 2 "step"`): footsteps, the frame a hit lands, a pickup's sparkle. Read in the tick after, like every event; a clip started at 0 fires its first frame's events.
   */
  const FlipbookEvent: EventType<FlipbookEvent>;
  /**
   * Plays a clip of a 2D rig at the entity: the og.skeleton2d system advances it by scaled time (Time.scaledDt) after the game's systems and fires its events as og.Skeleton2DEvent; extraction draws its slots, in their order, as textured triangles of the sprite batch at `layer` (region attachments as quads, mesh attachments weighted to bones and deformed by the clip), lit by og.Lighting2D when the layer is in its range, as a sprite there is. The rig's pixels are `unit` to a world unit and +y is up; the entity's og.Transform places, turns (about z) and scales it.
   * Changing `clip` restarts at 0 (or at `time`, written in the same tick); writing a different time is a seek and fires no events. The pose is a function of these fields, the Transform and the og.IkTarget2D on it; the playback position is world state (the og.skeleton2d state block).
   */
  const Skeleton2D: ComponentType<{ readonly rig: FieldSpec<"Rig2D">; readonly clip: FieldSpec<"u32">; readonly time: FieldSpec<"f32">; readonly speed: FieldSpec<"f32">; readonly loop: FieldSpec<"bool">; readonly flipX: FieldSpec<"bool">; readonly layer: FieldSpec<"i32">; readonly finished: FieldSpec<"bool"> }, Skeleton2D>;
  /**
   * A 2D skeleton's clip passed one of its events (`event 0.2 "step"` in the rig) in the last tick: footsteps, the frame a swing lands. Read in the tick after, like every event; a clip started at 0 fires its events at 0, a looping clip fires each once per pass.
   */
  const Skeleton2DEvent: EventType<Skeleton2DEvent>;
  /**
   * Moves one IK constraint of a 2D skeleton (Rig2DInfo.constraints) to a point: a foot planted on the ground under it, a hand on a ledge. Put it on its own entity, one per constraint; og.skeleton2d writes `reached` and `error`, and the drawn pose solves the same way.
   */
  const IkTarget2D: ComponentType<{ readonly skeleton: FieldSpec<"Entity">; readonly constraint: FieldSpec<"u32">; readonly target: FieldSpec<"Vector2">; readonly weight: FieldSpec<"f32">; readonly tolerance: FieldSpec<"f32">; readonly reached: FieldSpec<"bool">; readonly error: FieldSpec<"f32"> }, IkTarget2D>;
  /**
   * Skeletal animation over og-anim: glTF skins and clips (cooked into the `<model>#anim` blob, loaded with og.load.animation), blend-tree graphs with layered state machines, root motion, two-bone and FABRIK IK and blendshapes, evaluated by the engine's og.anim system after the game's systems and before physics.
   * Game code owns the components and sets players, graph parameters and IK targets; og.anim writes the pose (kept as world state in the `og.anim` state block, so snapshots, restore and the world hash cover it), the joint palette, root motion into og.Transform and the fields marked written. Asset handles are what og.load.animation and og.anim.graph answer; 0 is none.
   * Clips bind to a skeleton's nodes by name, so a clip of one rig plays on another with the same node names; a rig with other names or proportions plays it retargeted (og.anim.retarget at run time, or a model's `animation { library }` import setting at cook time). Assets are not world state: a hot reload registers them again.
   */
  namespace anim {
    /**
     * Register a blend-tree graph for AnimationGraph.asset. Its `clip` lines are resolved by name against `clips`.
     *
     * @param world The world whose animation owns the graph.
     * @param desc The text and the clips it may name.
     * @returns Refused for a parse error (the reason names the line), a clip name `clips` lacks, or an invalid graph.
     */
    function graph(world: World, desc?: GraphDesc): Result<GraphAsset>;
    /**
     * The standard humanoid bones of a skeleton, as retargeting sees them: to check a rig before retargeting onto it.
     *
     * @param world The world whose animation owns the skeleton.
     * @param desc The skeleton and its rig.
     * @returns Refused for an unregistered skeleton or rig, or a rig naming a node the skeleton lacks.
     */
    function humanoid(world: World, desc?: HumanoidDesc): Result<Humanoid>;
    /**
     * A new clip for `target` from a clip of `source`: each animated humanoid bone turns as the source's does from its rest pose (after a rest correction, so a T-pose drives an A-pose and other bone frames agree), the target keeps its bone lengths, the hips travel the source's distance scaled by the leg ratio (root motion kept, hip height scaled), and with footIk the feet land where the source's do, scaled. Same name, duration and markers. The cook does the same for a model's `animation { library }` import setting; this is for clips chosen at run time.
     *
     * @param world The world whose animation owns the skeletons and the clip.
     * @param desc The clip, the two skeletons and their rigs.
     * @returns The new clip (what AnimationPlayer.clip takes). Refused for an unregistered clip, skeleton or rig, a skeleton without recognisable hips (name them in a `type rig` file), or a rig naming a missing node.
     */
    function retarget(world: World, desc?: RetargetDesc): Result<ClipInfo>;
    /**
     * The index of a flipbook clip by name, for Flipbook.clip: `og.anim.clip(hero, "walk_down")`. -1 when the sheet has no such clip.
     *
     * @param info What og.load.flipbook answered.
     * @param name The clip's name.
     */
    function clip(info: FlipbookInfo, name: string): number;
    /**
     * A loaded model's clip by name, for AnimationPlayer.clip: `og.anim.clip(og.load.animation(world, "models/slime.glb").value, "chase")`, the same lookup as a flipbook's (Animation.clips by ClipInfo.name). A skinned glTF's clips are its animations' names; an animated `.vox`'s are `frames` or its `voxClip` names. 0 (plays nothing) when the model has no such clip.
     *
     * @param info What og.load.animation answered.
     * @param name The clip's name.
     */
    function clip(info: Animation, name: string): ClipAsset;
    /**
     * The index of a 2D rig's clip by name, for Skeleton2D.clip: `og.anim.clip(og.load.rig2d(world, "rigs/hero.ogt").value, "walk")`. -1 when the rig has no such clip.
     *
     * @param info What og.load.rig2d or og.anim.rig2dInfo answered.
     * @param name The clip's name.
     */
    function clip(info: Rig2DInfo, name: string): number;
    /**
     * What a loaded 2D rig holds: its bones, slots, IK constraints and clips, by the handle Skeleton2D.rig keeps (og.load.rig2d answers the same on load).
     *
     * @param world The world whose animation owns the rig.
     * @param rig The rig.
     * @returns Refused for a handle og.load.rig2d did not answer.
     */
    function rig2dInfo(world: World, rig: Rig2D): Result<Rig2DInfo>;
    /**
     * The bounding boxes an og.Skeleton2D's pose showed when og.skeleton2d last ran (after the game's systems, so a system reads the previous tick's): hurt and hit boxes a clip switches on and off with its slots' attachments, to test against points or other shapes. Empty for a rig without boxes or when no slot shows one. They are part of the og.skeleton2d state block, so a restored snapshot answers what it saved.
     *
     * @param world The world whose animation poses the skeleton.
     * @param entity The entity with the og.Skeleton2D.
     * @returns Refused for an entity og.skeleton2d has not posed: no og.Skeleton2D with a loaded rig, or before its first tick.
     */
    function skeleton2dBoxes(world: World, entity: Entity): Result<readonly Skeleton2DBox[]>;
  }
  /** A tween or group ended on its own (read in the tick after). */
  const TweenCompleted: EventType<TweenCompleted>;
  /**
   * Tweens: animate any numeric field of a component (og.Transform.position, og.Sprite.color, og.Sprite.opacity, a game's own f32) from its value to a target over time with an easing curve, a delay, loops and yoyo; chain them in sequences or run them in parallel. Advanced by the engine's og.tween system after the game's systems and before og.anim and physics, by scaled time (slow motion, hit stop) unless unscaled.
   * Running tweens are world state (the og.tween state block): snapshots, restore and the world hash cover them. og.TweenCompleted reports a tween or group that ends on its own; a cancelled one, or one whose entity or component went away, ends silently.
   * Feel presets are tweens too: og.tween.squash (squash and stretch that recovers) and og.tween.flash (a hit flash that fades).
   */
  namespace tween {
    /**
     * Start a tween. Systems may call it; it first moves in this tick's og.tween run.
     *
     * @param world The world.
     * @param desc The field, the values and the timing.
     * @returns Refused for a dead entity, a component it lacks, a field that is not numeric, or values of the wrong length.
     */
    function to(world: World, desc?: TweenDesc): Result<Tween>;
    /**
     * Start tweens one after another (each with its own delay, duration and loops); the time one leaves over goes to the next.
     *
     * @param world The world.
     * @param steps The tweens, in order.
     * @param desc Loops, scaling and the tag.
     */
    function sequence(world: World, steps: readonly TweenDesc[], desc?: TweenGroupDesc): Result<Tween>;
    /**
     * Start tweens together; the group ends with the last one.
     *
     * @param world The world.
     * @param tweens The tweens.
     * @param desc Loops, scaling and the tag.
     */
    function parallel(world: World, tweens: readonly TweenDesc[], desc?: TweenGroupDesc): Result<Tween>;
    /**
     * Stop a tween where it is (no event).
     *
     * @param world The world.
     * @param tween The handle.
     * @returns False when it was not running.
     */
    function cancel(world: World, tween: Tween): boolean;
    /**
     * Stop every tween of an entity (no events), say before starting a new reaction.
     *
     * @param world The world.
     * @param entity The entity.
     * @returns How many stopped.
     */
    function cancelEntity(world: World, entity: Entity): number;
    /**
     * Whether a tween is still running.
     *
     * @param world The world.
     * @param tween The handle.
     */
    function active(world: World, tween: Tween): boolean;
    /**
     * Squash and stretch the entity's og.Transform.scale (cancels its earlier squash).
     *
     * @param world The world.
     * @param desc The entity and the impact.
     * @returns Refused for an entity without og.Transform.
     */
    function squash(world: World, desc?: SquashDesc): Result<Tween>;
    /**
     * Flash the entity's og.Sprite, its og.Mesh, or its og.Model (a glTF instance: this instance only, the others sharing its materials stay as they are). Cancels its earlier flash.
     *
     * @param world The world.
     * @param desc The entity and the flash.
     * @returns Refused for an entity with none of og.Sprite, og.Mesh and og.Model.
     */
    function flash(world: World, desc?: FlashDesc): Result<Tween>;
  }
  /**
   * Draws a loaded map with its top-left corner at the entity's position (z included) and gives its collision layers static colliders. The og.tilemap system adopts it in the first tick it exists: from then on og.tilemap.setTile and getTile reach it.
   */
  const Tilemap: ComponentType<{ readonly map: FieldSpec<"TilemapAsset">; readonly layer: FieldSpec<"i32">; readonly color: FieldSpec<"Color">; readonly opacity: FieldSpec<"f32">; readonly visible: FieldSpec<"bool">; readonly collision: FieldSpec<"bool">; readonly colliderLayers: FieldSpec<"u32">; readonly friction: FieldSpec<"f32"> }, Tilemap>;
  /**
   * Tilemaps: layers of tiles from a tileset atlas (an `.ogt` of `type tileset`: grid, solid tiles and collision rects, animated tiles, 47-blob and 16-bitmask auto-tiling terrains), loaded from a cooked map (a hand-written `.ogt` of `type tilemap`, a Tiled `.tmj` or an LDtk `.ldtk`) with og.load.tilemap and placed by an og.Tilemap component.
   * Layers draw through the sprite batch in 16x16-tile chunks culled to the view, each at the map's sprite layer plus its z, scrolled by its parallax; solid tiles become static og-physics colliders merged into rectangles. Runtime edits (og.tilemap.setTile, re-auto-tiling the neighbourhood) are world state (the og.tilemap state block), and the colliders follow them.
   * With og.Lighting2D enabled, a layer whose sprite layer (the map's layer plus its z) lies in minLayer to maxLayer is lit as the sprites there are: multiplied by the ambient plus every Light2D reaching it, with the shadows of Occluder2Ds and castShadow sprites, so a dark cave lit by torches is the map, its lights and its occluders. A layer that should stay as authored in the lit range (a sky, a far parallax backdrop) opts out: a bool custom property `lit` set to false on the Tiled layer, or a `lit false` line in a tilemap `.ogt`'s layer block (TileLayerInfo.lit reports it).
   * Cells count from the top-left corner, x right and y down (Tiled's); the map's top-left corner is at the entity's position.
   */
  namespace tilemap {
    /**
     * Change a cell. It applies at once to getTile, and to drawing and colliders at the og.tilemap system (this tick when called before it). Deterministic: equal edits in equal order give equal maps.
     *
     * @param world The world.
     * @param desc The map, the layer, the cell and the tile.
     * @returns Refused for an entity whose map is not adopted yet, an unknown layer, a cell outside the map or a tile the tileset lacks.
     */
    function setTile(world: World, desc?: SetTileDesc): Result<void>;
    /**
     * A cell's tile id, -1 when empty: on a terrain layer the tile auto-tiling resolved from the neighbours (setTile writes 1 or 0 there; the id changes with them). -1 outside the map or before the map is adopted.
     *
     * @param world The world.
     * @param entity The entity with the og.Tilemap.
     * @param layer The layer's name.
     * @param x The column.
     * @param y The row.
     */
    function getTile(world: World, entity: Entity, layer: string, x: number, y: number): number;
    /**
     * The cell under a world point (its xy), for a map at its entity's position.
     *
     * @param world The world.
     * @param entity The entity with the og.Tilemap.
     * @param point A world point.
     */
    function cellAt(world: World, entity: Entity, point: Vector3): TileCell;
    /**
     * A cell's centre in world coordinates (z of the map).
     *
     * @param world The world.
     * @param entity The entity with the og.Tilemap.
     * @param x The column.
     * @param y The row.
     */
    function cellCenter(world: World, entity: Entity, x: number, y: number): Vector3;
  }
  /**
   * A sound an entity plays. Level-triggered: while `playing` is true it plays (inserting a source plays it), setting it false stops it, and a one-shot that reaches its end sets it back to false and sends og.SoundEnded.
   * A one-shot of F frames at rate R and pitch p started in tick S ends in tick S + ceil(F * tickRate / (R * p)) - 1. A despawned entity's voice is silenced without an event.
   */
  const AudioSource: ComponentType<{ readonly sound: FieldSpec<"Sound">; readonly bus: FieldSpec<"AudioBus">; readonly volume: FieldSpec<"f32">; readonly pitch: FieldSpec<"f32">; readonly pan: FieldSpec<"f32">; readonly loop: FieldSpec<"bool">; readonly loopStart: FieldSpec<"f32">; readonly loopEnd: FieldSpec<"f32">; readonly spatial: FieldSpec<"bool">; readonly refDistance: FieldSpec<"f32">; readonly maxDistance: FieldSpec<"f32">; readonly rolloffFactor: FieldSpec<"f32">; readonly playing: FieldSpec<"bool">; readonly priority: FieldSpec<"u32">; readonly lowpass: FieldSpec<"f32">; readonly occlude: FieldSpec<"bool">; readonly distanceCutoff: FieldSpec<"f32">; readonly startAt: FieldSpec<"f64">; readonly startOffset: FieldSpec<"f64">; readonly fade: FieldSpec<"f32"> }, AudioSource>;
  /**
   * Where spatial sources are heard from: the entity's world transform (its og.Transform down og.Parent, so a listener under a camera rig hears from the rig), looking down its -Z with +X to the right. The first listener in query order is used.
   */
  const AudioListener: ComponentType<{ readonly volume: FieldSpec<"f32"> }, AudioListener>;
  /** Bus volumes; master scales every bus. A change ramps over 256 frames in the mixer. */
  const Audio: ResourceType<{ readonly master: FieldSpec<"f32">; readonly music: FieldSpec<"f32">; readonly sfx: FieldSpec<"f32">; readonly ui: FieldSpec<"f32">; readonly maxVoices: FieldSpec<"u32">; readonly occlusionGain: FieldSpec<"f32">; readonly occlusionCutoff: FieldSpec<"f32">; readonly reverbRoom: FieldSpec<"f32">; readonly reverbDamping: FieldSpec<"f32">; readonly outputLatency: FieldSpec<"f64"> }, Audio>;
  /**
   * A bus's insert effects: a high- and low-pass (12 dB/octave), a compressor, side-chain ducking by another bus, and a send to the shared reverb, whose return joins master. One entity per bus (the first in query order counts); without one a bus passes straight through.
   * Changes reach the mixer through the command stream, so they are deterministic and in the world hash; the PCM is the mixer's.
   */
  const AudioBusFx: ComponentType<{ readonly bus: FieldSpec<"AudioBus">; readonly lowpass: FieldSpec<"f32">; readonly highpass: FieldSpec<"f32">; readonly reverb: FieldSpec<"f32">; readonly threshold: FieldSpec<"f32">; readonly ratio: FieldSpec<"f32">; readonly attack: FieldSpec<"f32">; readonly release: FieldSpec<"f32">; readonly makeup: FieldSpec<"f32">; readonly duck: FieldSpec<"bool">; readonly duckBus: FieldSpec<"AudioBus">; readonly duckDepth: FieldSpec<"f32">; readonly duckThreshold: FieldSpec<"f32">; readonly duckAttack: FieldSpec<"f32">; readonly duckRelease: FieldSpec<"f32"> }, AudioBusFx>;
  /**
   * A reverb zone: a box (or a sphere) where the shared reverb sounds like this room. While the og.AudioListener is inside, og.audio eases the reverb's room size and damping, and one bus's send to it, towards the zone's; past the volume's surface it fades out linearly over `fade` metres, so walking in through a door the room grows around the listener.
   * Deterministic: each tick the listener's weight in every zone is a function of its position (written to `weight`), the strongest zone wins (ties to the first in query order), and the eased values reach the mixer as fx commands in the stream and the world hash. Audio.reverbRoom, Audio.reverbDamping and the bus's AudioBusFx.reverb stay the open air's: the zone never writes them. The listener and the zone are placed at their world transforms (og.Transform down og.Parent): a zone on a child of a moving room moves with it.
   */
  const ReverbZone: ComponentType<{ readonly size: FieldSpec<"Vector3">; readonly radius: FieldSpec<"f32">; readonly fade: FieldSpec<"f32">; readonly bus: FieldSpec<"AudioBus">; readonly send: FieldSpec<"f32">; readonly room: FieldSpec<"f32">; readonly damping: FieldSpec<"f32">; readonly weight: FieldSpec<"f32"> }, ReverbZone>;
  /**
   * A musical clock: tempo, metre and where bar 1 starts. The og.music system (before og.audio) writes the bar and beat and the times of the next beat and bar line every tick, and sends MusicBeat for every beat; an AudioSource whose startAt is nextBar starts on the bar line to the sample, so stems started together stay in step and a stinger lands on the downbeat.
   */
  const Music: ComponentType<{ readonly bpm: FieldSpec<"f32">; readonly beatsPerBar: FieldSpec<"u32">; readonly start: FieldSpec<"f64">; readonly bar: FieldSpec<"u32">; readonly beat: FieldSpec<"u32">; readonly nextBeat: FieldSpec<"f64">; readonly nextBar: FieldSpec<"f64"> }, Music>;
  /**
   * A beat of a Music clock, sent in the tick whose interval holds it (systems read it the tick after).
   */
  const MusicBeat: EventType<MusicBeat>;
  /** A source started, sent in the tick it happens (systems read it the tick after). */
  const SoundStarted: EventType<SoundStarted>;
  /** A sound ended, sent in the tick it happens (systems read it the tick after). */
  const SoundEnded: EventType<SoundEnded>;
  /**
   * Sound: inside the tick, the engine's og.audio system (after the game's systems and physics) turns AudioSource changes into a deterministic, time-stamped command stream; outside the tick the host's mixer renders it (an AudioWorklet on the web, the output device, a capture or a null sink natively).
   * The stream, not the PCM, is world state: snapshots, restore and the world hash cover it, and nothing a sink does reaches the world. Load sounds with og.load.sound; inserting a source plays it, setting `playing` false stops it, and a one-shot sets `playing` back to false when it ends.
   * Spatial gain and pan are computed in the tick from the source's world transform relative to the first AudioListener's (three.js convention: the listener looks down -Z, +X is right; with none, the origin looking down -Z). A source, listener or zone on a child entity is placed at its og.Transform composed down og.Parent, as og.MatrixWorld is: a sound on a child is heard where its parent carries it, a listener under a camera rig hears from the rig.
   */
  namespace audio {
    /**
     * How long after its world time a sample leaves the speaker, as the host reports it: on the web the AudioContext's baseLatency and outputLatency and the worklet ring's buffer, natively the device's latency (cpal's playback less callback instant) and the mixer's buffer; 0 headless (the null and capture sinks are driven to the tick). Host input, written to og.Audio between ticks, so a replay carries it. A rhythm game hears a note at its time plus this: judge a press against the note's time plus the latency, then a calibrated rest (the display, the hand).
     * Inside a system og.Audio must be among its reads or writes.
     *
     * @param world The world.
     * @returns Seconds; 0 headless.
     */
    function outputLatency(world: World): number;
  }
  /**
   * An entity that moves itself towards `target` while `seeking`, by a path over `nav` (a grid, navmesh or flow field; 0 goes straight). og.nav re-plans when the target moved more than repathDistance, every repathTicks, or when the grid or navmesh changed, at most 32 plans a tick across agents.
   */
  const NavAgent: ComponentType<{ readonly nav: FieldSpec<"Navigation">; readonly target: FieldSpec<"Vector3">; readonly seeking: FieldSpec<"bool">; readonly speed: FieldSpec<"f32">; readonly acceleration: FieldSpec<"f32">; readonly radius: FieldSpec<"f32">; readonly stopDistance: FieldSpec<"f32">; readonly slowRadius: FieldSpec<"f32">; readonly repathTicks: FieldSpec<"u32">; readonly repathDistance: FieldSpec<"f32">; readonly smooth: FieldSpec<"bool">; readonly faceMovement: FieldSpec<"bool">; readonly turnSpeed: FieldSpec<"f32">; readonly drive: FieldSpec<"bool">; readonly status: FieldSpec<"u32">; readonly velocity: FieldSpec<"Vector3">; readonly waypoint: FieldSpec<"Vector3">; readonly remaining: FieldSpec<"f32">; readonly plans: FieldSpec<"u32"> }, NavAgent>;
  /**
   * Steering behaviours an og.NavAgent adds to its path following, each by weight (0 is off): flee, wander, pursue, evade, separation from other agents and avoidance of colliders ahead (three physics rays).
   */
  const Steering: ComponentType<{ readonly seek: FieldSpec<"f32">; readonly flee: FieldSpec<"f32">; readonly fleeFrom: FieldSpec<"Vector3">; readonly fleeRadius: FieldSpec<"f32">; readonly wander: FieldSpec<"f32">; readonly wanderRadius: FieldSpec<"f32">; readonly wanderDistance: FieldSpec<"f32">; readonly wanderJitter: FieldSpec<"f32">; readonly pursue: FieldSpec<"Entity">; readonly pursueWeight: FieldSpec<"f32">; readonly evade: FieldSpec<"Entity">; readonly evadeWeight: FieldSpec<"f32">; readonly separation: FieldSpec<"f32">; readonly separationRadius: FieldSpec<"f32">; readonly avoidance: FieldSpec<"f32">; readonly avoidDistance: FieldSpec<"f32">; readonly avoidMask: FieldSpec<"u32">; readonly wanderAngle: FieldSpec<"f32"> }, Steering>;
  /**
   * Navigation: grids (A* and jump point search with eight-way moves that never cut a blocked corner, line-of-sight smoothing), flow fields for crowds, and navmeshes (baked at cook time from a level's static colliders and models, or built at runtime for a flat procedural level) with funnel paths and box obstacles that rebuild only the tiles they touch.
   * Agents are entities with an og.NavAgent (and an og.Transform): the og.nav system, after the game's systems and before physics, plans their paths, follows them and adds the og.Steering behaviours; an entity with an og.Velocity (a physics body) gets its velocity written, any other its Transform moved. Everything is deterministic and in snapshots: paths, grid edits, obstacles and flow targets.
   * Agents move on the profile's plane: x and y in 2D (a grid's rows run up), x and z in 3D (y kept on grids, from the surface on a navmesh).
   */
  namespace nav {
    /**
     * A grid agents and path queries plan on. Runtime edits (og.nav.setCost) are world state.
     *
     * @param world The world.
     * @param desc Its size, cell size, origin and costs.
     * @returns Refused for costs of the wrong length.
     */
    function grid(world: World, desc?: NavGridDesc): Result<Navigation>;
    /**
     * A grid over a placed map: its cells, its world position (the map's top-left corner at the entity), blocked where the collision layers have solid tiles. Edits to the map later do not follow; build another.
     *
     * @param world The world.
     * @param desc The map's entity and the blocking layers.
     * @returns Refused for an entity without an og.Tilemap or a map not loaded.
     */
    function gridFromTilemap(world: World, desc?: TilemapGridDesc): Result<Navigation>;
    /**
     * Change a grid cell's cost (0 blocks it): agents on the grid re-plan, flow fields over it rebuild.
     *
     * @param world The world.
     * @param grid The grid.
     * @param x The column.
     * @param y The row (0 is the bottom).
     * @param cost 0 blocked, 1 normal, up to 255.
     * @returns Refused for an unknown grid or a cell outside it.
     */
    function setCost(world: World, grid: Navigation, x: number, y: number, cost: number): Result<void>;
    /**
     * The grid cell under a world point.
     *
     * @param world The world.
     * @param grid The grid.
     * @param point A world point.
     */
    function cellAt(world: World, grid: Navigation, point: Vector3): NavCell;
    /**
     * A path between two points on a grid or a navmesh, now: corners, length and whether it reaches the end.
     *
     * @param world The world.
     * @param desc The grid or navmesh and the two points.
     * @returns Refused for an unknown grid or navmesh, or when no path starts (an end off the grid or mesh).
     */
    function findPath(world: World, desc?: PathDesc): Result<NavPath>;
    /**
     * A flow field towards targets over a grid: one Dijkstra pass gives every cell a direction, so any number of agents (NavAgent.nav = the field) steer by a lookup. It rebuilds when the grid changes.
     *
     * @param world The world.
     * @param desc The grid and the targets.
     * @returns Refused for an unknown grid.
     */
    function flowField(world: World, desc?: FlowDesc): Result<Navigation>;
    /**
     * Move a flow field's targets (rebuilds it).
     *
     * @param world The world.
     * @param field The flow field.
     * @param targets The new targets.
     * @returns Refused for an unknown flow field.
     */
    function setFlowTargets(world: World, field: Navigation, targets: readonly Vector3[]): Result<void>;
    /**
     * The direction a flow field gives at a point (unit length, or zero at a target or where nothing is reachable).
     *
     * @param world The world.
     * @param field The flow field.
     * @param point A world point.
     */
    function flowAt(world: World, field: Navigation, point: Vector3): Vector3;
    /**
     * A cooked navmesh: an `.ogt` of `type navmesh` (the agent, the cell sizes, the bounds and the geometry: a scene's static colliders and models, glTF models, terrains, boxes; og-ai §1 has every line) baked by og cook.
     *
     * @param world The world.
     * @param path The navmesh asset, such as `levels/arena.nav.ogt`.
     * @returns Refused for a path that is not a cooked navmesh.
     */
    function load(world: World, path: string): Result<NavMeshInfo>;
    /**
     * A navmesh over a flat floor with boxes carved out, built now: for procedural levels (a level with slopes and stairs is baked at cook time with og.nav.load).
     *
     * @param world The world.
     * @param desc The floor, the boxes and the agent.
     * @returns Refused for an empty floor.
     */
    function build(world: World, desc?: NavMeshDesc): Result<NavMeshInfo>;
    /**
     * Set, move or (with no box) remove an obstacle on a navmesh: the spans under the box (grown by the agent radius) are blocked and only the tiles it touches, and their neighbours, are rebuilt. Agents on the mesh re-plan. World state.
     *
     * @param world The world.
     * @param mesh The navmesh.
     * @param id The obstacle's id, the game's own (a door's number).
     * @param obstacle The box; omitted removes the obstacle.
     * @returns Refused for an unknown navmesh.
     */
    function setObstacle(world: World, mesh: Navigation, id: number, obstacle?: NavBox): Result<NavRebuild>;
    /**
     * The point on a navmesh nearest a world point, within `extent` (half sizes).
     *
     * @param world The world.
     * @param mesh The navmesh.
     * @param point A world point.
     * @param extent The search box's half sizes; zero: 2 2 2.
     * @returns Null when no polygon is within the box.
     */
    function nearest(world: World, mesh: Navigation, point: Vector3, extent?: Vector3): Vector3 | null;
    /**
     * The path an agent follows now (its corners from where it planned), for debugging and playtests; `dump_nav:<entity>` writes the same.
     *
     * @param world The world.
     * @param entity An entity with an og.NavAgent.
     * @returns Null for an entity that has not planned.
     */
    function pathOf(world: World, entity: Entity): NavPath | null;
  }
  /**
   * An entity that thinks with a behaviour. Give it an og.NavAgent to move and an og.Perception to see and hear.
   */
  const AiAgent: ComponentType<{ readonly behaviour: FieldSpec<"Behaviour">; readonly active: FieldSpec<"bool">; readonly thinkEvery: FieldSpec<"u32">; readonly status: FieldSpec<"u32">; readonly state: FieldSpec<"u32">; readonly node: FieldSpec<"u32">; readonly changes: FieldSpec<"u32"> }, AiAgent>;
  /**
   * What an agent sees and hears. It sees the nearest og.Perceivable of another team within `sight`, inside the `fov` cone around `forward` (or the way it last moved), unless a collider is in the way; it hears a spatial sound that starts (og.SoundStarted) or a game noise (og.ai.noise) within `hearing` times its loudness, except its own: a sound of its entity or of an entity under it (og.Parent), or a noise whose NoiseDesc.source is one of them.
   */
  const Perception: ComponentType<{ readonly sight: FieldSpec<"f32">; readonly fov: FieldSpec<"f32">; readonly eyeHeight: FieldSpec<"f32">; readonly occlude: FieldSpec<"bool">; readonly occluders: FieldSpec<"u32">; readonly hearing: FieldSpec<"f32">; readonly memory: FieldSpec<"f32">; readonly team: FieldSpec<"u32">; readonly forward: FieldSpec<"Vector3">; readonly target: FieldSpec<"Entity">; readonly sees: FieldSpec<"bool">; readonly lastSeen: FieldSpec<"Vector3">; readonly sinceSeen: FieldSpec<"f32">; readonly heard: FieldSpec<"bool">; readonly heardAt: FieldSpec<"Vector3">; readonly sinceHeard: FieldSpec<"f32"> }, Perception>;
  /** Something agents of other teams can see. */
  const Perceivable: ComponentType<{ readonly team: FieldSpec<"u32">; readonly eyeHeight: FieldSpec<"f32"> }, Perceivable>;
  /** An agent's state or running node changed (systems read it the tick after). */
  const AiChanged: EventType<AiChanged>;
  /**
   * Enemy and NPC AI as data: behaviour trees and state machines in `.ogt` files of `type behaviour` (og-ai §3 has every line), perception (sight cones checked against physics occlusion, hearing of spatial sounds and game noises) and a blackboard per agent.
   * An entity with an og.AiAgent (and an og.Transform) thinks in the og.ai system before the game's systems: perception first, then its tree or machine, whose running action moves it through its og.NavAgent (og.nav plans and walks after the game's systems) or runs one of the game's commands (GameDesc.commands, with the agent's entity handle as its first argument) before the next tick.
   * Conditions read blackboard keys (declared, set by actions or og.ai.set, or built in: self, position, sees, heard, target, targetPosition, targetDistance, lastSeen, sinceSeen, heardAt, sinceHeard, elapsed), the agent's component fields (`Health.hp`) and resource fields (`Game.alarm`), and compare them with a literal or with another key or field read each time (`check Foe.hp below Tuning.fleeHp`), so thresholds live in a tuning resource. og.ai.restart starts a behaviour afresh. Deterministic; memories and blackboards are in snapshots. `dump_ai:<entity>` writes an agent's state, running node and blackboard.
   */
  namespace ai {
    /**
     * A cooked behaviour (an `.ogt` of `type behaviour`, validated by og cook).
     *
     * @param world The world.
     * @param path The behaviour asset, such as `ai/slime.ogt`.
     * @returns Refused for a path that is not a cooked behaviour.
     */
    function load(world: World, path: string): Result<BehaviourInfo>;
    /**
     * A behaviour from text, now (tests and generated behaviours; assets go through og cook).
     *
     * @param world The world.
     * @param text The behaviour's `.ogt` text.
     * @returns Refused with the first error's line and the fix.
     */
    function parse(world: World, text: string): Result<BehaviourInfo>;
    /**
     * A blackboard value of an agent as of its last think.
     *
     * @param world The world.
     * @param entity The agent.
     * @param key The key.
     * @returns A number, a boolean, a point `{x, y, z}`, an entity handle, or null for an unknown key or an agent that has not thought.
     */
    function get(world: World, entity: Entity, key: string): unknown;
    /**
     * Set a blackboard value of an agent: a number, a boolean, a point `{x, y, z}` or an entity handle `{entity}`.
     *
     * @param world The world.
     * @param entity The agent.
     * @param key The key; built-in keys are refused.
     * @param value The value.
     * @returns Refused for an agent that has not thought yet, a built-in key or another kind of value.
     */
    function set(world: World, entity: Entity, key: string, value: unknown): Result<void>;
    /**
     * Start an agent's behaviour afresh at its next think, as a new behaviour would: a machine in its initial state (its enter actions run again), a tree at its root, every timer and cooldown reset, the blackboard back to the declared keys, wander's centre where it then stands, and what it saw and heard forgotten. Actions without a speed keep the speed it first started with. For a new run or a respawn; an agent that has not thought yet starts afresh anyway.
     *
     * @param world The world.
     * @param entity The agent.
     * @returns Refused for a world without og.ai.
     */
    function restart(world: World, entity: Entity): Result<void>;
    /**
     * Make a noise agents may hear next tick (a gunshot, a door, a thrown stone).
     *
     * @param world The world.
     * @param desc Where and how loud.
     * @returns Refused for a world without og.ai.
     */
    function noise(world: World, desc?: NoiseDesc): Result<void>;
    /**
     * What an agent is doing: its state, the node it runs and the path to it, its status, its blackboard and its target.
     *
     * @param world The world.
     * @param entity The agent.
     * @returns Null for an entity that has not thought.
     */
    function describe(world: World, entity: Entity): AiDescription | null;
  }
  /**
   * An answer arrived: sent by og.platform at the start of the tick it was applied, read by systems the tick after (og.platform.result has it at once).
   */
  const PlatformAnswered: EventType<PlatformAnswered>;
  /**
   * roomCreate or roomJoin finished, or the host put this engine in a room (`og run --room`, a Play link to a room): sent at the start of the tick after the host answered. With ok false the engine is in no room and `reason` says why.
   */
  const RoomJoined: EventType<RoomJoined>;
  /** Another peer joined this engine's room. */
  const RoomPeerJoined: EventType<RoomPeerJoined>;
  /**
   * A peer left this engine's room (it left, closed, or lost its connection), after every message it sent; or, with peer equal to this engine's own number, this engine is in no room any more (roomLeave, or the host lost the room).
   */
  const RoomPeerLeft: EventType<RoomPeerLeft>;
  /**
   * Another peer's message arrived; read it with og.platform.roomMessage. Messages from one peer arrive in the order it sent them. While lockstep runs (og.Lockstep) a message is this peer's host input alone: a system that changes the world from one makes it differ from the other peers' (og.Desync); carry what all must see in og.PlayerInput.
   */
  const RoomMessage: EventType<RoomMessage>;
  /**
   * Platform services as host capabilities (docs/design.md §2.8): cloud store, leaderboards, telemetry, Origin Point and AI marks. Game code names a method and its arguments, never a credential, a URL or a cookie: the host owns them. On the web the shell forwards each call to the Player through origin-checked postMessage (Player protocol 2, docs/web-export.md); natively the host answers from a local JSON store (`og run --platform-store`, default `<project>/.og/platform.json`).
   * Calls are asynchronous: each returns a request number at once, and the answer arrives at the start of a later tick, when og.platform sends PlatformAnswered; read it with og.platform.result. Answers are host input, like key presses: the probe's `dump_platform` lists every call, its arguments, its answer and the ticks.
   * Errors carry the Player's stable codes: offline, timeout, invalid, unauthorized, not_found, conflict, too_large, rate_limited, unavailable, failed; retryable is true for offline, timeout, unavailable and rate_limited.
   * Rooms (roomCreate, roomJoin, roomSend, ...) put this engine with other players' engines: the host carries them (natively `og relay` with `og run --room`, on the web the platform's rooms through the shell, docs/platform-asks.md) and its answers and the other peers' messages arrive as events at the start of a tick (og.RoomJoined, og.RoomPeerJoined, og.RoomPeerLeft, og.RoomMessage), host input like key presses. Deterministic play over a room is og.net's lockstep.
   */
  namespace platform {
    /**
     * Read one key of the player's store for this game.
     *
     * @param world The world.
     * @param key The key.
     * @returns Refused for a malformed key. The answer's data is `{ key, value, revision }` (value null and revision 0 for a key never written).
     */
    function storeGet(world: World, key: string): Result<number>;
    /**
     * Write one key of the player's store for this game; with a revision, only if nobody wrote it since (another device, another tab).
     *
     * @param world The world.
     * @param desc The key, the value and the revision it must be at.
     * @returns Refused for a malformed key or a value over 64 KiB. The answer's data is `{ key, revision }` (the new revision); `conflict` when the key moved past desc.revision.
     */
    function storeSet(world: World, desc?: StoreSetDesc): Result<number>;
    /**
     * The keys of the player's store and their revisions.
     *
     * @param world The world.
     * @returns The answer's data is `{ keys: [{ key, revision }] }`.
     */
    function storeList(world: World): number;
    /**
     * Submit a score; the board keeps the player's best.
     *
     * @param world The world.
     * @param desc The board, the score and its order.
     * @returns Refused for a score that is not finite. The answer's data is `{ board, rank, best }`, or `{ board, rank: null, queued: true }` while offline.
     */
    function leaderboardSubmit(world: World, desc?: LeaderboardSubmitDesc): Result<number>;
    /**
     * The best entries of a board, and the player's own.
     *
     * @param world The world.
     * @param desc The board and how many.
     * @returns The answer's data is `{ board, order, entries: [{ rank, name, score, me }], me }`.
     */
    function leaderboardTop(world: World, desc?: LeaderboardQueryDesc): number;
    /**
     * The entries around the player's rank.
     *
     * @param world The world.
     * @param desc The board and how many around the player.
     * @returns The answer's data is the leaderboardTop shape with the entries around the player's rank.
     */
    function leaderboardAround(world: World, desc?: LeaderboardQueryDesc): number;
    /**
     * Record a game event for the game's analytics.
     *
     * @param world The world.
     * @param name The event: `level.started`, `level.completed`, `boss.defeated` (letters, digits, `.`, `_`; the Player files it as `game.custom.<name>`).
     * @param attrs Its attributes: an object of at most 16 strings (up to 512 characters), numbers, booleans or null, keys a lowercase letter then letters, digits and `_`.
     * @returns Refused for a malformed name or attributes. Fire and forget: nothing answers.
     */
    function telemetry(world: World, name: string, attrs?: unknown): Result<void>;
    /**
     * Ask for Origin Point for something the player did.
     *
     * @param world The world.
     * @param desc Why and how many.
     * @returns Refused for a reason outside 1 to 64 of letters, digits, `.`, `_` and `-`. The answer's data is `{ granted, balance }`; under a Player today always `{ granted: 0, balance: null }` (the platform grants nothing until a release carries an approved point policy), locally the amount asked.
     */
    function point(world: World, desc?: PointDesc): Result<number>;
    /**
     * Disclose AI-generated content the game shows.
     *
     * @param world The world.
     * @param desc What was generated.
     * @returns Refused for an empty kind or label. The answer's data is `{ marked: true }`.
     */
    function aiMark(world: World, desc?: AiMarkDesc): Result<number>;
    /**
     * Ask for the game's saved settings (volume, controls, language): store.get of the key `og.settings`. Where the Player has no store, or is offline, the shell answers from the browser's own storage; natively, from the local store.
     *
     * @param world The world.
     * @returns The request; its answer's data is `{ key, value, revision }` (value null before the first save).
     */
    function loadSettings(world: World): number;
    /**
     * Save the game's settings under `og.settings` (the last save wins), with the same local fallback as loadSettings.
     *
     * @param world The world.
     * @param value The settings: an object, at most 64 KiB encoded.
     * @returns Refused for a value that is not an object or is too large.
     */
    function saveSettings(world: World, value: unknown): Result<number>;
    /**
     * Save a long text under a slot, in as many store keys as it needs: a mid-size 3D world's snapshot is over a key's 64 KiB (examples/kestrel-stair's 93 entities: about 78 KB). The parts go in the bank of keys the slot's last save did not use (`<slot>.a*` or `<slot>.b*`), then the manifest (`<slot>`: the bank, the part count, the length, a checksum and `meta`) is written last, at the revision read before: so a save cut short leaves the last one whole, and another device's save landing meanwhile is a conflict, never a mix. One request for all of it; a slot takes at most 17 of the player's 64 keys for this game. og-game §17.
     *
     * @param world The world.
     * @param desc The slot, the text and what a title shows of it.
     * @returns Refused for a malformed slot name, a text over 512 KiB or a meta over 4 KiB. The answer arrives when the manifest is written: data `{ slot, parts, bytes, revision }`; an error is the first refused part's, or the manifest's (`conflict`: another save of the slot landed meanwhile; the slot holds that one).
     */
    function saveSlot(world: World, desc?: SaveSlotDesc): Result<number>;
    /**
     * Read a slot saveSlot wrote: its manifest, then its parts, joined and checked against the manifest's length and checksum. One request for all of it.
     *
     * @param world The world.
     * @param desc The slot, and whether to read its meta alone.
     * @returns Refused for a malformed slot name. The answer's data is `{ slot, text, meta, parts, bytes, revision }`: text and meta null for a slot never saved, text null with metaOnly; `invalid` for a key that holds no slot manifest or parts that do not match it.
     */
    function loadSlot(world: World, desc?: LoadSlotDesc): Result<number>;
    /**
     * The answer to a call, once it arrived.
     *
     * @param world The world.
     * @param request The call.
     * @returns Null while it is in flight, or for an unknown request.
     */
    function result(world: World, request: number): PlatformResult | null;
    /**
     * What answers platform calls here, and what it answers.
     *
     * @param world The world.
     */
    function info(world: World): PlatformInfo;
    /**
     * The host's wall-clock time, UTC milliseconds since 1970, as of this tick: what the host last told the engine (when it attached, and with every answer, applied at a tick's start like the answers) moved on by the ticks since. The one wall-clock read game code may make: it is host input, so a replay with the same answers gives the same values. Use it to know how long the game was closed (a save's time against a launch's), never to time play (that is og.Time).
     *
     * @param world The world.
     * @returns UTC milliseconds; 0 until a host has told the time (`none`).
     */
    function now(world: World): number;
    /**
     * Create a room and join it. The answer is og.RoomJoined, with the room's id to share (natively the relay names it `room-N`; on the web the platform does).
     *
     * @param world The world.
     * @param desc Its name and capacity.
     * @returns Refused with no room host (og.platform.roomInfo's host `none`), while in or joining a room (roomLeave first), or for a malformed name.
     */
    function roomCreate(world: World, desc?: RoomDesc): Result<void>;
    /**
     * Join a room by its id. The answer is og.RoomJoined (with ok false and `not_found` or `full` when it cannot); then og.RoomPeerJoined and og.RoomPeerLeft as others come and go.
     *
     * @param world The world.
     * @param room The room's id: 1 to 64 of letters, digits, `.`, `_` and `-`.
     * @returns Refused with no room host, while in or joining a room, or for a malformed id.
     */
    function roomJoin(world: World, room: string): Result<void>;
    /**
     * Leave the room: og.RoomPeerLeft with this engine's own peer number answers, and the other peers hear it leave after its last message.
     *
     * @param world The world.
     * @returns Refused when in no room.
     */
    function roomLeave(world: World): Result<void>;
    /**
     * Send a message to every other peer in the room, or to one: a JSON value, or bytes (a Uint8Array). It arrives as og.RoomMessage, after every earlier message of this peer.
     *
     * @param world The world.
     * @param data What to send: a Uint8Array, or any JSON value; at most 64 KiB encoded.
     * @param to The one peer to send it to; omitted is every other peer.
     * @returns Refused when in no room, for a message over 64 KiB, or a peer not in the room.
     */
    function roomSend(world: World, data: Uint8Array | unknown, to?: number): Result<void>;
    /**
     * The peers in this engine's room, this one included, by number; empty in none.
     *
     * @param world The world.
     */
    function roomPeers(world: World): readonly RoomPeer[];
    /**
     * The room this engine is in, and what carries rooms here.
     *
     * @param world The world.
     */
    function roomInfo(world: World): RoomInfo;
    /**
     * A message og.RoomMessage announced: who sent it and what it carries. The engine keeps the last 256.
     *
     * @param world The world.
     * @param message og.RoomMessage.message.
     * @returns Null for a number never given, or a message older than the last 256.
     */
    function roomMessage(world: World, message: number): RoomMessageData | null;
  }
  /**
   * Localization: one string table per locale (`.ogt` of `type strings`: `locale "ja"`, its font chain `fonts "..."`, `string "key" "message"`), messages in a subset of ICU MessageFormat (`{name}`, `{n, number}`, `{n, plural, =0 {..} one {..} other {..}}` with `#`, `{g, select, a {..} other {..}}`), CLDR plural rules, and the locale the host asks for (the browser's `navigator.language` on the web, `og run --locale` natively).
   * The active locale is the best loaded table for the request (ja-JP takes ja; zh-TW takes zh-Hant or zh-TW), else the base table (`base true`, else en, else the first). A key it lacks is read from its `fallback` table, then the base. It also sets the UI's font chain (its fonts first, then the other tables') and direction: a right-to-left locale (ar, he, fa, ur) mirrors og.ui's layout, and og-ui shapes Arabic words with the font's own tables (a font cooked with the `arabic` charset carries them). `og cook` warns about every key a locale lacks, or takes other arguments for, than the base.
   */
  namespace i18n {
    /**
     * Load a cooked string table; the active locale is negotiated again.
     *
     * @param world The world.
     * @param path The table, such as `strings/ja.ogt`.
     * @returns Refused for a path that is not a cooked string table, or a font of its chain that is not cooked.
     */
    function load(world: World, path: string): Result<LocaleInfo>;
    /**
     * Ask for a locale (a player's choice in settings); the active one is the best table for it.
     *
     * @param world The world.
     * @param locale A BCP 47 tag: `ja`, `zh-CN`, `ar-EG`.
     */
    function setLocale(world: World, locale: string): LocaleInfo;
    /**
     * The requested and active locales, the direction and the font chain.
     *
     * @param world The world.
     */
    function locale(world: World): LocaleInfo;
    /**
     * A key's message in the active locale, formatted with its arguments; a key no table has answers the key itself and logs one warning.
     *
     * @param world The world.
     * @param key The key, such as `menu.start`.
     * @param args The arguments: an object of strings and numbers, `{ count: 3, name: "Kai" }`.
     */
    function t(world: World, key: string, args?: unknown): string;
    /**
     * Whether the active locale's own table has the key (not a fallback).
     *
     * @param world The world.
     * @param key The key.
     */
    function has(world: World, key: string): boolean;
  }
  /**
   * The engine's UI: an immediate-mode toolkit. Systems describe the tree every tick with these calls; the engine lays it out Clay-style, styles it with the game's UI style (the placeholder prototype style until the game picks one), draws it over the frame and answers the probe (dump_ui, touch_move_to, move_to_text) from it.
   * The engine opens the frame before the game's systems (from og.Input and the play surface) and closes it after them, so calls belong inside systems; outside one they do nothing and log a warning. Interaction is decided against the previous tick's rectangles, which are also what the probe resolves, so a scripted touch_move_to and a click agree; response edges (clicked, changed, submitted) last one tick.
   * Lengths are CSS pixels of the play surface (origin top left, y down), the unit of og.Input and the probe. Ids: an element's id is unique among its siblings (`idi("Slot", 3)` is `Slot#3`); a path `Inventory/Slot#3` names it after its nearest ancestors. The UI state (hover, press, focus, caret, scroll) is world state: snapshots and the world hash carry it.
   */
  namespace ui {
    /**
     * Set how the UI looks (the scheme, density, fonts, style, sounds, motion…), typically in GameDesc.start; it takes effect from the next tick. The desc is a partial: a field left out keeps the last UiDesc's value, the one an earlier call or a look's style (og.render.applyLook) gave it, so a settings screen that configures sounds or reducedMotion keeps the style and the fonts; null clears a field to its default (`style: null` goes back to the prototype style). A UI left with no font at all (no font, no style with fonts, no fallback or locale font) warns: it would draw no text.
     *
     * @param world The world whose UI is configured.
     * @param desc The fields to change; the others keep their values.
     * @returns Refused for a dead font or style, or a desc that is not an object.
     */
    function configure(world: World, desc?: UiDesc): Result<void>;
    /**
     * This tick's play surface: the render target's size in CSS pixels, its DPR and safe area, as the host reports them.
     *
     * @param world The world.
     */
    function viewport(world: World): Viewport;
    /**
     * The style values the UI draws with.
     *
     * @param world The world.
     */
    function theme(world: World): UiTheme;
    /**
     * Open a container; the calls until its close() build its children.
     *
     * @param id Its id, unique among its siblings.
     * @param desc Its layout and look.
     */
    function open(id: string, desc?: ElementDesc): void;
    /**
     * Open a card, closed by close(): og.ui.panel without the callback.
     *
     * @param id Its id.
     * @param desc Its layout and look.
     */
    function openPanel(id: string, desc?: PanelDesc): void;
    /**
     * Open a vertical scroll container, closed by close(): og.ui.scroll without the callback.
     *
     * @param id Its id.
     * @param desc Its size and spacing.
     */
    function openScroll(id: string, desc?: ScrollDesc): void;
    /**
     * Close the open container.
     *
     * @returns Its response, for an interactive container; the zero response otherwise.
     */
    function close(): Response;
    /**
     * A row built by `build`: open(id, {...desc, direction: row}), build(), close().
     *
     * @param id Its id.
     * @param desc Its layout and look; direction is row.
     * @param build Makes its children.
     * @returns Its response, for an interactive row.
     */
    function row(id: string, desc: ElementDesc, build: UiBuild): Response;
    /**
     * A column built by `build`: open(id, {...desc, direction: column}), build(), close().
     *
     * @param id Its id.
     * @param desc Its layout and look; direction is column.
     * @param build Makes its children.
     * @returns Its response, for an interactive column.
     */
    function column(id: string, desc: ElementDesc, build: UiBuild): Response;
    /**
     * A card built by `build`: openPanel(id, desc), build(), close().
     *
     * @param id Its id.
     * @param desc Its layout and look.
     * @param build Makes its children.
     */
    function panel(id: string, desc: PanelDesc, build: UiBuild): void;
    /**
     * A vertical scroll container built by `build`: openScroll(id, desc), build(), close().
     *
     * @param id Its id.
     * @param desc Its size and spacing.
     * @param build Makes its children.
     */
    function scroll(id: string, desc: ScrollDesc, build: UiBuild): void;
    /**
     * A paragraph of inline spans from markup, for a string-table message (og.i18n.t) so a translation carries its own emphasis: `[b]bold[/b]` (thickened in the distance field as far as it reaches, plus a second copy one device pixel right), `[color=danger]` or `[color=#ff8800]` (a ThemeColor follows the style), `[size=xl]` (a TypeStepName), `[font=mono]`, `[icon=coin]` (a cell of the icon atlas, one em), `[link=shop]Open the shop[/link]` (accent ink, underlined, focusable by keys and pads; a child element `<id>/shop` of kind link for the probe) and per-glyph motion `[wave]`, `[shake]`, `[pulse]` (by the tick; still under reduced motion). `[[` is a literal bracket. Lines break between the spans' words (a word restyled mid-way stays whole), each as tall as its tallest span on one baseline. Markup it cannot read shows as text and logs a warning.
     *
     * @param id Its id.
     * @param markup The markup.
     * @param desc Defaults for its spans: type step, ink, alignment, wrapping, width and font.
     */
    function richText(id: string, markup: string, desc?: RichTextDesc): RichTextResult;
    /**
     * MSDF text that wraps at its width (word breaks; between CJK characters under kinsoku, katakana words kept whole; long words between characters) unless nowrap. The probe reads its text.
     *
     * @param id Its id.
     * @param text The text.
     * @param desc Type step, ink, alignment, wrapping, width and font.
     */
    function label(id: string, text: string, desc?: LabelDesc): void;
    /**
     * A button with a label; `clicked` is a press and a release inside it. The probe reads its label.
     *
     * @param id Its id.
     * @param text The label.
     * @param desc Variant, size, disabled and width.
     */
    function button(id: string, text: string, desc?: ButtonDesc): Response;
    /**
     * A texture region, or a solid rectangle; with ImageDesc.aspect, a fitting side follows the other.
     *
     * @param id Its id.
     * @param desc Texture, region, tint, size and aspect ratio.
     */
    function image(id: string, desc?: ImageDesc): void;
    /**
     * A zoomable picture (a map, an artwork): two fingers pinch it to zoom about their centre and pan it as they move (og.Pointers), the wheel zooms about the pointer, a drag pans; Tab or a pad focuses it, and then + (= and the keypad's) and - zoom it about its centre, 0 fits it again and the arrows pan it a tenth of its size. It stays on its picture and within its zoom limits; its zoom and centre are UI state (snapshots carry them). Two fingers over a scroll container scroll it the same way, and fling it as they lift. The probe reads its zoom as its value.
     *
     * @param id Its id.
     * @param desc The picture, its size and zoom limits.
     */
    function zoomView(id: string, desc?: ZoomViewDesc): ZoomResult;
    /**
     * A track filled to a fraction.
     *
     * @param id Its id.
     * @param value How full it is, from 0 to 1.
     * @param desc Its width.
     */
    function progress(id: string, value: number, desc?: ProgressDesc): void;
    /**
     * A track with a fill and a thumb: a press anywhere on it sets the value and dragging follows. Pass back the value it answers next tick.
     *
     * @param id Its id.
     * @param value The value from game state.
     * @param desc Range, step and width.
     */
    function slider(id: string, value: number, desc?: SliderDesc): SliderResult;
    /**
     * A switch with a label; a click flips it. Pass back the state it answers next tick. The probe reads its label.
     *
     * @param id Its id.
     * @param on The state from game state.
     * @param label The label.
     */
    function toggle(id: string, on: boolean, label: string): ToggleResult;
    /**
     * One line of text: a press focuses it with the caret at the press, a drag selects (and scrolls the text past the field's ends), a double click selects a word and a triple click everything; typed text (og.input's TextInput events: the page's typing, the probe's type: and commit:; until a world has had one, a printable key pressed types its US-layout character, and after it keys only edit) replaces the selection at the caret; Backspace, Delete, arrows, Home and End edit, Shift with them selects, Alt or Ctrl with them moves or deletes by words (Ctrl with Home and End: the text's ends); Ctrl or Cmd with A selects all, with C copies and with X cuts (the page's clipboard, through the shell), with Z undoes and with Shift+Z or Y redoes (a history of 100 steps per field, typing undone a word at a time; a text the game sets starts it again); an input method's composition shows underlined at the caret until it commits; Enter submits and leaves, Escape leaves. The text scrolls to keep the caret in view and the caret blinks by the tick (steady under reduced motion). Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty.
     *
     * @param id Its id.
     * @param value The text from game state.
     * @param desc Placeholder, width and length limit.
     */
    function textField(id: string, value: string, desc?: FieldDesc): FieldResult;
    /**
     * Several lines of text: textField's editing, with Enter starting a new line, Up and Down moving between lines and Home and End to a line's ends, a triple click selecting the line between its breaks, the text wrapped between words at the area's width and scrolled to keep the caret's line in view (a drag above or below the area scrolls it, faster the further out). Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty.
     *
     * @param id Its id.
     * @param value The text from game state.
     * @param desc Placeholder, width, rows and length limit.
     */
    function textArea(id: string, value: string, desc?: TextAreaDesc): FieldResult;
    /**
     * A tooltip for the element just built: shown below it (above without room) once the pointer has rested on it for the slow motion duration, drawn over everything. The probe reads its text.
     *
     * @param text The text.
     */
    function tooltip(text: string): void;
    /**
     * The id of the index-th of several siblings of one name: `idi("Slot", 3)` is `Slot#3`.
     *
     * @param name The name.
     * @param index The index.
     */
    function idi(name: string, index: number): string;
    /**
     * The size class of the play surface of the frame being built (desktop outside a frame).
     */
    function breakpoint(): Breakpoint;
    /**
     * The play surface of the frame being built: og.ui.viewport without the world, for code that only builds UI.
     */
    function frameViewport(): Viewport;
    /**
     * Whether the icon atlas has an icon of this name (templates draw a fallback otherwise).
     *
     * @param name The icon's name.
     */
    function hasIcon(name: string): boolean;
    /**
     * The UI's clock in seconds: frames built over the tick rate (simulation time, never the wall clock).
     */
    function seconds(): number;
    /**
     * The style values of the frame being built: og.ui.theme without the world, for code that only builds UI (the templates).
     */
    function frameTheme(): UiTheme;
    /**
     * Open an anchored group, closed by close(): og.ui.anchor without the callback.
     *
     * @param id Its id; the probe path is the id alone.
     * @param desc Where it sits and how it lays out.
     */
    function openAnchor(id: string, desc?: AnchorDesc): void;
    /**
     * An anchored group built by `build`: a layer at a point of the safe area, over the tree, for HUD corners and touch controls.
     *
     * @param id Its id.
     * @param desc Where it sits and how it lays out.
     * @param build Makes its children.
     */
    function anchor(id: string, desc: AnchorDesc, build: UiBuild): void;
    /**
     * Open a transition container (a column), closed by close(); build its children only while the answer is visible: it stays visible while it leaves. Its content takes no input while it moves.
     *
     * @param id Its id.
     * @param show Whether it should be shown.
     * @param desc How it moves.
     */
    function openTransition(id: string, show: boolean, desc?: TransitionDesc): Presence;
    /**
     * Content that enters and leaves: fade, slide or scale by the tick from the style's motion. `build` runs while it is visible.
     *
     * @param id Its id.
     * @param show Whether it should be shown.
     * @param desc How it moves.
     * @param build Makes its content.
     */
    function transition(id: string, show: boolean, desc: TransitionDesc, build: UiBuild): Presence;
    /**
     * Open a dialog, closed by closeModal(); build its content only while the answer is visible. The probe sees the backdrop as `<id>` and the panel as `<id>/Panel`.
     *
     * @param id Its id.
     * @param open Whether it should be open.
     * @param desc Title, width and dismissal rules.
     */
    function openModal(id: string, open: boolean, desc?: ModalDesc): ModalResult;
    /** Close a dialog opened with openModal (its panel and backdrop). */
    function closeModal(): void;
    /**
     * A dialog built by `build` while visible: scrim, centred panel, focus kept inside, Back and the backdrop dismiss it unless locked. Set your open flag false on `dismissed`.
     *
     * @param id Its id.
     * @param open Whether it should be open.
     * @param desc Title, width and dismissal rules.
     * @param build Makes its content.
     */
    function modal(id: string, open: boolean, desc: ModalDesc, build: UiBuild): ModalResult;
    /**
     * A spotlight over everything (a tutorial step): the scrim on four sides of the hole, a primary edge around it. Presses on the dimmed parts stop there (clicked); presses in the hole reach what is under it. Shades are `<id>/Top`, `Left`, `Right`, `Bottom`; the hole is `<id>/Hole`.
     *
     * @param id Its id.
     * @param hole The rectangle to leave lit, as og.ui.bounds answers; null dims everything.
     * @param desc Room around the hole.
     */
    function spotlight(id: string, hole: UiRect | null, desc?: SpotlightDesc): Response;
    /**
     * Where an element was in the last frame, by id or `Parent/Child` path (topmost first), as touch_move_to resolves it.
     *
     * @param path The id or path.
     */
    function bounds(path: string): UiRect | null;
    /**
     * Open a virtualised list (a scroll column) and learn which rows to build: listItem(i) and close() for each, then closeList().
     *
     * @param id Its id.
     * @param desc Row count and size.
     */
    function openList(id: string, desc?: ListDesc): ListRange;
    /**
     * Open a virtualised grid and learn which cells to build: gridItem(i) and close() for each, then closeList().
     *
     * @param id Its id.
     * @param desc Cell count, size and columns.
     */
    function openGrid(id: string, desc?: GridDesc): ListRange;
    /**
     * Open row `index` of the open list (`Row#index`): selectable and focusable; its content follows until close(). A press that started on one row and is released on this one answers `dropped`.
     *
     * @param index Its index.
     * @param selected Drawn selected (controlStrong).
     */
    function listItem(index: number, selected: boolean): Response;
    /**
     * Open cell `index` of the open grid (`Cell#index`): a slot on control, focusable; its content follows until close(). Drops answer `dropped` as listItem's.
     *
     * @param index Its index.
     * @param selected Drawn selected (controlStrong with a primary bar).
     */
    function gridItem(index: number, selected: boolean): Response;
    /** Close a list or grid opened with openList or openGrid. */
    function closeList(): Response;
    /**
     * The row or cell a held press started on in the list or grid of this id (built under the same parent): a drag in progress, to draw what is dragged under the pointer.
     *
     * @param id The list's or grid's id.
     */
    function dragging(id: string): number | null;
    /**
     * A virtualised list built by `build(index)` for the rows in view.
     *
     * @param id Its id.
     * @param desc Row count, size and the selected row.
     * @param build Makes one row's content.
     */
    function list(id: string, desc: ListDesc, build: UiItemBuild): ListResult;
    /**
     * A virtualised grid built by `build(index)` for the cells in view.
     *
     * @param id Its id.
     * @param desc Cell count, size, columns and the selected cell.
     * @param build Makes one cell's content.
     */
    function grid(id: string, desc: GridDesc, build: UiItemBuild): ListResult;
    /**
     * A segmented control: a neutral track, the chosen tab on the thumb. Tabs are `Tab#i` and the probe reads their labels.
     *
     * @param id Its id.
     * @param items The tab labels.
     * @param selected The chosen index from game state.
     * @param desc Fill and width.
     */
    function tabs(id: string, items: readonly string[], selected: number, desc?: TabsDesc): ChoiceResult;
    /**
     * A select: a field-like trigger with the chosen label and a chevron; a press or Activate opens its menu `<id>.menu` (options `Option#i`) under it, a focus scope; choosing closes it with changed. Back or a press outside closes it.
     *
     * @param id Its id.
     * @param options The option labels.
     * @param selected The chosen index from game state.
     * @param desc Placeholder, width, disabled.
     */
    function select(id: string, options: readonly string[], selected: number, desc?: SelectDesc): ChoiceResult;
    /**
     * A number stepper: `Dec`, `Value`, `Inc`. The buttons step within the range; Left and Right step it while a button is focused; a screen reader's increment and decrement on the stepper itself (its spinbutton, which the web shell mirrors as an adjustable range) step it too.
     *
     * @param id Its id.
     * @param value The value from game state.
     * @param desc Range, step and digits.
     */
    function stepper(id: string, value: number, desc?: StepperDesc): StepperResult;
    /**
     * A radio group of `Option#i` rows: a ring, filled primary when chosen, and a label.
     *
     * @param id Its id.
     * @param options The option labels.
     * @param selected The chosen index from game state.
     * @param desc Row or column.
     */
    function radio(id: string, options: readonly string[], selected: number, desc?: RadioDesc): ChoiceResult;
    /**
     * A checkbox with a label; a press flips it.
     *
     * @param id Its id.
     * @param on The state from game state.
     * @param label The label.
     */
    function checkbox(id: string, on: boolean, label: string): ToggleResult;
    /**
     * Checkboxes in a column (`Option#i`).
     *
     * @param id Its id.
     * @param labels The labels.
     * @param values The states from game state.
     */
    function checkboxList(id: string, labels: readonly string[], values: readonly boolean[]): CheckListResult;
    /**
     * A chip: a control-filled tag; selected inverts to ink. For filters and tags.
     *
     * @param id Its id.
     * @param text The label.
     * @param desc Selected and disabled.
     */
    function chip(id: string, text: string, desc?: ChipDesc): Response;
    /**
     * A badge: a small status tag in its tone.
     *
     * @param id Its id.
     * @param text The label (a count, a word).
     * @param tone Its tone.
     */
    function badge(id: string, text: string, tone: Tone): void;
    /**
     * A hairline in the border colour across its parent.
     *
     * @param id Its id.
     * @param vertical Stand it up (in a row).
     */
    function divider(id: string, vertical: boolean): void;
    /**
     * Empty space.
     *
     * @param id Its id.
     * @param size Its size along both axes; null grows into the free space.
     */
    function spacer(id: string, size: Size | null): void;
    /**
     * A key (inkMuted) and its value (ink, at the end) on one row: stats, settings summaries, receipts.
     *
     * @param id Its id.
     * @param key The key.
     * @param value The value.
     * @param mono The value in the monospaced font (prices, ids).
     */
    function keyValue(id: string, key: string, value: string, mono: boolean): void;
    /**
     * A skeleton: the neutral fill standing in for content that is loading, with a sheen that passes every 1.6 s (still under reduced motion).
     *
     * @param id Its id.
     * @param desc Width, line height and lines.
     */
    function skeleton(id: string, desc?: SkeletonDesc): void;
    /**
     * A loading spinner: eight dots around a circle, the bright one turning by the tick.
     *
     * @param id Its id.
     * @param size Its diameter in CSS px; 0 is the small control height.
     */
    function spinner(id: string, size: number): void;
    /**
     * An icon of the atlas (UiDesc.icons) by name: a texture cell, or an icon set's distance-field glyph fitted to its square; an unknown name draws an empty slot and warns. Rich text takes the same icons inline with `[icon=name]`. The probe reads the name.
     *
     * @param id Its id.
     * @param name The icon's name.
     * @param desc Size and ink.
     */
    function icon(id: string, name: string, desc?: IconDesc): void;
    /**
     * A health, mana or xp bar. A drop shows at once and leaves a pale trail that follows after a beat; a rise grows into place with a ghost of the target; both by the tick.
     *
     * @param id Its id.
     * @param value The current value.
     * @param max The full value.
     * @param desc Kind, label, size and notches.
     */
    function meter(id: string, value: number, max: number, desc?: MeterDesc): void;
    /**
     * A cooldown: a disc whose primary ring fills clockwise from the top as it runs out, bright and full when ready, a label or icon in the middle. A press reports clicked (ready or not).
     *
     * @param id Its id.
     * @param remaining How much of the cooldown is left, 0 to 1: 1 just used, 0 ready.
     * @param desc Size, label and icon.
     */
    function cooldown(id: string, remaining: number, desc?: CooldownDesc): Response;
    /**
     * A mini-map frame: the map in a lineStrong frame, markers as dots (the player primary and ringed), an N mark and a zone label.
     *
     * @param id Its id.
     * @param desc Map, markers and label.
     */
    function minimap(id: string, desc?: MinimapDesc): void;
    /**
     * A dialogue box: a portrait slot, the speaker in primary ink and the line typed out by the tick. A press or Activate while typing shows the whole line; once shown it answers advanced. The line is `<id>/Body/Line` and the probe reads what is shown.
     *
     * @param id Its id.
     * @param desc Speaker, text, portrait and speed.
     */
    function dialogue(id: string, desc?: DialogueDesc): DialogueResult;
    /**
     * A virtual joystick: a base and a knob that follows a press that started on it. It keeps the finger that grabbed it (any of og.Pointers), so another finger can hold an action button at the same time.
     *
     * @param id Its id.
     * @param desc Size and dead zone.
     */
    function joystick(id: string, desc?: JoystickDesc): JoystickResult;
    /**
     * A d-pad of four arms (`Up`, `Down`, `Left`, `Right`), each at least 44 CSS px.
     *
     * @param id Its id.
     * @param desc Its size.
     */
    function dpad(id: string, desc?: DpadDesc): DpadResult;
    /**
     * A square touch action button, cornered by the style's control radius: pressed while held (jump, fire), clicked on a tap; at least 44 CSS px.
     *
     * @param id Its id.
     * @param text Its label (and what the probe reads).
     * @param desc Size, primary and icon.
     */
    function actionButton(id: string, text: string, desc?: ActionButtonDesc): Response;
    /**
     * Queue a toast: it stacks at UiDesc.toastAnchor, enters, stays its time and leaves; at most three are up (older ones leave first); a press dismisses it. Callable anywhere (systems, commands, start); it is world state, so snapshots carry it.
     *
     * @param world The world whose UI shows it.
     * @param text The message: a few words.
     * @param desc Tone and time.
     * @returns Its id, for dismissToast.
     */
    function toast(world: World, text: string, desc?: ToastDesc): number;
    /**
     * Start a toast's exit.
     *
     * @param world The world.
     * @param toastId The id toast() answered.
     */
    function dismissToast(world: World, toastId: number): void;
    /**
     * Queue a navigation intent for the next tick: a gamepad og.Input does not read yet, a custom binding, a command. A move queued while the focus is on nothing puts the focus on the first control, whatever its direction, and no further: moves asked again before the focus lands (each tick until og.ui.focus answers a path) land on that same first control.
     *
     * @param world The world.
     * @param intent The intent.
     */
    function navigate(world: World, intent: NavIntent): void;
    /**
     * Escape or gamepad B this tick, not taken by a dialog or menu: true once, then taken. A pause screen closes on it. While a dialog or menu that Back closes is open (the topmost of the last frame) it answers false and leaves Back to it, even when asked before the dialog is built this tick: a HUD drawn under pause never takes the Back that closes pause. A locked dialog lets Back through.
     */
    function back(): boolean;
    /**
     * Local multiplayer: how many players this screen takes (1 to 4; 1 by default). Pads 1 to count - 1 each move a focus of their own among the same controls, shown as a ring in the player's colour (info, success, warn), and their A presses the control under it (Response.pad says which pad); pad 0 shares the one focus with the keyboard and the pointer, and Back and adjusting a slider stay pad 0's. Declare it every tick the screen shows; it takes effect from the next tick, and a dialog's focus scope holds every pad's focus inside it.
     *
     * @param count Players, 1 to 4 (clamped).
     */
    function setPlayers(count: number): void;
    /**
     * The path of the control pad pad's focus is on in the last frame (pad 0: the shared focus of the pointer, keyboard and pad 0), as touch_move_to resolves it; empty when it is on none.
     *
     * @param pad The pad, 0 to 3; past 3 answers empty.
     */
    function focus(pad: number): string;
    /**
     * The accessibility tree of the last frame: every element with a role, its name, value, states and bounds, and the focus order Tab walks. A shell mirrors it into the page for screen readers; a playtest reads the same fields on dump_ui's elements.
     */
    function a11y(): A11yTree;
    /**
     * Whether keyboard and gamepad navigation moves focus on this screen (on by default). A gameplay HUD whose keys move the player turns it off; an open dialog or menu turns it back on while it is open. Takes effect from the next tick.
     *
     * @param on On or off.
     */
    function setNavigation(on: boolean): void;
    /**
     * A number the UI keeps for an id under the open element (a template's tab, a count-up's start tick), in snapshots like the rest of the UI state. Read it every tick it is used: an unused memory is dropped.
     *
     * @param id The id, unique among its siblings' memories.
     * @param initial Its value the first time.
     */
    function memory(id: string, initial: number): number;
    /**
     * Set a number memory (see memory).
     *
     * @param id The id.
     * @param value The value.
     */
    function remember(id: string, value: number): void;
    /** Frames the UI has built: its clock, for animations by the tick. */
    function tick(): number;
    /**
     * A number label that counts (ease-out cubic, by the tick) from where it was to the value each time the value changes, grouped with commas; a style with motion.pop bumps it when it lands. Scores, currency, rewards.
     *
     * @param id Its id.
     * @param value The value to count to.
     * @param desc Look, duration, start and format.
     * @returns The number shown this tick.
     */
    function counter(id: string, value: number, desc?: CounterDesc): number;
    /**
     * Throw rays and sparks from the element built just before (a star landing, a reward), once, from when it first appears. Off under reduced motion.
     *
     * @param desc Count, ink, reach and timing.
     */
    function burst(desc?: BurstDesc): void;
    /**
     * Shake the element built just before, from this tick (a wrong answer, a locked door, a refused purchase): call it once, on the tick it happens; the element swings and comes to rest by itself, and snapshots carry it. Still under reduced motion. A refused press on a disabled button already shakes when the style's motion.shake is set.
     *
     * @param desc Amount, duration and sound.
     */
    function shake(desc?: ShakeDesc): void;
    /**
     * A transition between game screens drawn over the whole frame (through the overlay, over every other UI layer). Pass the screen the game wants every tick and build the one it answers: when the wanted screen changes the cover closes over the old screen for half the duration, the answer switches, and the cover opens on the new one. Deterministic by the tick; snapshots carry it; under reduced motion every kind is a fade.
     *
     * @param id Its id.
     * @param screen The screen the game wants now.
     * @param desc Kind, duration, colour or texture, side.
     */
    function screenTransition(id: string, screen: string, desc?: ScreenTransitionDesc): ScreenTransitionResult;
    /**
     * Lay the UI out for another play surface (a phone in portrait at 3x, a notch) until cleared with null: for headless previews and playtests of responsive layouts. Render at width x dpr by height x dpr (`og run --size`).
     *
     * @param world The world.
     * @param viewport The surface to lay out for; null returns to the host's.
     */
    function previewViewport(world: World, viewport: Viewport | null): void;
    /**
     * The title screen template. Returns the item chosen this tick.
     *
     * @param id Its id.
     * @param desc Title, items and footer.
     */
    function mainMenu(id: string, desc?: MainMenuDesc): MenuResult;
    /**
     * The pause screen template: a dialog with Resume first; Back answers `resume`.
     *
     * @param id Its id.
     * @param desc Open and items.
     */
    function pauseMenu(id: string, desc?: PauseMenuDesc): MenuResult;
    /**
     * The settings template: Audio (volumes, mute), Graphics (quality, frame rate, reduced motion, contrast), Controls (sensitivity, invert, vibration, bindings) and Language. Pass the settings in every tick and keep what it answers.
     *
     * @param id Its id.
     * @param desc Open, the settings, languages and hooks.
     */
    function settingsScreen(id: string, desc?: SettingsDesc): SettingsResult;
    /**
     * The platformer HUD template.
     *
     * @param id Its id.
     * @param desc Lives, coins, score, level and timer.
     */
    function platformerHud(id: string, desc?: PlatformerHudDesc): HudResult;
    /**
     * The shooter HUD template.
     *
     * @param id Its id.
     * @param desc Health, armor, ammo, abilities and map.
     */
    function shooterHud(id: string, desc?: ShooterHudDesc): HudResult;
    /**
     * The puzzle HUD template.
     *
     * @param id Its id.
     * @param desc Level, score, target, stars, moves and timer.
     */
    function puzzleHud(id: string, desc?: PuzzleHudDesc): HudResult;
    /**
     * The idle and incremental HUD template: answers `tap` and `buy:<id>` actions.
     *
     * @param id Its id.
     * @param desc Currencies and generators.
     */
    function idleHud(id: string, desc?: IdleHudDesc): HudResult;
    /**
     * The racing HUD template.
     *
     * @param id Its id.
     * @param desc Speed, gear, laps, place, times, splits, boost and its words.
     */
    function racingHud(id: string, desc?: RacingHudDesc): HudResult;
    /**
     * The dialogue and cutscene template: letterbox, the dialogue box, Skip.
     *
     * @param id Its id.
     * @param desc Lines and the current index.
     */
    function cutscene(id: string, desc?: CutsceneDesc): CutsceneResult;
    /**
     * The shop template.
     *
     * @param id Its id.
     * @param desc Balance, items and categories.
     */
    function shopScreen(id: string, desc?: ShopDesc): ShopResult;
    /**
     * The inventory template: a grid of slots with drag and drop.
     *
     * @param id Its id.
     * @param desc Slots and items.
     */
    function inventoryScreen(id: string, desc?: InventoryDesc): InventoryResult;
    /**
     * The level select template: a card picked answers `chosen`; its Back button, a press on the scrim around it, Escape and the pad's B answer `closed`.
     *
     * @param id Its id.
     * @param desc The levels and the screen's words.
     */
    function levelSelect(id: string, desc?: LevelSelectDesc): LevelSelectResult;
    /**
     * The results template: stars, the score (points or a time) counting up under its label, the best, stats, actions and Share.
     *
     * @param id Its id.
     * @param desc Score, stars, stats and actions.
     */
    function resultsScreen(id: string, desc?: ResultsDesc): MenuResult;
    /**
     * The leaderboard template.
     *
     * @param id Its id.
     * @param desc Scopes, rows and states.
     */
    function leaderboardScreen(id: string, desc?: LeaderboardDesc): LeaderboardResult;
    /**
     * The onboarding template: a spotlight and a callout per step.
     *
     * @param id Its id.
     * @param desc Steps and the current index.
     */
    function tutorialOverlay(id: string, desc?: TutorialDesc): TutorialResult;
    /**
     * The loading template.
     *
     * @param id Its id.
     * @param desc Title, progress and tips.
     */
    function loadingScreen(id: string, desc?: LoadingDesc): void;
    /**
     * The confirm dialog template.
     *
     * @param id Its id.
     * @param desc Question, consequence and labels.
     */
    function confirmDialog(id: string, desc?: ConfirmDesc): ConfirmResult;
    /**
     * The alert template. Returns true the tick it is dismissed.
     *
     * @param id Its id.
     * @param desc Title, message and label.
     */
    function alertDialog(id: string, desc?: AlertDesc): boolean;
  }
  /**
   * Systems the engine ships as TypeScript: game-level code, listed explicitly in GameDesc.systems like the game's own, so the schedule (and dump_world) shows them and nothing runs that the game did not ask for.
   */
  namespace systems {
    /**
     * The camera controllers, in this order: og.orbitCamera (og.OrbitCamera), og.followCamera (og.FollowCamera) and og.follow2D (og.Follow2D). Each moves og.Camera only while an entity carries its component. List them before the game's systems that read og.Camera: `systems: [...og.systems.cameras, ...yours]`.
     */
    const cameras: readonly System[];
  }
  /**
   * The command channel: actions scheduled on simulation time, from `og run --commands "t:action[:args];..."` or the probe's commands().
   * Core actions ship with the engine; a module or a game registers more. Unknown actions warn and continue. Not in publish builds.
   * Outputs are relative to the output directory: an absolute path, a `..` component or a name starting with `.` (kept for the harness, such as a playtest's kept store) is refused. A file an earlier step of the script wrote is never overwritten: the later step's is written as `<stem>-2.<ext>` with a warning naming both steps.
   */
  namespace commands {
    /**
     * Register an action into the command channel. GameDesc.commands does this for a game's own actions.
     *
     * @param name The action name, snake_case, such as `spawn_wave`.
     * @param handler What it does.
     * @param desc Its usage line and doc.
     * @returns Refused for a name a core action or another registration already uses.
     */
    function register(name: string, handler: CommandHandler, desc?: CommandDesc): Result<void>;
    /**
     * Parse a script without scheduling it.
     *
     * @param script `t:action[:args];...` with t in simulation seconds.
     * @returns The steps, or the first malformed step.
     */
    function parse(script: string): Result<readonly CommandStep[]>;
    /**
     * Every action the channel knows: core actions in CoreAction order, then registered ones by name.
     *
     * @param world The world whose channel is asked.
     */
    function help(world: World): readonly CommandInfo[];
    /**
     * Schedule a script on simulation time, from the current tick.
     *
     * @param world The world.
     * @param script `t:action[:args];...` with t in simulation seconds.
     * @returns The report, or the first malformed step.
     */
    function schedule(world: World, script: string): Result<CommandsReport>;
  }
  /**
   * Draws a loaded terrain placed by the entity's world transform (centred on its position, turned and scaled with it), with its layers (as cooked, or as og.terrain.setLayer left them), casting shadows unless told not to.
   */
  const Terrain: ComponentType<{ readonly data: FieldSpec<"TerrainData">; readonly castShadows: FieldSpec<"bool">; readonly lodSpacing: FieldSpec<"f32">; readonly lodRing: FieldSpec<"u32"> }, Terrain>;
  /**
   * Scatters copies of a mesh (grass, flowers, stones) over a loaded terrain near the camera, where one of its layers shows: on a jittered grid fixed to the terrain, so each copy keeps its place, size and turn as the camera moves, every cell growing one with the probability of the layer's weight there (its control-map channel times its slope and altitude rules, over all the layers' weights). Copies keep their size and thin out (dithered) over the outer `fade` of `distance`. The renderer places them itself, a chunk of the grid at a time, once, and draws the chunks in range: copies cost about nothing per frame while the camera stays. They grow over the og.Terrain drawing the same data, placed as it is. Drawn only: no colliders, and the copies are not entities.
   */
  const TerrainScatter: ComponentType<{ readonly data: FieldSpec<"TerrainData">; readonly geometry: FieldSpec<"Geometry">; readonly material: FieldSpec<"Material">; readonly layer: FieldSpec<"u32">; readonly density: FieldSpec<"f32">; readonly distance: FieldSpec<"f32">; readonly fade: FieldSpec<"f32">; readonly minScale: FieldSpec<"f32">; readonly maxScale: FieldSpec<"f32">; readonly align: FieldSpec<"f32">; readonly seed: FieldSpec<"u32"> }, TerrainScatter>;
  /**
   * Terrain (phase 2): a height field over a rectangle, cooked by `og cook` from a `type terrain` .ogt (a 16-bit grey PNG or an EXR heightmap, an optional control map, up to four layered materials with slope and altitude rules, their maps separate textures or layers of two texture arrays: `layers` and `layerNormals` with each layer's `albedoLayer` and `normalLayer`, two sampled textures instead of up to eight) into tiles; drawn as a geometry clipmap around the camera (nested levels by distance, no cracks) that casts and receives shadows under the fog; collided with as a height field; baked into navmeshes by a navmesh file's `terrain` entry.
   * Load it with og.terrain.load, then place it: an og.Terrain on an entity draws it placed by the entity's world transform (centred on its position, turned and scaled with it), and an og.Collider of shape SHAPE_HEIGHTFIELD naming the TerrainInfo's geometry makes it solid. og.TerrainScatter grows grass, flowers or stones over a layer near the camera. og.terrain.sample answers the surface's height and normal anywhere (pass it the entity's position, quaternion and scale); og.terrain.edit raises, lowers, flattens, smooths or paints it at runtime, and og.terrain.setLayer retunes a layer. Edits are world state: a snapshot keeps them, the world hash covers them, and a restore brings the ground back as it was (rebuilt from the cook with the restored edits replayed, for the renderer, physics and sample alike).
   * Its tiles stream (phase 3): the cook stores the field as a pyramid of tiles, each compressed on its own, and every frame the renderer keeps on the GPU only the tiles the camera's clipmap wants, within each terrain's tile budget (16 MiB by default), decoded off the frame (a worker thread natively, the browser's DecompressionStream on the Web) a few at a time, the most needed first; until a tile arrives its area draws from the coarser level that holds it, then fades in. Nothing a tick reads depends on which tiles are resident (collision and og.terrain.sample read the whole field), so a run's world hash is the same streamed or not. The probe's `terrain_stream:resident` decodes every wanted tile before each frame instead (`terrain_stream:stream[:<MiB>]` streams again, within that budget), and `dump_stream`'s `terrain` gives the tiles resident, waiting, decoding and cached.
   */
  namespace terrain {
    /**
     * Load a cooked terrain (a `type terrain` .ogt under assets/) and its layers' textures (or texture arrays, kept resident while it is loaded), and register its height field with physics under TerrainInfo.geometry. Loading the same path again answers the same terrain.
     *
     * @param world The world.
     * @param path The terrain file, such as `terrain/island.ogt`.
     * @returns Refused for a path that is not a cooked terrain, or a layer texture that does not load.
     */
    function load(world: World, path: string): Result<TerrainInfo>;
    /**
     * The surface under a world point: its height, normal and slope. Callable inside systems (it reads the terrain, not the world).
     *
     * @param world The world.
     * @param desc The terrain, where it stands and the point.
     * @returns Refused for an unknown terrain.
     */
    function sample(world: World, desc?: TerrainSampleDesc): Result<TerrainSample>;
    /**
     * A terrain's box in world space when it stands at `origin`, turned by `quaternion` and scaled by `scale` (its entity's world transform): the box around its turned corners.
     *
     * @param world The world.
     * @param data The terrain.
     * @param origin Where it stands.
     * @param quaternion How it is turned.
     * @param scale How it is scaled.
     * @returns Refused for an unknown terrain.
     */
    function bounds(world: World, data: TerrainData, origin?: Vector3, quaternion?: Quaternion, scale?: Vector3): Result<TerrainBounds>;
    /**
     * Change a terrain under a round brush: raise, lower, flatten or smooth its heights (the renderer, physics and og.terrain.sample see the change from the next frame and tick), or paint a layer into its control map (making one when it has none).
     *
     * @param world The world.
     * @param data The terrain.
     * @param brush What to do and where.
     * @returns Refused for an unknown terrain, a layer beyond its layers, or a brush that misses it.
     */
    function edit(world: World, data: TerrainData, brush?: TerrainBrush): Result<TerrainEdit>;
    /**
     * A terrain's layer as it is now (for a tuning tool: read it, change it, setLayer it).
     *
     * @param world The world.
     * @param data The terrain.
     * @param index The layer, from 0.
     * @returns Refused for an unknown terrain or layer.
     */
    function layer(world: World, data: TerrainData, index: number): Result<TerrainLayerDesc>;
    /**
     * Replace a terrain's layer look and rules (its textures stay). Drawn from the next frame.
     *
     * @param world The world.
     * @param data The terrain.
     * @param index The layer, from 0.
     * @param layer The layer.
     * @returns Refused for an unknown terrain or layer.
     */
    function setLayer(world: World, data: TerrainData, index: number, layer?: TerrainLayerDesc): Result<void>;
  }
  /** A group changed state: sent by og.stream at the start of the tick after the change. */
  const StreamEvent: EventType<StreamEvent>;
  /**
   * Asset streaming groups (phase 3): a scene's `stream "<name>" { asset ...; distance x y z r [unload d] | volume "<entity>"; priority n }` blocks name glTF models (without skins or morph targets), textures, texture arrays, flipbooks (an `array true` sheet brings its array), sounds, environment maps, palettes, looks and colour LUTs (`.cube`) that load and unload together, and what loads them: the camera within `r` of the anchor (kept until it is past `unload`, 1.25 r by default), the camera inside the named entity's box (its position ± half its scale, turned with it), or neither (og.stream.request only). `og cook` packs each group's files into `cooked/stream/<name>.ogb`; `og build --publish` ships the packs beside the boot bundle, which the shell and the service worker fetch first, and fetches a pack only when its group loads.
   * At every tick boundary the engine decides which groups are wanted, admits them by priority (then name) within the memory budget (256 MiB by default; GPU textures and meshes and decoded audio, as the cook measured them), evicting lower-priority resident groups to make room for a higher one, and reads and puts in place at most one group's assets per boundary. A group that cannot fit waits, queued; one larger than the whole budget fails. Until its group is resident, a model draws its proxy (its coarsest LOD, untextured), a texture is a 1x1 grey placeholder and a sound plays as silence of its own length; then the real ones take over the same handles. Physics always collides with the proxy's meshes and a sound's length never changes, so nothing a tick reads depends on what is resident: a run's world hash is the same streamed or not.
   * Texture arrays, flipbooks, environment maps, palettes, looks and colour LUTs have no stand-in: they load once their group is resident, and when it unloads the renderer lets them go and their handles wait (the sky lights the scene, Stylise.palette quantises to none, PostEffects.lut grades nothing, og.render.sprite of the array and applying the look are refused, an og.Sprite, a flipbook's sprite or a particle emitter of the array draws nothing) until the group is resident again, when they come back under the same handles. One the frame still uses when its group unloads (the map Environment lights it with, the palette Stylise quantises to, an array a sprite, a screen sprite or a particle emitter draws from, the look last applied, the LUT PostEffects grades with) is never freed under the frame: it is kept, with a warning naming it in the engine's log, until the frame stops using it. What is kept still takes its memory: its bytes count in the budget's `used` (and `kept`, and its group's StreamStatus.kept) until it is let go, so a group that would not fit beside it waits.
   * Every change of a group's state sends StreamEvent at the start of the next tick; og.stream.status and og.stream.budget answer at once, and the probe's `dump_stream` lists the budget, the memory used and its peak, every group (trigger, state, why, since which tick, what it keeps for the frame after unloading with each asset's bytes), the recent changes, the terrain streamer's tiles and the fetch queue.
   * What streams is read through one queue: first the chunks of the sounds cooked in chunks that a playhead needs soonest (the one it is playing, then by the seconds until it gets there at its pitch, a loop's start by the time to its wrap), then the loading groups' packs by priority (then name). At most four reads are in flight at once (on the web, fetches; natively a read completes at once, so only the order shows); the rest wait in that order. `dump_stream`'s `fetch` lists what is in flight and what waits, each with its path and what it is for (a sound's chunk and its seconds, a group's pack and its priority).
   */
  namespace stream {
    /**
     * One streaming group's state.
     *
     * @param world The world.
     * @param group The group's name, as its scene declares it.
     * @returns Refused for a group no scene declares.
     */
    function status(world: World, group: string): Result<StreamStatus>;
    /**
     * Ask for a group (any trigger) until og.stream.release: it loads from the next tick boundary, within the budget by its priority.
     *
     * @param world The world.
     * @param group The group's name.
     * @returns Refused for a group no scene declares.
     */
    function request(world: World, group: string): Result<void>;
    /**
     * Stop asking for a group; it unloads at the next tick boundary unless its trigger still wants it.
     *
     * @param world The world.
     * @param group The group's name.
     * @returns Refused for a group no scene declares.
     */
    function release(world: World, group: string): Result<void>;
    /**
     * The memory budget, what the groups take now, the peak and every group.
     *
     * @param world The world.
     * @returns The budget and the groups; refused only for a world that does not exist.
     */
    function budget(world: World): Result<StreamBudget>;
    /**
     * Set the memory budget from the next tick boundary; a lower one evicts the lowest-priority groups until the rest fit.
     *
     * @param world The world.
     * @param bytes The bytes resident groups may take.
     * @returns Refused for a negative or non-finite size.
     */
    function setBudget(world: World, bytes: number): Result<void>;
  }
  /**
   * Plays a loaded video: VideoInfo.texture shows its current frame (frame n is the source at n / fps seconds), and when the entity has an og.AudioSource the og.video system sets its sound, playing, pitch and loop each tick so the sound plays in step: a seek or a restart starts the sound again at the new time on the next tick (AudioSource.startOffset). Several players of one video share its texture, which shows the first player's frame in query order; cook a second asset to show two moments of one clip.
   */
  const VideoPlayer: ComponentType<{ readonly video: FieldSpec<"Video">; readonly playing: FieldSpec<"bool">; readonly time: FieldSpec<"f32">; readonly speed: FieldSpec<"f32">; readonly loop: FieldSpec<"bool">; readonly frame: FieldSpec<"u32">; readonly finished: FieldSpec<"bool">; readonly texture: FieldSpec<"Texture"> }, VideoPlayer>;
  /**
   * Debug draw: lines, boxes, spheres, arrows, axes and labels in the world, each for a number of ticks, drawn over the finished frame (after every post effect, before the UI) so they read in every capture and never hide behind geometry; and the debug views (physics, navigation, bounds) the command channel's `debug:` actions switch.
   * Development only: not in publish builds, where every member does nothing and answers its default, so a game may leave its debug draws in. Debug draw is rendering: it never feeds back into the simulation, and its draws are not world state (a snapshot or a replay does not carry them).
   */
  namespace debug {
    /**
     * A line from one point to another.
     *
     * @param world The world drawn in.
     * @param from Where it starts.
     * @param to Where it ends.
     * @param desc Colour, width, duration.
     */
    function line(world: World, from: Vector3, to: Vector3, desc?: DebugDrawDesc): void;
    /**
     * A box's twelve edges, as three.js Box3Helper (turned by desc.rotation).
     *
     * @param world The world drawn in.
     * @param center Its centre.
     * @param size Its extent along each axis.
     * @param desc Colour, width, duration, rotation.
     */
    function box(world: World, center: Vector3, size: Vector3, desc?: DebugDrawDesc): void;
    /**
     * A sphere as its three great circles.
     *
     * @param world The world drawn in.
     * @param center Its centre.
     * @param radius Its radius.
     * @param desc Colour, width, duration, rotation.
     */
    function sphere(world: World, center: Vector3, radius: number, desc?: DebugDrawDesc): void;
    /**
     * An arrow from one point to another, its head a fifth of its length (three.js ArrowHelper): a velocity, a force, a direction to a target.
     *
     * @param world The world drawn in.
     * @param from Its tail.
     * @param to Its tip.
     * @param desc Colour, width, duration.
     */
    function arrow(world: World, from: Vector3, to: Vector3, desc?: DebugDrawDesc): void;
    /**
     * A frame's three axes, as three.js AxesHelper: +x red, +y green, +z blue (desc.color is not read).
     *
     * @param world The world drawn in.
     * @param position The frame's origin.
     * @param size Each axis's length.
     * @param desc Width, duration and the frame's rotation.
     */
    function axes(world: World, position: Vector3, size: number, desc?: DebugDrawDesc): void;
    /**
     * A label anchored at a world point, drawn in screen space at a constant size in the UI's body font (a world with no UI font draws none), centred above the point: an entity's state, a counter, a name.
     *
     * @param world The world drawn in.
     * @param position The point the label sits above.
     * @param text The label.
     * @param desc Colour and duration (width is the text's size in CSS pixels, 14 where it is 2 or less).
     */
    function text(world: World, position: Vector3, text: string, desc?: DebugDrawDesc): void;
    /**
     * Remove every debug draw that is still showing.
     *
     * @param world The world drawn in.
     */
    function clear(world: World): void;
    /**
     * Show a debug view besides og.debug's draws (the command channel's `debug:<view>`); none shows the draws alone.
     *
     * @param world The world drawn in.
     * @param view The view.
     */
    function setView(world: World, view: DebugView): void;
    /**
     * The debug view shown.
     *
     * @param world The world asked.
     */
    function view(world: World): DebugView;
    /**
     * Show or hide every debug draw and view (`debug:on`, `debug:off`); on by default outside publish builds.
     *
     * @param world The world drawn in.
     * @param enabled Whether they show.
     */
    function setEnabled(world: World, enabled: boolean): void;
    /**
     * Whether debug draws and views show.
     *
     * @param world The world asked.
     */
    function enabled(world: World): boolean;
  }
  /**
   * Moves the entity along a path at `speed`, by the engine's og.path system after the game's systems and before physics: og.Transform.position is the path's point at `distance` plus `offset` (in the path's frame with align, else in world axes). Advanced by scaled time (slow motion, hit stop). The distance is the component's own field, so a snapshot, a restore and the world hash carry it, and writing it moves the entity there.
   */
  const PathFollower: ComponentType<{ readonly path: FieldSpec<"Path">; readonly speed: FieldSpec<"f32">; readonly distance: FieldSpec<"f32">; readonly loop: FieldSpec<"bool">; readonly align: FieldSpec<"bool">; readonly offset: FieldSpec<"Vector3">; readonly playing: FieldSpec<"bool">; readonly finished: FieldSpec<"bool"> }, PathFollower>;
  /**
   * Paths: a curve through or controlled by points, measured by arc length so a parameter t from 0 to 1 moves along it at an even speed however its points are spaced. catmullRom passes through every point (three.js CatmullRomCurve3, tension 0.5), bezier chains cubic segments `p0 c0 c1 p1 c2 c3 p2 ...` (three.js CubicBezierCurve3 in a CurvePath), linear joins the points with straight segments. A closed path joins its last point to its first and wraps t.
   * Make one with og.path.create, or declare it in a scene (`path "<name>" { kind catmullRom; closed false; point x y z ... }`) and find it by name with og.path.find; a timeline's camera rides one with `rail "<name>"`. og.PathFollower moves an entity along one, og.CameraRail.path moves the camera along one, og.render's tube geometry can take its og.path.sample points. Paths are assets: they are not world state, and a hot reload registers them again.
   */
  namespace path {
    /**
     * Make a path from points.
     *
     * @param world The world.
     * @param desc The points, kind, closed and tension.
     * @returns Refused for too few points for the kind (bezier needs 3n + 1 open, 3n closed) or a point that is not finite.
     */
    function create(world: World, desc?: CurveDesc): Result<Path>;
    /**
     * A path a loaded scene declares (`path "<name>" { ... }`), by name.
     *
     * @param world The world.
     * @param name The path's name in its scene.
     * @returns Refused for a name no spawned scene declares; the reason lists the names.
     */
    function find(world: World, name: string): Result<Path>;
    /**
     * The point at arc-length parameter t: 0 the start, 1 the end (a closed path wraps, an open one clamps). Equal steps of t are equal lengths along the curve.
     *
     * @param world The world.
     * @param path The path.
     * @param t The parameter, 0 to 1.
     * @returns Refused for a dead path.
     */
    function sample(world: World, path: Path, t: number): Result<PathSample>;
    /**
     * Its length along the curve.
     *
     * @param world The world.
     * @param path The path.
     * @returns Metres; refused for a dead path.
     */
    function length(world: World, path: Path): Result<number>;
    /**
     * The point of the path nearest a point, with its parameter: snapping to a track, projecting a pointer onto a rail.
     *
     * @param world The world.
     * @param path The path.
     * @param point The point.
     * @returns Refused for a dead path.
     */
    function nearest(world: World, path: Path, point: Vector3): Result<PathNearest>;
  }
  /**
   * Plays a loaded timeline. Set `playing` (or call og.timeline.play) to run it from `time`; write `time` (or call og.timeline.seek) to jump; og.timeline.stop goes back to 0 and lets go of what it owns.
   */
  const Timeline: ComponentType<{ readonly asset: FieldSpec<"TimelineAsset">; readonly playing: FieldSpec<"bool">; readonly time: FieldSpec<"f32">; readonly speed: FieldSpec<"f32">; readonly loop: FieldSpec<"bool"> }, Timeline>;
  /**
   * A timeline started playing from stopped or ended (not on a resume from pause, nor on a loop).
   */
  const TimelineStarted: EventType<TimelineStarted>;
  /** Playback passed a marker key (a seek over one does not send it). */
  const TimelineMarker: EventType<TimelineMarker>;
  /** A timeline without loop reached its duration and stopped playing. */
  const TimelineEnded: EventType<TimelineEnded>;
  /**
   * Timelines: cutscenes and sequences as data, Unity's Timeline and Godot's AnimationPlayer as an `.ogt` the agent reads and diffs. A `type timeline` file has a `duration` in seconds, `path "<name>" { ... }` blocks (as og.path's) and `track <kind> ["<entity>"] { key <seconds> { ... } }` blocks; `og cook` cooks it (importing caption files) and og.load.timeline loads it. Tracks: `camera` (position and target keys, or `rail "<path>"` with `at` 0..1 along it, or `follow "<entity>"` with offsets; fov; eased), `transform "<entity>"` (position, rotation in degrees or quaternion, scale; eased), `animation "<entity>"` (clip, speed, blendIn, loop; `model` names the glTF whose clips they are, else the entity's og.Model), `audio` (sound, volume, pitch, bus), `music` (sound, volume, fadeIn, fadeOut, end, loop, bus), `caption` (text from `key` to `end` with a UI type step `style` and an `anchor`; `captions "narration.srt" [offset]` imports a SubRip file or word timestamps `.json`), `event` (`do "<command-channel action>"`: teleport, set, a game's registered command; `retroactive true` runs it when a seek passes it), `transition` (kind cut, fade, wipe, dissolve, iris or slide; mode through, in or out; duration, color, from, over og.ui's screen cover) and `marker` (`name`, sending og.TimelineMarker when played past).
   * Between two keys that set a value the later key's `ease` (og.tween's Ease names, linear when it gives none) mixes them; before the first and after the last the value is held. Every value is evaluated at the timeline's time, never accumulated over ticks, and that time is kept exactly (in steps of a thousandth of a tick), so seeking to a time and playing to it give the same world: a seek is how a video render starts at any frame.
   * An og.Timeline component plays one on its entity, run by the engine's og.timeline system after the game's systems (and tweens) and before og.anim and physics, so what it sets is what animation, physics and the frame see that tick. While one with a camera track plays (or is paused) it owns og.Camera, and hands back the camera it found when it stops or ends. Its event keys' actions run at the start of the next tick, as og.ai's commands do. The command channel plays them too: `timeline:play:<name>` (`timelines/<name>.ogt`, or an asset id), `timeline:seek:<seconds>[:<name>]`, `timeline:pause[:<name>]`, `timeline:resume[:<name>]`, `timeline:stop[:<name>]`.
   */
  namespace timeline {
    /**
     * What a loaded timeline holds.
     *
     * @param world The world.
     * @param asset The timeline.
     * @returns Refused for a timeline not loaded in this world.
     */
    function info(world: World, asset: TimelineAsset): Result<TimelineInfo>;
    /**
     * Play the entity's og.Timeline from its time (0 after a stop or an end); systems may call it, it takes effect at this tick's og.timeline.
     *
     * @param world The world.
     * @param entity The entity with the og.Timeline.
     * @returns Refused for an entity without og.Timeline, or one whose asset is not loaded.
     */
    function play(world: World, entity: Entity): Result<void>;
    /**
     * Hold the entity's og.Timeline where it is (it keeps the camera).
     *
     * @param world The world.
     * @param entity The entity with the og.Timeline.
     * @returns Refused for an entity without og.Timeline.
     */
    function pause(world: World, entity: Entity): Result<void>;
    /**
     * Jump to a time: this tick evaluates every track there exactly as playing to it would, without sending the markers or running the events passed (except `retroactive` ones).
     *
     * @param world The world.
     * @param entity The entity with the og.Timeline.
     * @param time Seconds, 0 to the duration.
     * @returns Refused for an entity without og.Timeline or a time outside 0 to its duration.
     */
    function seek(world: World, entity: Entity, time: number): Result<void>;
    /**
     * Stop at 0: the camera is handed back, its sounds and music stop, its caption and cover go.
     *
     * @param world The world.
     * @param entity The entity with the og.Timeline.
     * @returns Refused for an entity without og.Timeline.
     */
    function stop(world: World, entity: Entity): Result<void>;
  }
  /**
   * Lockstep's settings and this peer's place. The game sets enabled, players, inputDelay and hashEvery (the authority's are the room's once it starts: changes while running wait for the next start); the engine sets localPlayer and running. It is this peer's own: never hashed, never compared.
   */
  const Lockstep: ResourceType<{ readonly enabled: FieldSpec<"bool">; readonly players: FieldSpec<"u32">; readonly localPlayer: FieldSpec<"u32">; readonly inputDelay: FieldSpec<"u32">; readonly hashEvery: FieldSpec<"u32">; readonly running: FieldSpec<"bool"> }, Lockstep>;
  /**
   * One player's input for this tick, the same on every peer: the engine writes every field but `player` at the start of each tick from the seat's input (zero for a seat nobody holds, and before its holder's first input). Its packed bytes are what crosses the room, read through this layout.
   */
  const PlayerInput: ComponentType<{ readonly player: FieldSpec<"u32">; readonly buttons: FieldSpec<"u32">; readonly x: FieldSpec<"f32">; readonly y: FieldSpec<"f32">; readonly aimX: FieldSpec<"f32">; readonly aimY: FieldSpec<"f32">; readonly tick: FieldSpec<"u32"> }, PlayerInput>;
  /**
   * This peer's world and another's differ: sent once, at the start of the tick after the hashes showed it. The probe's `desync` dump (`dump_desync`) names the component and field that differ, once the other peer's snapshot of that tick arrived.
   */
  const Desync: EventType<Desync>;
  /**
   * Deterministic lockstep over a room (docs/design.md §2.10): every peer runs the whole world, and what crosses the room is each player's input, one per tick, and the world's hash. Turn it on with og.Lockstep (`enabled`, `players`); join a room (og.platform.roomJoin, or `og run --room ws://127.0.0.1:P/<room>` with `og relay` natively). Until the room holds `players` peers the world runs freely on each (a lobby, the local input driving the local player); then the authority (the lowest peer number) starts: every other peer takes its world's snapshot, the seats go to the peers in number order, and from then on tick t runs only when every seated player's input for t is in. A peer that joins a running room takes a free seat (or watches) from a snapshot a running peer hands it.
   * Inputs are og.PlayerInput: one entity per player carries it with `player` set to its seat; the engine writes every other field at the start of each tick from that seat's input for the tick. A system reads the inputs from og.PlayerInput, never from og.Input, and gives this peer's own with og.net.submit (from a system that reads og.Input: what the local player holds now, sent for the tick inputDelay ticks on). Without a room, or with lockstep off, og.net.submit drives the local player's og.PlayerInput the next tick, so the same game plays alone.
   * Every tick each peer hashes its world without its own session (og.Input, og.Pointers, og.Gamepad, og.Audio, og.Camera, og.Lockstep, the input, room, platform, asset and stream events, the UI's focus and hover) and every hashEvery ticks sends the hashes since its last batch; the first tick whose hashes differ is reported once as og.Desync, and the probe's `desync` dump (`dump_desync`) names it with the component-level diff of the two peers' snapshots of that tick: the answer to what is not deterministic (Math.random, Date.now, a wall clock, iteration over a Set of objects, a module variable, input read from og.Input).
   */
  namespace net {
    /**
     * Give this peer's player's input: in lockstep, for the tick inputDelay ticks from the one running (sent to every peer when the tick ends); otherwise, for the next tick. It holds until the next call, so a game calls it once a tick from a system that reads og.Input (or og.Gamepad, or the UI's touch controls). Callable from a system.
     *
     * @param world The world.
     * @param input The buttons and axes.
     * @returns Refused for an axis that is not finite.
     */
    function submit(world: World, input?: InputDesc): Result<void>;
    /**
     * Lockstep on this peer: running, the seats and who it waits for.
     *
     * @param world The world.
     */
    function status(world: World): LockstepStatus;
    /**
     * The world's shared hash now, hex: the hash lockstep compares, which leaves this peer's session out (og.Input, og.Lockstep, input and room events, the UI's focus; module docs). Two peers at the same tick agree on it exactly when their simulations agree; the world dump's `hash` covers everything and differs between peers.
     *
     * @param world The world.
     */
    function hash(world: World): string;
  }
}
