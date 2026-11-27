//! Pure, typed functions registered by project.ts before any Yarn is cooked.
use crate::cook::source_schemas::project::{StoryFunction, StoryType};
use og_story::yarn::types::{Function, Signature, Type};
use std::collections::BTreeMap;

pub(crate) fn read(
    input: &BTreeMap<String, StoryFunction>,
) -> Result<BTreeMap<String, Function>, String> {
    let mut functions = BTreeMap::new();
    let ty = |kind: &StoryType| match kind {
        StoryType::Number => Type::Number,
        StoryType::String => Type::String,
        StoryType::Bool => Type::Bool,
    };
    for (name, input) in input {
        let function = Function {
            signature: Signature {
                parameters: input.parameters.iter().map(|p| ty(&p.kind)).collect(),
                variadic: None,
                returns: ty(&input.returns),
            },
            parameters: input.parameters.iter().map(|p| p.name.clone()).collect(),
            body: og_story::yarn::parse_expression(&input.body)
                .map_err(|e| format!("project.ts: storyFunctions.{name}: {e}"))?,
        };
        functions.insert(name.clone(), function);
    }
    og_story::yarn::functions::validate(&mut functions)?;
    Ok(functions)
}
