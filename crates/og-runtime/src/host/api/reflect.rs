//! og.reflect: components, resources and events defined by the game.

use super::*;

pub(super) fn define_record(name: String, fields: &str, kind: RecordKind) -> Result<Value, String> {
    let declared = (|| {
        if name.starts_with("og.") {
            return Err(format!(
                "'{name}': the og. prefix belongs to the engine's own records"
            ));
        }
        let fields = parse(Some(fields))?;
        let layout = Layout::from_fields(&name, kind, &fields)?;
        layout.schema().validate().map_err(err)?;
        Ok(layout)
    })();
    DECL.with(|d| {
        let mut d = d.borrow_mut();
        match declared {
            Ok(layout) => {
                if let Some(prev) = d.records.iter().find(|l| l.schema == name) {
                    if prev.schema().layout_hash() != layout.schema().layout_hash()
                        || prev.kind != layout.kind
                    {
                        let reason = format!("'{name}' is declared twice with different layouts");
                        d.errors.push(reason.clone());
                        return Err(reason);
                    }
                } else {
                    d.records.push(layout);
                }
                Ok(json!({ "kind": kind.as_str(), "name": name }))
            }
            Err(reason) => {
                d.errors.push(reason.clone());
                Err(reason)
            }
        }
    })
}

pub fn og_reflect_define_component(name: String, fields: String) -> Result<Value, String> {
    define_record(name, &fields, RecordKind::Component)
}

pub fn og_reflect_define_resource(name: String, fields: String) -> Result<Value, String> {
    define_record(name, &fields, RecordKind::Resource)
}

pub fn og_reflect_define_event(name: String, fields: String) -> Result<Value, String> {
    define_record(name, &fields, RecordKind::Event)
}
