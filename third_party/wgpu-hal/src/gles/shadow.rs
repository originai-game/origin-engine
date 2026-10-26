//! What one `Queue::submit` has set on the GL context (an origin-engine patch:
//! third_party/wgpu-hal/ORIGIN.md), so a call that would set what is already
//! set is not made. The command encoder assumes the default state at every
//! command buffer and restates a pass's whole framebuffer, texture, sampler
//! and vertex state at every pass and draw; on WebGL every such call crosses
//! from wasm into JavaScript and the browser's validation, most of them
//! changing nothing (3,797 calls a frame of origin-engine's reference scene,
//! 761 of which changed state: tests/evidence/r4/perf-webgl2-2/NOTES.md).
//!
//! The shadow starts unknown at each submit (anything may have touched the
//! context between submits) and is forgotten after any command it does not
//! model ([`Shadow::forget`]), so a skipped call is one whose state the
//! shadow saw set within this submit. Every state is left as the unpatched
//! backend leaves it after each command, with one deferral: the draw
//! framebuffer's attachments a pass detaches and attaches are applied at the
//! pass's next other command ([`Shadow::flush`]), so an attachment detached
//! and attached again is not touched.

use glow::HasContext;
use naga::FastHashMap;

/// The capabilities the backend enables and disables, by bit.
const CAPS: [u32; 8] = [
    glow::DEPTH_TEST,
    glow::STENCIL_TEST,
    glow::SCISSOR_TEST,
    glow::BLEND,
    glow::CULL_FACE,
    glow::POLYGON_OFFSET_FILL,
    glow::SAMPLE_ALPHA_TO_COVERAGE,
    glow::DEPTH_CLAMP,
];

/// The draw framebuffer's attachment points: the colours, depth, stencil.
const POINTS: usize = crate::MAX_COLOR_ATTACHMENTS + 2;
const DEPTH: usize = crate::MAX_COLOR_ATTACHMENTS;
const STENCIL: usize = crate::MAX_COLOR_ATTACHMENTS + 1;

/// What an attachment point holds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Attached {
    None,
    Renderbuffer(glow::Renderbuffer),
    Texture2d {
        target: u32,
        texture: glow::Texture,
        level: i32,
    },
    Layer {
        texture: glow::Texture,
        level: i32,
        layer: i32,
    },
}

/// A vertex attribute's array as `vertexAttribPointer` sets it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Pointer {
    pub buffer: glow::Buffer,
    pub integer: bool,
    pub normalized: bool,
    pub size: i32,
    pub ty: u32,
    pub stride: i32,
    pub offset: i32,
}

const UNITS: usize = super::MAX_TEXTURE_SLOTS;
const ATTRIBUTES: usize = super::MAX_VERTEX_ATTRIBUTES;

/// The state known set in this submit; `None` is unknown.
pub(super) struct Shadow {
    /// The queue's draw framebuffer, whose attachments are kept.
    draw_fbo: glow::Framebuffer,
    caps_known: u8,
    caps_on: u8,
    /// The capabilities set since the last [`Shadow::flush`], and to what.
    caps_wanted: u8,
    caps_wanted_on: u8,
    program: Option<Option<glow::Program>>,
    draw_binding: Option<Option<glow::Framebuffer>>,
    read_binding: Option<Option<glow::Framebuffer>>,
    /// `draw_fbo`'s attachments as GL has them, and as a pass wants them
    /// at its next command.
    attached: [Option<Attached>; POINTS],
    wanted: [Option<Attached>; POINTS],
    active_unit: Option<u32>,
    /// The texture bound per (unit, target).
    textures: FastHashMap<(u32, u32), glow::Texture>,
    /// Per texture, its base and max level: texture state, which only
    /// `BindTexture` sets, so it is kept when the rest is forgotten.
    levels: FastHashMap<glow::Texture, (i32, i32)>,
    samplers: [Option<Option<glow::Sampler>>; UNITS],
    array_buffer: Option<Option<glow::Buffer>>,
    element_buffer: Option<Option<glow::Buffer>>,
    enabled: [Option<bool>; ATTRIBUTES],
    /// Attribute arrays enabled or disabled since the last flush.
    enabled_wanted: [Option<bool>; ATTRIBUTES],
    pointers: [Option<Pointer>; ATTRIBUTES],
    divisors: [Option<u32>; ATTRIBUTES],
    color_mask: Option<[bool; 4]>,
    depth_mask: Option<bool>,
    /// Front and back.
    stencil_mask: [Option<u32>; 2],
}

