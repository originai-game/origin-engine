//! Reading the model out of the open bundle and making it resident.

use super::*;

/// The model's weights in the cooked bundle.
pub(crate) const MODEL_FILE: &str = "motion/model.ogb";

/// The weights' file inside that pack.
pub(crate) const MODEL_ENTRY: &str = "motion/model.safetensors";

/// The model file the open bundle declares (its manifest's `motion` entry,
/// which og cook writes for a project whose code uses og.motion): `None`
/// while no bundle is open, else the path or why there is none.
pub(crate) fn declared(eng: &Engine) -> Option<Result<String, String>> {
    let a = eng.assets.borrow();
    let i = a.default_bundle?;
    let crate::assets::Payload::Bundle { bundle, dir } = &a.entries[i].payload else {
        return None;
    };
    Some(match &bundle.manifest().motion {
        Some(m) => Ok(format!("{dir}/{}", m.file)),
        None => Err(format!(
            "{dir} declares no og.motion model: og cook writes {MODEL_FILE} for a project whose src/ uses og.motion and finds the engine's weights"
        )),
    })
}

/// Whether the open bundle declares og.motion's model (the game uses it).
pub(crate) fn declares(eng: &Engine) -> bool {
    matches!(declared(eng), Some(Ok(_)))
}

/// The model from its pack's bytes: on the CPU, or with `gpu` on that device.
pub(crate) fn build(
    bytes: &[u8],
    gpu: Option<(wgpu::Device, wgpu::Queue)>,
) -> Result<Parts, String> {
    let files = og_bundle::runtime::unpack(bytes).map_err(|e| e.to_string())?;
    let (_, weights) = files
        .iter()
        .find(|(n, _)| n == MODEL_ENTRY)
        .ok_or_else(|| format!("it holds no {MODEL_ENTRY}"))?;
    parts(weights, gpu)
}

pub(crate) fn ready(parts: Parts) -> Model {
    Model::Ready(Box::new(resident(parts, now_ms)))
}

/// A load on a native host's background thread: the pack's size, then the
/// model or why it failed.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type Loading = std::sync::mpsc::Receiver<(u64, Result<Parts, String>)>;

/// Starts or continues reading the model and putting it on the renderer's
/// device; answers the state. Natively the file is read, unpacked and
/// uploaded on a thread of its own (wgpu's device takes buffers and
/// pipelines from any thread), so no frame waits for it; on the web the
/// host's fetch lands at a later frame, which then unpacks and uploads it.
/// `wait`: natively, wait for the thread's answer (an agent needs the model
/// this tick); the web cannot wait.
pub(crate) fn load(
    p: &mut Presenter,
    eng: &Engine,
    backend: Option<&dyn RenderBackend>,
    wait: bool,
) -> Model {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(rx) = &p.loading {
        let answer = if wait {
            rx.recv()
                .map_err(|_| std::sync::mpsc::TryRecvError::Disconnected)
        } else {
            rx.try_recv()
        };
        return match answer {
            Ok((bytes, r)) => {
                p.loading = None;
                p.model_bytes = p.model_bytes.max(bytes);
                r.map_or_else(Model::Failed, ready)
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                Model::Loading("loading: the weights are being read and uploaded".into())
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                p.loading = None;
                Model::Failed("the model's loading thread ended without an answer".into())
            }
        };
    }
    let gpu = if !p.cpu {
        let Some(compute) = backend.and_then(|b| b.compute()) else {
            return Model::Waiting(
                "no renderer is attached: the model runs on the renderer's GPU (og.motion.setBackend cpu runs it without one)"
                    .into(),
            );
        };
        Some(compute)
    } else {
        None
    };
    let path = match declared(eng) {
        None => return Model::Waiting("the cooked bundle is not open yet".into()),
        Some(Err(why)) => return Model::Failed(why),
        Some(Ok(path)) => path,
    };
    #[cfg(not(target_arch = "wasm32"))]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        let spawned = std::thread::Builder::new()
            .name("og-motion load".into())
            .spawn(move || {
                let answer = match std::fs::read(&path) {
                    Ok(bytes) => (
                        bytes.len() as u64,
                        build(&bytes, gpu).map_err(|e| format!("{path}: {e}")),
                    ),
                    Err(e) => (0, Err(format!("{path}: {e}"))),
                };
                let _ = tx.send(answer);
            });
        if let Err(e) = spawned {
            return Model::Failed(format!("the model's loading thread: {e}"));
        }
        p.loading = Some(rx);
        Model::Loading("loading: the weights are being read and uploaded".into())
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = wait;
        let bytes = match hooks().map(|h| h.read_asset(&path)) {
            Some(crate::assets::Fetch::Ready(b)) => b,
            Some(crate::assets::Fetch::Pending) => {
                return Model::Loading(format!("loading {path}"));
            }
            Some(crate::assets::Fetch::Failed(e)) => return Model::Failed(format!("{path}: {e}")),
            None => return Model::Failed("no host is installed to read assets".into()),
        };
        p.model_bytes = p.model_bytes.max(bytes.len() as u64);
        build(&bytes, gpu).map_or_else(|e| Model::Failed(format!("{path}: {e}")), ready)
    }
}
