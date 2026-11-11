use super::*;

pub(super) fn plan_sfx<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    // A preset's stamp moves with the synthesiser.
    let st = stamp(&[b"sfx", sfx::SYNTH_VERSION.as_bytes(), &hash]);
    outs.push((s.rel.clone(), format::Sound::NAME, st, Work::Sound(s, None)));
}

pub(super) fn plan_sound<'a>(
    _cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let PlanState { early_errors, .. } = state;
    let settings = match audio::SoundSettings::parse(settings_block(s, "sound")) {
        Ok(x) => x,
        Err(e) => {
            early_errors.push((s.rel.clone(), "sound", e));
            return;
        }
    };
    // An unreadable file fails in the cook, with its reason.
    let chunking = audio::shape(&s.rel, &s.bytes)
        .ok()
        .and_then(|(rate, ch, frames)| settings.chunking(rate, ch, frames));
    let shape = chunking.map_or("whole".to_string(), |(p, n)| format!("{n} chunks of {p}"));
    let st = stamp(&[
        b"sound",
        &hash,
        settings.describe().as_bytes(),
        shape.as_bytes(),
    ]);
    outs.push((
        s.rel.clone(),
        format::Sound::NAME,
        st,
        Work::Sound(s, chunking),
    ));
    if let Some((per, count)) = chunking {
        for n in 1..count {
            let st = stamp(&[b"sound chunk", &hash, shape.as_bytes(), &n.to_le_bytes()]);
            outs.push((
                format!("{}#chunk{n}", s.rel),
                format::SoundChunk::NAME,
                st,
                Work::SoundChunk(s, per, n),
            ));
        }
    }
}

pub(super) fn plan_video<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { tex_desc, .. } = *cx;
    let PlanState { early_errors, .. } = state;
    let text = String::from_utf8_lossy(&s.bytes);
    let root = texture_array::assets_dir(&s.path, &s.rel);
    let planned = video::parse(&s.rel, &text, &|p| root.join(p).is_file()).and_then(|d| {
        let src = root.join(&d.source);
        let bytes = std::fs::read(&src).map_err(|e| format!("{}: {e}", src.display()))?;
        let encoder = if d.stream {
            video::stream_encoder()?
        } else {
            ""
        };
        let plan = video::plan(&d, video::probe(&src)?, encoder)?;
        Ok((src, hash_bytes(&bytes), plan))
    });
    match planned {
        Ok((src, src_hash, plan)) => {
            let described = plan.describe();
            let base = [
                b"video 2".as_slice(),
                tex_desc.as_bytes(),
                &hash,
                &src_hash.to_le_bytes(),
                described.as_bytes(),
            ];
            let plan = Box::new(plan);
            outs.push((
                s.rel.clone(),
                format::Video::NAME,
                stamp(&base),
                Work::VideoMeta(plan.clone()),
            ));
            if plan.decl.stream {
                outs.push((
                    video::stream_id(&s.rel),
                    format::VideoStream::NAME,
                    stamp(&[&base[..], &[b"stream".as_slice()]].concat()),
                    Work::VideoStream(src.clone(), src_hash, plan.clone()),
                ));
                if let Some(size) = og_video::stream::fallback_size(plan.size) {
                    outs.push((
                        video::stream_fallback_id(&s.rel),
                        format::VideoStream::NAME,
                        stamp(&[&base[..], &[b"stream720".as_slice()]].concat()),
                        Work::VideoStream(
                            src.clone(),
                            src_hash,
                            Box::new(video::fallback_plan(&plan, size)),
                        ),
                    ));
                }
                outs.push((
                    video::poster_id(&s.rel),
                    format::Texture::NAME,
                    stamp(&[&base[..], &[b"poster".as_slice()]].concat()),
                    Work::VideoPoster(src.clone(), src_hash, plan.clone()),
                ));
            } else {
                let frames = video::frames_id(&s.rel);
                outs.push((
                    frames.clone(),
                    format::TextureArray::NAME,
                    stamp(&[&base[..], &[b"array".as_slice()]].concat()),
                    Work::VideoArray(s, plan.clone()),
                ));
                for n in 0..plan.frames as usize {
                    let st = stamp(&[&base[..], &[&(n as u32).to_le_bytes()[..]]].concat());
                    outs.push((
                        texture_array::layer_id(&frames, n),
                        format::Texture::NAME,
                        st,
                        Work::VideoFrame(src.clone(), src_hash, plan.clone(), n),
                    ));
                }
            }
            if plan.audio {
                outs.push((
                    video::audio_id(&s.rel),
                    format::Sound::NAME,
                    stamp(&[&base[..], &[b"audio".as_slice()]].concat()),
                    Work::VideoAudio(src.clone(), plan.clone()),
                ));
            }
        }
        Err(e) => early_errors.push((s.rel.clone(), "video", e)),
    }
}