impl Shadow {
    pub(super) fn new(draw_fbo: glow::Framebuffer) -> Self {
        Self {
            draw_fbo,
            caps_known: 0,
            caps_on: 0,
            caps_wanted: 0,
            caps_wanted_on: 0,
            program: None,
            draw_binding: None,
            read_binding: None,
            attached: [None; POINTS],
            wanted: [None; POINTS],
            active_unit: None,
            textures: FastHashMap::default(),
            levels: FastHashMap::default(),
            samplers: [None; UNITS],
            array_buffer: None,
            element_buffer: None,
            enabled: [None; ATTRIBUTES],
            enabled_wanted: [None; ATTRIBUTES],
            pointers: [None; ATTRIBUTES],
            divisors: [None; ATTRIBUTES],
            color_mask: None,
            depth_mask: None,
            stencil_mask: [None; 2],
        }
    }

    /// Whether `command` only sets state, observing none of what the shadow
    /// defers (the draw framebuffer's attachments, the capabilities, the
    /// attribute arrays' enables): those wait for the next command that
    /// draws, clears, copies or reads.
    pub(super) fn sets_only(command: &super::Command) -> bool {
        use super::Command as C;
        matches!(
            command,
            C::ResetFramebuffer { .. }
                | C::BindAttachment { .. }
                | C::SetViewport { .. }
                | C::SetScissor(..)
                | C::SetStencilFunc { .. }
                | C::SetStencilOps { .. }
                | C::SetDepth(..)
                | C::SetDepthBias(..)
                | C::ConfigureDepthStencil(..)
                | C::SetAlphaToCoverage(..)
                | C::SetVertexAttribute { .. }
                | C::UnsetVertexAttribute(..)
                | C::SetProgram(..)
                | C::SetPrimitive(..)
                | C::SetBlendConstant(..)
                | C::SetColorTarget { .. }
                | C::BindBuffer { .. }
                | C::BindSampler(..)
                | C::BindTexture { .. }
                | C::SetIndexBuffer(..)
        )
    }

    /// Whether every state `command` sets that the shadow keeps goes
    /// through it; after any other command the shadow forgets.
    pub(super) fn follows(command: &super::Command) -> bool {
        use super::Command as C;
        !matches!(
            command,
            C::ClearBuffer { .. }
                | C::CopyBufferToBuffer { .. }
                | C::CopyExternalImageToTexture { .. }
                | C::CopyTextureToTexture { .. }
                | C::CopyBufferToTexture { .. }
                | C::CopyTextureToBuffer { .. }
                | C::CopyQueryResults { .. }
                | C::BufferBarrier(..)
                | C::TextureBarrier(..)
        )
    }

    /// After a command that sets state the shadow does not follow: all of
    /// it is unknown again but the texture levels. Pending attachments
    /// were applied before the command ([`Shadow::flush`]).
    pub(super) fn forget(&mut self) {
        let levels = core::mem::take(&mut self.levels);
        *self = Self::new(self.draw_fbo);
        self.levels = levels;
    }

    /// `enable`/`disable` of `cap`, applied at the next [`Shadow::flush`]
    /// when it is one the shadow keeps.
    pub(super) fn cap(&mut self, gl: &glow::Context, cap: u32, on: bool) {
        let Some(bit) = CAPS.iter().position(|c| *c == cap).map(|i| 1u8 << i) else {
            if on {
                unsafe { gl.enable(cap) };
            } else {
                unsafe { gl.disable(cap) };
            }
            return;
        };
        self.caps_wanted |= bit;
        if on {
            self.caps_wanted_on |= bit;
        } else {
            self.caps_wanted_on &= !bit;
        }
    }

    /// A capability set some other way (per draw buffer): unknown.
    pub(super) fn forget_cap(&mut self, cap: u32) {
        if let Some(i) = CAPS.iter().position(|c| *c == cap) {
            self.caps_known &= !(1 << i);
            self.caps_wanted &= !(1 << i);
        }
    }

    pub(super) fn use_program(&mut self, gl: &glow::Context, program: Option<glow::Program>) {
        if self.program != Some(program) {
            unsafe { gl.use_program(program) };
            self.program = Some(program);
        }
    }

