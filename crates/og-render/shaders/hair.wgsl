// Hair (physical.rs `hair`): strands lying along the surface, across the
// anisotropy's stretch, from root to tip the way v runs. A strand is a
// cylinder of keratin under tilted cuticle scales, and light leaves it by
// three paths (Marschner et al., 2003): R, off its surface, a white
// highlight shifted towards the root; TT, through it, the hair's colour,
// seen against the light; TRT, in, off its far wall and out, a coloured
// highlight shifted towards the tip. Each is a Gaussian along the strand
// times a share around it, in Karis's fits (Physically Based Hair Shading
// in Unreal, SIGGRAPH 2016, after d'Eon et al., 2011).

// The cuticle scales' tilt (rad): R leans twice it towards the root, TT
// by it and TRT four times it towards the tip.
const HAIR_TILT: f32 = 0.035;
// The strands a ray crosses in a metre of hair: a head grows 150 to the
// square centimetre, each 80 µm across.
const HAIR_PER_METRE: f32 = 120.0;

// The way of the light behind hair `thickness` (m) deep to the surface: x
// how far it travels through the hair (m), y how much of it gets there.
// An open surface (a card, a lock: drawn from both sides) is a layer the
// light crosses. A closed one is a body with the hair on it: the light
// comes along a chord of it (`curvature`, 1 / m), and only where the
// chord stays in the hair and does not dip under it, which is its rim.
fn hair_path(thickness: f32, curvature: f32, closed: bool, n: vec3<f32>, l: vec3<f32>) -> vec2<f32> {
  let c = abs(dot(n, l));
  if (!closed) { return vec2<f32>(thickness / max(c, 0.05), 1.0); }
  let radius = 1.0 / max(curvature, 1e-6);
  let dip = radius * (1.0 - sqrt(max(1.0 - c * c, 0.0)));
  return vec2<f32>(2.0 * c * radius, 1.0 - smoothstep(0.5 * thickness, thickness, dip));
}

struct Hair {
  front: vec3<f32>,    // R and TRT: the two highlights
  through: vec3<f32>,  // TT: what comes through the strand from behind
  scatter: vec3<f32>,  // the light the strands pass between them
};

fn hair_gauss(width: f32, x: f32) -> f32 {
  return exp(-0.5 * x * x / (width * width)) / (2.5066283 * width);
}

// Keratin's Fresnel (index 1.55).
fn hair_fresnel(cos_theta: f32) -> f32 {
  let m = 1.0 - cos_theta;
  let m2 = m * m;
  return 0.0465 + 0.9535 * m2 * m2 * m;
}

// One light on hair of `color`: `root` the strand's direction towards its
// root, `v` and `l` towards the eye and the light, `roughness` how wide
// the highlights are along the strand, `path` how far the light behind
// travels through the hair (m, `hair_path`).
fn hair_lobes(color: vec3<f32>, roughness: f32, root: vec3<f32>, v: vec3<f32>, l: vec3<f32>, path: f32) -> Hair {
  let sin_l = clamp(dot(root, l), -1.0, 1.0);
  let sin_v = clamp(dot(root, v), -1.0, 1.0);
  // Half the angle between the two along the strand, and the angle
  // between them around it.
  let cos_d = cos(0.5 * abs(asin(sin_v) - asin(sin_l)));
  let lp = l - sin_l * root;
  let vp = v - sin_v * root;
  let cos_phi = dot(lp, vp) * inverseSqrt(dot(lp, lp) * dot(vp, vp) + 1e-4);
  let cos_half_phi = sqrt(clamp(0.5 + 0.5 * cos_phi, 0.0, 1.0));
  let width = roughness * roughness;
  let along = sin_l + sin_v;
  var o: Hair;

  // R: the strand's surface, leaning towards the root.
  let tilt = -2.0 * HAIR_TILT;
  let lean = 2.0 * sin(tilt) * (cos(tilt) * cos_half_phi * sqrt(max(1.0 - sin_v * sin_v, 0.0)) + sin(tilt) * sin_v);
  let r = hair_gauss(width * 1.4142135 * max(cos_half_phi, 0.05), along - lean)
    * 0.25 * cos_half_phi
    * hair_fresnel(sqrt(clamp(0.5 + 0.5 * dot(v, l), 0.0, 1.0)));

  // TT: through the strand, forwards, tinted by the way through it: once
  // for every strand on the light's path.
  let inner = 1.0 / (1.19 / cos_d + 0.36 * cos_d);
  let h = cos_half_phi * (1.0 + inner * (0.6 - 0.8 * cos_phi));
  let f_tt = hair_fresnel(cos_d * sqrt(clamp(1.0 - h * h, 0.0, 1.0)));
  let tint_tt = pow(color, vec3<f32>(0.5 * sqrt(max(1.0 - h * h * inner * inner, 0.0)) / cos_d));
  o.through = hair_gauss(width * 0.5, along - HAIR_TILT)
    * exp(-3.65 * cos_phi - 3.98)
    * pow((1.0 - f_tt) * (1.0 - f_tt) * tint_tt, vec3<f32>(max(path * HAIR_PER_METRE, 1.0)));

  // TRT: in, off the far wall and out, leaning towards the tip.
  let f_trt = hair_fresnel(cos_d * 0.5);
  let trt = hair_gauss(width * 2.0, along - 4.0 * HAIR_TILT)
    * exp(17.0 * cos_phi - 16.78)
    * (1.0 - f_trt) * (1.0 - f_trt) * f_trt
    * pow(color, vec3<f32>(0.8 / cos_d));
  o.front = vec3<f32>(r) + trt;

  // The light the strands pass between them: a diffuse light wrapped
  // round the strand as the eye sees it, a third of it Kajiya and Kay's
  // (1989), scaled so a strand gives back what a matte surface of its
  // colour would under an even light.
  let facing = normalize(vp + root * 1e-4);
  let wrapped = clamp((dot(facing, l) + 1.0) * 0.25, 0.0, 1.0);
  o.scatter = color / PI * 0.752 * mix(wrapped, 1.0 - abs(sin_l), 0.33);
  return o;
}
