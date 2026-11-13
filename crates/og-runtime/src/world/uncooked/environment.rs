//! Environment maps: packed to RGB9E5 with their irradiance, a few rows a frame.

use super::*;

/// An environment's image decoded: RGB9E5 texels, rows from the top, and
/// its irradiance.
pub(super) struct Prepared {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) texels: Vec<u32>,
    pub(super) irradiance: [[f32; 3]; 9],
}

/// Rows of an environment's image packed as they come, its irradiance
/// gathered beside them; a non-finite pixel refuses the image.
pub(super) struct Packer {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) intensity: f32,
    pub(super) texels: Vec<u32>,
    pub(super) irradiance: og_render::EquirectIrradiance,
}

impl Packer {
    pub(super) fn new(width: u32, height: u32, intensity: f32) -> Packer {
        Packer {
            width,
            height,
            intensity,
            texels: Vec::with_capacity(width as usize * height as usize),
            irradiance: og_render::EquirectIrradiance::new(width, height),
        }
    }

    pub(super) fn row(&mut self, y: u32, rgb: &mut [f32]) -> Result<(), String> {
        if rgb.len() != self.width as usize * 3 {
            return Err(format!("row {y} does not hold {} pixels", self.width));
        }
        if let Some(i) = rgb.iter().position(|v| !v.is_finite()) {
            return Err(format!(
                "pixel {} of row {y} holds a value that is not a finite number",
                i / 3
            ));
        }
        for v in rgb.iter_mut() {
            *v = v.max(0.0) * self.intensity;
        }
        self.irradiance.add_row(y, rgb);
        self.texels
            .extend(rgb.as_chunks::<3>().0.iter().map(|c| og_render::rgb9e5(*c)));
        Ok(())
    }

    pub(super) fn finish(self) -> Prepared {
        Prepared {
            width: self.width,
            height: self.height,
            irradiance: self.irradiance.finish(),
            texels: self.texels,
        }
    }
}

/// Where an environment's decode is (inside the job's box: one a load).
#[allow(clippy::large_enum_variant)]
pub(super) enum EnvDecode {
    /// A `.hdr` read in rows at the boundaries (the web).
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Rows {
        rows: og_bundle::runtime::HdrRows,
        packer: Packer,
        row: Vec<f32>,
    },
    /// On a worker thread of this process.
    #[cfg(not(target_arch = "wasm32"))]
    Thread(std::sync::mpsc::Receiver<Result<Prepared, String>>),
    Done(Result<Prepared, String>),
}

pub(super) struct Env {
    pub(super) size: u32,
    pub(super) decode: EnvDecode,
}

/// An environment's whole image decoded and packed (a worker thread's
/// work natively).
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn prepare_native(
    bytes: Vec<u8>,
    exr: bool,
    intensity: f32,
) -> Result<Prepared, String> {
    if exr {
        let (w, h) = og_bundle::runtime::exr_size(&bytes)?;
        let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes));
        reader.set_format(image::ImageFormat::OpenExr);
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(w);
        limits.max_image_height = Some(h);
        reader.limits(limits);
        let mut img = reader.decode().map_err(|e| e.to_string())?.into_rgb32f();
        if (img.width(), img.height()) != (w, h) {
            return Err(format!(
                "decoded to {}x{}, but its header says {w}x{h}",
                img.width(),
                img.height()
            ));
        }
        let mut packer = Packer::new(w, h, intensity);
        for (y, row) in img.chunks_mut(w as usize * 3).enumerate() {
            packer.row(y as u32, row)?;
        }
        return Ok(packer.finish());
    }
    let mut rows = og_bundle::runtime::HdrRows::new(bytes)?;
    let mut packer = Packer::new(rows.width(), rows.height(), intensity);
    let mut row = Vec::new();
    while rows.next_row(&mut row)? {
        packer.row(rows.row() - 1, &mut row)?;
    }
    Ok(packer.finish())
}

