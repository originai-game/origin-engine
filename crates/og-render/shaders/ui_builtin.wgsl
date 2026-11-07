// The built-in UI materials (UiBuiltin, docs/ui.md §3.3), after
// ui_common.wgsl and ui_material.wgsl: one `material` that switches on the
// layer's kind. Parameters are the layer's four vectors a, b, c, d; time is
// the UI clock.

struct Params {
  a: vec4<f32>,
  b: vec4<f32>,
  c: vec4<f32>,
  d: vec4<f32>,
};

fn material_params() -> Params {
  var p: Params;
  p.a = layer_params.params[0];
  p.b = layer_params.params[1];
  p.c = layer_params.params[2];
  p.d = layer_params.params[3];
  return p;
}

fn hue_rotate(rgb: vec3<f32>, a: f32) -> vec3<f32> {
  // Rotation about the grey axis.
  let k = vec3<f32>(0.57735);
  let c = cos(a);
  return rgb * c + cross(k, rgb) * sin(a) + k * dot(k, rgb) * (1.0 - c);
}

fn material(m: MaterialInput, p: Params) -> vec4<f32> {
  let kind = u32(layer_params.info.x);
  let texel = layer_texel();
  switch kind {
    case 1u: {
      // Dissolve: two octaves of value noise against the threshold, an
      // edge of the edge colour where it is about to go.
      let scale = max(p.a.z, 1.0);
      let n = 0.65 * value_noise(m.pixel / scale) + 0.35 * value_noise(m.pixel / (0.37 * scale) + 17.0);
      let t = p.a.x * 1.02 - 0.01;
      let c = layer(m.uv);
      let keep = smoothstep(t - 0.01, t + 0.01, n);
      let edge = 1.0 - smoothstep(t, t + max(p.a.y, 1e-3), n);
      let rgb = mix(c.rgb, p.b.rgb, clamp(edge * p.b.a, 0.0, 1.0) * step(0.0001, p.a.x));
      return vec4<f32>(rgb, c.a * keep);
    }
    case 2u: {
      // Shine: a soft band across the diagonal at `position` (negative:
      // sweeping by the clock), added on the layer's own pixels.
      let c = layer(m.uv);
      var pos = p.a.x;
      if (pos < 0.0) {
        pos = fract(m.time * -pos) * 1.6 - 0.3;
      }
      let dir = vec2<f32>(cos(p.a.z), sin(p.a.z));
      let ext = abs(dir.x) + abs(dir.y);
      let along = dot(m.uv - 0.5, dir) / ext + 0.5;
      let band = smoothstep(max(p.a.y, 1e-3), 0.0, abs(along - pos));
      return vec4<f32>(c.rgb + p.b.rgb * p.b.a * band * p.a.w, c.a);
    }
    case 3u: {
      // Hologram: a chromatic split, scanlines and a flicker, tinted.
      let split = vec2<f32>(p.a.z * texel.x, 0.0);
      let cr = layer(m.uv + split);
      let cg = layer(m.uv);
      let cb = layer(m.uv - split);
      let a = max(cg.a, max(cr.a, cb.a));
      var rgb = vec3<f32>(cr.r, cg.g, cb.b);
      rgb = mix(rgb, rgb * p.b.rgb * 1.6, p.b.a);
      let spacing = max(p.a.x, 1.0);
      let scan = 0.5 + 0.5 * sin(m.pixel.y * TAU / spacing + m.time * 6.0);
      let flicker = 1.0 - p.a.y * (0.5 + 0.5 * sin(m.time * 43.0 + sin(m.time * 7.0) * 3.0));
      return vec4<f32>(rgb, a * (1.0 - p.a.w * scan) * flicker);
    }
    case 4u: {
      // Glitch: slices shifted sideways at random by the clock, their
      // colours split.
      let slice = floor(m.pixel.y / max(p.a.y, 1.0));
      let seed = floor(m.time * max(p.a.w, 0.1));
      let r = hash12(vec2<f32>(slice, seed));
      let hit = step(1.0 - p.a.x, r);
      let shift = (hash12(vec2<f32>(seed, slice + 7.0)) - 0.5) * 2.0 * p.a.z * hit;
      let uv = m.uv + vec2<f32>(shift * texel.x, 0.0);
      let split = vec2<f32>(p.a.z * 0.35 * hit * texel.x, 0.0);
      let cr = layer(uv + split);
      let cg = layer(uv);
      let cb = layer(uv - split);
      return vec4<f32>(cr.r, cg.g, cb.b, max(cg.a, max(cr.a, cb.a)));
    }
    case 5u: {
      // Ripple: rings from the centre moving outwards.
      let centre = p.b.xy * m.size;
      let d = m.pixel - centre;
      let dist = length(d);
      let wl = max(p.a.y, 1.0);
      let wave = sin((dist - m.time * p.a.z) * TAU / wl) * p.a.x;
      let dir = select(vec2<f32>(0.0), d / dist, dist > 1e-3);
      return layer(m.uv + dir * wave * texel);
    }
    case 6u: {
      // Pixelate: the layer sampled once per cell.
      let cell = max(p.a.x, 1.0);
      let uv = (floor(m.pixel / cell) + 0.5) * cell / max(m.size, vec2<f32>(1.0));
      return layer(uv);
    }
    case 7u: {
      // Colour adjust: hue, saturation, brightness, grayscale.
      let c = layer(m.uv);
      var rgb = hue_rotate(c.rgb, p.a.x);
      let luma = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
      rgb = mix(vec3<f32>(luma), rgb, p.a.y) * p.a.z;
      let grey = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
      rgb = mix(rgb, vec3<f32>(grey), clamp(p.a.w, 0.0, 1.0));
      return vec4<f32>(max(rgb, vec3<f32>(0.0)), c.a);
    }
    case 8u: {
      // Outline of the alpha: the most alpha within the width, under the layer.
      let c = layer(m.uv);
      let w = max(p.a.x, 0.0);
      var most = 0.0;
      for (var i = 0; i < 16; i++) {
        let a = f32(i) * TAU / 16.0;
        let o = vec2<f32>(cos(a), sin(a)) * texel;
        most = max(most, layer_premultiplied(m.uv + o * w).a);
        most = max(most, layer_premultiplied(m.uv + o * w * 0.5).a);
      }
      let oa = clamp(most - c.a, 0.0, 1.0) * p.b.a;
      let a = c.a + oa * (1.0 - c.a);
      let rgb = (c.rgb * c.a + p.b.rgb * oa * (1.0 - c.a)) / max(a, 1e-5);
      return vec4<f32>(rgb, a);
    }
    case 9u: {
      // Layer blur: a Gaussian of half the radius, 9x9 taps over the
      // premultiplied layer, each read from the level of the layer's mips
      // (ui/pass.rs) as coarse as the taps are apart, so every tap
      // averages the detail between it and its neighbours: read from
      // level 0, taps r/4 apart point-sampled fine detail into a mosaic.
      let r = max(p.a.x, 0.0);
      if (r < 0.5) {
        return layer(m.uv);
      }
      let sigma = 0.5 * r;
      let step = r / 4.0;
      let device = f32(textureDimensions(layer_tex, 0u).x) / max(layer_params.rect.z, 1.0);
      let lod = log2(max(step * device, 1.0));
      var sum = vec4<f32>(0.0);
      var wsum = 0.0;
      for (var y = -4; y <= 4; y++) {
        for (var x = -4; x <= 4; x++) {
          let o = vec2<f32>(f32(x), f32(y)) * step;
          let w = exp(-dot(o, o) / (2.0 * sigma * sigma));
          sum += textureSampleLevel(layer_tex, layer_sampler, m.uv + o * texel, lod) * w;
          wsum += w;
        }
      }
      let c = sum / wsum;
      return vec4<f32>(c.rgb / max(c.a, 1e-5), c.a);
    }
    default: {
      return layer(m.uv);
    }
  }
}
