//! The story's variables and its saliency strategy.

use super::*;

pub fn og_story_set_saliency(world: u32, strategy: String) -> Result<(), String> {
    use og_story::yarn::saliency::Saliency;
    (|| {
        let strategy = match strategy.as_str() {
            "first" => Saliency::First,
            "best" => Saliency::Best,
            "bestLeastRecentlyViewed" => Saliency::BestLeastRecentlyViewed,
            "randomBestLeastRecentlyViewed" => Saliency::RandomBestLeastRecentlyViewed,
            "conditionalFirstWeighted" => Saliency::ConditionalFirstWeighted,
            _ => return Err(format!("unknown Yarn saliency strategy '{strategy}'")),
        };
        story(world, |_, s| s.saliency = strategy)
    })()
}

pub fn og_story_get(world: u32, name: String) -> Option<Value> {
    story(world, |lib, s| s.get(lib, &name))
        .ok()
        .flatten()
        .map(|v| val_json(&v))
}

pub fn og_story_set(world: u32, name: String, value: String) -> Result<(), String> {
    (|| {
        let v = parse(Some(&value))?;
        let v = val_of(&v).ok_or("a story variable is a number, a boolean or a string")?;
        story(world, |lib, s| s.set(lib, &name, v))
    })()
}

pub fn og_story_reset(world: u32) {
    let _ = story(world, |_, s| s.reset());
}