/// `og.load.environmentData`: checks the image against its budget and
/// the desc, starts its decode and answers its (pending) asset index.
pub(crate) fn environment_data(
    eng: &Engine,
    name: &str,
    bytes: Vec<u8>,
    desc: &Value,
) -> Result<usize, String> {
    let budget = Budget::from_desc(desc)?;
    let size = match &desc["size"] {
        Value::Null => 128,
        v => v
            .as_u64()
            .ok_or_else(|| format!("EnvironmentDataDesc.size is a whole number, got {v}"))?
            as u32,
    };
    if !size.is_power_of_two() || !ENV_SIZES.contains(&size) {
        return Err(format!(
            "{name}: EnvironmentDataDesc.size is a power of two from {} to {}, not {size}",
            ENV_SIZES.start(),
            ENV_SIZES.end()
        ));
    }
    if size > budget.texture_size {
        return Err(format!(
            "{name}: EnvironmentDataDesc.size {size} is over LoadBudget.textureSize ({}); ask for a smaller cube or raise the budget",
            budget.texture_size
        ));
    }
    let intensity = match &desc["intensity"] {
        Value::Null => 1.0,
        v => v
            .as_f64()
            .filter(|x| x.is_finite() && *x >= 0.0)
            .ok_or_else(|| {
                format!("EnvironmentDataDesc.intensity is a number of at least 0, got {v}")
            })? as f32,
    };
    check_size(name, &bytes, &budget)?;
    let exr = og_bundle::runtime::is_exr(&bytes);
    let (width, height) = if exr {
        if !EXR_DECODE {
            return Err(format!(
                "{name}: an OpenEXR image, which this host does not decode (RenderCapabilities.exrDecode is false on the web): convert it to a Radiance .hdr, or cook it as an environment"
            ));
        }
        og_bundle::runtime::exr_size(&bytes).map_err(|e| format!("{name}: {e}"))?
    } else if og_bundle::runtime::is_hdr(&bytes) {
        og_bundle::runtime::hdr_size(&bytes).map_err(|e| format!("{name}: {e}"))?
    } else {
        return Err(format!(
            "{name}: not a Radiance .hdr or OpenEXR .exr (an environment is an equirectangular HDR image)"
        ));
    };
    if width.max(height) > budget.texture_size {
        return Err(format!(
            "{name}: {width}x{height} is over LoadBudget.textureSize ({}); scale the image down or raise the budget (at most {DEVICE_TEXTURE_SIZE}, the largest every device takes)",
            budget.texture_size
        ));
    }
    if width < 2 * height.saturating_sub(1) || width > 2 * height + 1 {
        return Err(format!(
            "{name} is {width} × {height}: an equirectangular image is twice as wide as it is tall"
        ));
    }
    #[cfg(not(target_arch = "wasm32"))]
    let decode = {
        let (send, recv) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = send.send(prepare_native(bytes, exr, intensity));
        });
        EnvDecode::Thread(recv)
    };
    #[cfg(target_arch = "wasm32")]
    let decode = {
        let rows = og_bundle::runtime::HdrRows::new(bytes).map_err(|e| format!("{name}: {e}"))?;
        EnvDecode::Rows {
            packer: Packer::new(rows.width(), rows.height(), intensity),
            rows,
            row: Vec::new(),
        }
    };
    Ok(queue(eng, name, &budget, |_| {
        Work::Environment(Box::new(Env { size, decode }))
    }))
}

/// Advances an environment's decode; whether it is finished. Each row is
/// a unit of the frame's work.
pub(super) fn env_step(env: &mut Env, clock: &mut Clock, frame_ms: f64, wait: bool) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    if let EnvDecode::Thread(recv) = &env.decode {
        let got = if wait {
            recv.recv().map_err(|_| ())
        } else {
            recv.try_recv().map_err(|_| ())
        };
        match got {
            Ok(r) => env.decode = EnvDecode::Done(r),
            Err(()) if wait => {
                env.decode = EnvDecode::Done(Err("the decoding thread stopped".into()))
            }
            Err(()) => return false,
        }
    }
    if let EnvDecode::Rows { rows, packer, row } = &mut env.decode {
        loop {
            // The last row read finishes the image in the same frame.
            if (rows.row() < rows.height()) && !wait && clock.over(frame_ms) {
                return false;
            }
            match rows.next_row(row) {
                Ok(true) => {
                    clock.worked = true;
                    if let Err(e) = packer.row(rows.row() - 1, row) {
                        env.decode = EnvDecode::Done(Err(e));
                        break;
                    }
                }
                Ok(false) => {
                    let EnvDecode::Rows { packer, .. } =
                        std::mem::replace(&mut env.decode, EnvDecode::Done(Err(String::new())))
                    else {
                        unreachable!()
                    };
                    env.decode = EnvDecode::Done(Ok(packer.finish()));
                    break;
                }
                Err(e) => {
                    env.decode = EnvDecode::Done(Err(e));
                    break;
                }
            }
        }
    }
    true
}

/// A decoded environment into the renderer: its map added, the asset's
/// payload naming it.
pub(super) fn put_environment(
    eng: &Engine,
    name: &str,
    entry: usize,
    env: Env,
) -> Result<(), String> {
    let EnvDecode::Done(result) = env.decode else {
        unreachable!("only finished jobs are put in place");
    };
    let p = result?;
    let mut rs = eng.render.borrow_mut();
    let handle = rs.scene.add_environment_equirect(
        og_render::EnvironmentEquirect {
            width: p.width,
            height: p.height,
            texels: p.texels,
            size: env.size,
        },
        p.irradiance,
    )?;
    // Not in by_path: og.load.environment never answers it, and a release
    // never brings it back.
    rs.env
        .maps
        .push((format!("{name} (uncooked)"), Some(handle)));
    let map = rs.env.maps.len() - 1;
    drop(rs);
    eng.assets.borrow_mut().entries[entry].payload = Payload::Environment {
        map,
        width: p.width,
        height: p.height,
        size: env.size,
    };
    Ok(())
}

/// `og.load.environmentInfo`.
pub(crate) fn environment_info(eng: &Engine, index: Option<usize>) -> Result<Value, String> {
    let a = eng.assets.borrow();
    let e = index
        .and_then(|i| a.entries.get(i))
        .ok_or("not an asset handle")?;
    match (&e.status, &e.payload) {
        (Status::Failed(r), _) => Err(r.clone()),
        (Status::Pending, Payload::Uncooked) => Err(format!(
            "{} is still loading: og.load.status answers ready once it is in place",
            e.path
        )),
        (
            Status::Ready,
            Payload::Environment {
                map,
                width,
                height,
                size,
            },
        ) => Ok(json!({
            "map": crate::world::assets::handle(*map) as f64,
            "width": width,
            "height": height,
            "size": size,
        })),
        _ => Err(format!(
            "{} is not an environment og.load.environmentData loaded",
            e.path
        )),
    }
}