    pub(super) fn bind_framebuffer(
        &mut self,
        gl: &glow::Context,
        target: u32,
        framebuffer: Option<glow::Framebuffer>,
    ) {
        let (draw, read) = match target {
            glow::DRAW_FRAMEBUFFER => (true, false),
            glow::READ_FRAMEBUFFER => (false, true),
            _ => (true, true),
        };
        if draw {
            self.flush_attachments(gl);
        }
        let same = (!draw || self.draw_binding == Some(framebuffer))
            && (!read || self.read_binding == Some(framebuffer));
        if !same {
            unsafe { gl.bind_framebuffer(target, framebuffer) };
        }
        if draw {
            self.draw_binding = Some(framebuffer);
        }
        if read {
            self.read_binding = Some(framebuffer);
        }
    }

    /// The points `attachment` names.
    fn points(attachment: u32) -> Option<core::ops::Range<usize>> {
        match attachment {
            glow::DEPTH_ATTACHMENT => Some(DEPTH..DEPTH + 1),
            glow::STENCIL_ATTACHMENT => Some(STENCIL..STENCIL + 1),
            glow::DEPTH_STENCIL_ATTACHMENT => Some(DEPTH..STENCIL + 1),
            a if (glow::COLOR_ATTACHMENT0..glow::COLOR_ATTACHMENT0 + DEPTH as u32)
                .contains(&a) =>
            {
                let i = (a - glow::COLOR_ATTACHMENT0) as usize;
                Some(i..i + 1)
            }
            _ => None,
        }
    }

    /// Whether `attach` can wait: the draw framebuffer bound is the
    /// queue's, whose attachments the shadow keeps.
    fn keeps(&self, target: u32) -> bool {
        matches!(target, glow::DRAW_FRAMEBUFFER | glow::FRAMEBUFFER)
            && self.draw_binding == Some(Some(self.draw_fbo))
    }

    /// Whether the framebuffer `target` binds may be the queue's draw
    /// framebuffer (its binding unknown, or it).
    fn may_be_draw_fbo(&self, target: u32) -> bool {
        let binding = match target {
            glow::READ_FRAMEBUFFER => self.read_binding,
            _ => self.draw_binding,
        };
        binding.is_none_or(|b| b == Some(self.draw_fbo))
    }

    /// `framebufferTexture2D`/`Layer`/`Renderbuffer` of `attachment` on the
    /// draw framebuffer `target` binds, applied at [`Shadow::flush`].
    pub(super) fn attach(
        &mut self,
        gl: &glow::Context,
        target: u32,
        attachment: u32,
        what: Attached,
    ) {
        match Self::points(attachment) {
            Some(points) if self.keeps(target) => {
                for p in points {
                    self.wanted[p] = Some(what);
                }
            }
            _ => {
                self.flush_attachments(gl);
                unsafe { Self::apply(gl, target, attachment, what) };
                if self.may_be_draw_fbo(target) {
                    self.attached = [None; POINTS];
                }
            }
        }
    }

    /// An attachment made in a form the shadow does not keep (multiview,
    /// multisampled render to texture), on the draw framebuffer bound.
    pub(super) fn attached_otherwise(&mut self, gl: &glow::Context) {
        self.flush_attachments(gl);
        self.attached = [None; POINTS];
    }

    unsafe fn apply(gl: &glow::Context, target: u32, attachment: u32, what: Attached) {
        match what {
            Attached::None => unsafe {
                gl.framebuffer_texture_2d(target, attachment, glow::TEXTURE_2D, None, 0)
            },
            Attached::Renderbuffer(rb) => unsafe {
                gl.framebuffer_renderbuffer(target, attachment, glow::RENDERBUFFER, Some(rb))
            },
            Attached::Texture2d {
                target: t,
                texture,
                level,
            } => unsafe { gl.framebuffer_texture_2d(target, attachment, t, Some(texture), level) },
            Attached::Layer {
                texture,
                level,
                layer,
            } => unsafe {
                gl.framebuffer_texture_layer(target, attachment, Some(texture), level, layer)
            },
        }
    }

