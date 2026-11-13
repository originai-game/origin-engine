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
