//! Pure, typed functions registered by project.ogt before any Yarn is cooked.
use og_core::ogt::Document;
use og_story::yarn::types::{Function, Signature, Type};
use std::collections::BTreeMap;
use std::path::Path;

pub(super) fn read(root: &Path) -> Result<BTreeMap<String, Function>, String> {
    let source = match std::fs::read_to_string(root.join("project.ogt")) {
        Ok(source) => source,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(e) => return Err(e.to_string()),
    };
    let doc = Document::parse(&source).map_err(|e| e.to_string())?;
    let Some(entry) = doc.body.get("storyFunctions") else {
        return Ok(BTreeMap::new());
    };
    let block = entry.block.as_ref().ok_or("storyFunctions needs a block")?;
    let mut functions = BTreeMap::new();
    let ty = |name: &str| match name {
        "number" => Ok(Type::Number),
        "string" => Ok(Type::String),
        "bool" => Ok(Type::Bool),
        _ => Err(format!(
            "function type '{name}': use number, string or bool"
        )),
    };
    for entry in &block.entries {
        let fail = || {
            format!(
                "project.ogt:{}: write function <name> <return-type> {{ param <name> <type>; body \"expression\" }}",
                entry.line
            )
        };
        if entry.key != "function" || entry.values.len() != 2 {
            return Err(fail());
        }
        let name = entry.values[0].as_word().ok_or_else(fail)?;
        let returns = ty(entry.values[1].as_word().ok_or_else(fail)?)?;
        let body = entry.block.as_ref().ok_or_else(fail)?;
        let mut parameters = Vec::new();
        let mut types = Vec::new();
        let mut expression = None;
        for entry in &body.entries {
            if entry.block.is_some() {
                return Err(fail());
            }
            match entry.key.as_str() {
                "param" if entry.values.len() == 2 => {
                    parameters.push(entry.values[0].as_word().ok_or_else(fail)?.to_owned());
                    types.push(ty(entry.values[1].as_word().ok_or_else(fail)?)?);
                }
                "body" if entry.values.len() == 1 && expression.is_none() => {
                    expression = Some(og_story::yarn::parse_expression(
                        entry.values[0].as_str().ok_or_else(fail)?,
                    )?);
                }
                _ => return Err(fail()),
            }
        }
        let function = Function {
            signature: Signature {
                parameters: types,
                variadic: None,
                returns,
            },
            parameters,
            body: expression.ok_or_else(fail)?,
        };
        if functions.insert(name.to_owned(), function).is_some() {
            return Err(format!("duplicate story function '{name}'"));
        }
    }
    og_story::yarn::functions::validate(&mut functions)?;
    Ok(functions)
}