    /// Applies the attachments a pass wants (before any command that is
    /// not a pass's detaching and attaching), one call per point that
    /// changes, depth and stencil in one where both change to the same.
    /// Applies everything a command that observes state needs: the
    /// attachments, capabilities and attribute arrays set since the last
    /// flush, each only where it changes.
    pub(super) fn flush(&mut self, gl: &glow::Context) {
        self.flush_attachments(gl);
        let changed = self.caps_wanted & !(self.caps_known & !(self.caps_on ^ self.caps_wanted_on));
        for (i, cap) in CAPS.iter().enumerate() {
            let bit = 1u8 << i;
            if changed & bit != 0 {
                if self.caps_wanted_on & bit != 0 {
                    unsafe { gl.enable(*cap) };
                } else {
                    unsafe { gl.disable(*cap) };
                }
            }
        }
        self.caps_on = (self.caps_on & !self.caps_wanted) | (self.caps_wanted_on & self.caps_wanted);
        self.caps_known |= self.caps_wanted;
        self.caps_wanted = 0;
        self.flush_attributes(gl);
    }

    fn flush_attributes(&mut self, gl: &glow::Context) {
        for (location, w) in self.enabled_wanted.iter_mut().enumerate() {
            let Some(on) = w.take() else { continue };
            if self.enabled[location] != Some(on) {
                if on {
                    unsafe { gl.enable_vertex_attrib_array(location as u32) };
                } else {
                    unsafe { gl.disable_vertex_attrib_array(location as u32) };
                }
                self.enabled[location] = Some(on);
            }
        }
    }

    fn flush_attachments(&mut self, gl: &glow::Context) {
        if self.wanted.iter().all(Option::is_none) {
            return;
        }
        let changes = |s: &Self, p: usize| s.wanted[p].filter(|w| s.attached[p] != Some(*w));
        let (depth, stencil) = (changes(self, DEPTH), changes(self, STENCIL));
        if let (Some(d), Some(s)) = (depth, stencil) {
            if d == s {
                unsafe {
                    Self::apply(gl, glow::DRAW_FRAMEBUFFER, glow::DEPTH_STENCIL_ATTACHMENT, d)
                };
                self.attached[DEPTH] = Some(d);
                self.attached[STENCIL] = Some(d);
            }
        }
        for p in 0..POINTS {
            if let Some(w) = changes(self, p) {
                let attachment = match p {
                    DEPTH => glow::DEPTH_ATTACHMENT,
                    STENCIL => glow::STENCIL_ATTACHMENT,
                    i => glow::COLOR_ATTACHMENT0 + i as u32,
                };
                unsafe { Self::apply(gl, glow::DRAW_FRAMEBUFFER, attachment, w) };
                self.attached[p] = Some(w);
            }
        }
        self.wanted = [None; POINTS];
    }

    fn active(&mut self, gl: &glow::Context, unit: u32) {
        if self.active_unit != Some(unit) {
            unsafe { gl.active_texture(glow::TEXTURE0 + unit) };
            self.active_unit = Some(unit);
        }
    }

    /// `BindTexture`: `texture` on `unit` with its base and max level. The
    /// unit is made active only when something is set on it: no command
    /// reads the active unit but one that binds a texture of its own (a
    /// copy, after which the shadow forgets).
    pub(super) fn bind_texture(
        &mut self,
        gl: &glow::Context,
        unit: u32,
        target: u32,
        texture: glow::Texture,
        levels: (i32, i32),
    ) {
        if self.textures.get(&(unit, target)) != Some(&texture) {
            self.active(gl, unit);
            unsafe { gl.bind_texture(target, Some(texture)) };
            self.textures.insert((unit, target), texture);
        }
        if self.levels.get(&texture) != Some(&levels) {
            self.active(gl, unit);
            unsafe { gl.tex_parameter_i32(target, glow::TEXTURE_BASE_LEVEL, levels.0) };
            unsafe { gl.tex_parameter_i32(target, glow::TEXTURE_MAX_LEVEL, levels.1) };
            self.levels.insert(texture, levels);
        }
    }

    pub(super) fn bind_sampler(
        &mut self,
        gl: &glow::Context,
        unit: u32,
        sampler: Option<glow::Sampler>,
    ) {
        match self.samplers.get_mut(unit as usize) {
            Some(s) if *s == Some(sampler) => {}
            Some(s) => {
                unsafe { gl.bind_sampler(unit, sampler) };
                *s = Some(sampler);
            }
            None => unsafe { gl.bind_sampler(unit, sampler) },
        }
    }

    /// `bindBuffer`: the array and element bindings are kept, any other
    /// target is bound.
    pub(super) fn bind_buffer(
        &mut self,
        gl: &glow::Context,
        target: u32,
        buffer: Option<glow::Buffer>,
    ) {
        let kept = match target {
            glow::ARRAY_BUFFER => &mut self.array_buffer,
            glow::ELEMENT_ARRAY_BUFFER => &mut self.element_buffer,
            _ => {
                unsafe { gl.bind_buffer(target, buffer) };
                return;
            }
        };
        if *kept != Some(buffer) {
            unsafe { gl.bind_buffer(target, buffer) };
            *kept = Some(buffer);
        }
    }

    /// `enable`/`disableVertexAttribArray`, applied at the next
    /// [`Shadow::flush`].
    pub(super) fn enable_attribute(&mut self, gl: &glow::Context, location: u32, on: bool) {
        match self.enabled_wanted.get_mut(location as usize) {
            Some(w) => *w = Some(on),
            None if on => unsafe { gl.enable_vertex_attrib_array(location) },
            None => unsafe { gl.disable_vertex_attrib_array(location) },
        }
    }

    /// `vertexAttribPointer` (or its integer form) of `p.buffer`, bound
    /// to the array target only when the pointer changes: no command
    /// reads that binding but one that binds its own.
    pub(super) fn pointer(&mut self, gl: &glow::Context, location: u32, p: Pointer) {
        let kept = self.pointers.get(location as usize).copied().flatten();
        if kept == Some(p) {
            return;
        }
        self.bind_buffer(gl, glow::ARRAY_BUFFER, Some(p.buffer));
        if p.integer {
            unsafe { gl.vertex_attrib_pointer_i32(location, p.size, p.ty, p.stride, p.offset) };
        } else {
            unsafe {
                gl.vertex_attrib_pointer_f32(location, p.size, p.ty, p.normalized, p.stride, p.offset)
            };
        }
        if let Some(slot) = self.pointers.get_mut(location as usize) {
            *slot = Some(p);
        }
    }

    pub(super) fn divisor(&mut self, gl: &glow::Context, location: u32, divisor: u32) {
        match self.divisors.get_mut(location as usize) {
            Some(d) if *d == Some(divisor) => {}
            Some(d) => {
                unsafe { gl.vertex_attrib_divisor(location, divisor) };
                *d = Some(divisor);
            }
            None => unsafe { gl.vertex_attrib_divisor(location, divisor) },
        }
    }

    /// The vertex attributes set some other way (the separate attribute
    /// format of GLES 3.1): unknown.
    pub(super) fn forget_attributes(&mut self, gl: &glow::Context) {
        self.flush_attributes(gl);
        self.enabled = [None; ATTRIBUTES];
        self.pointers = [None; ATTRIBUTES];
        self.divisors = [None; ATTRIBUTES];
    }

    pub(super) fn color_mask(&mut self, gl: &glow::Context, mask: [bool; 4]) {
        if self.color_mask != Some(mask) {
            unsafe { gl.color_mask(mask[0], mask[1], mask[2], mask[3]) };
            self.color_mask = Some(mask);
        }
    }

    /// The colour mask set per draw buffer: unknown.
    pub(super) fn forget_color_mask(&mut self) {
        self.color_mask = None;
    }

    pub(super) fn depth_mask(&mut self, gl: &glow::Context, on: bool) {
        if self.depth_mask != Some(on) {
            unsafe { gl.depth_mask(on) };
            self.depth_mask = Some(on);
        }
    }

    /// `stencilMaskSeparate` of `face` (`stencilMask` is both faces).
    pub(super) fn stencil_mask(&mut self, gl: &glow::Context, face: u32, mask: u32) {
        let faces: &[usize] = match face {
            glow::FRONT => &[0],
            glow::BACK => &[1],
            _ => &[0, 1],
        };
        if faces.iter().all(|f| self.stencil_mask[*f] == Some(mask)) {
            return;
        }
        if faces.len() == 2 {
            unsafe { gl.stencil_mask(mask) };
        } else {
            unsafe { gl.stencil_mask_separate(face, mask) };
        }
        for f in faces {
            self.stencil_mask[*f] = Some(mask);
        }
    }
}
