//! The world's type registry: every component, resource and event type is a
//! reflected [`Schema`] with a [`SchemaId`]. Rust types register through
//! [`Reflect`]; hosts register a layout by schema (by name).

use og_core::reflect::{FieldType, Layout, Reflect, Schema};
use og_core::{Error, ErrorKind, Result};
use std::any::{Any, TypeId};
use std::collections::{BTreeMap, HashMap};

/// The id of a type registered with a world (component, resource or event).
/// Ids are dense and follow registration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SchemaId(pub(crate) u32);

impl SchemaId {
    pub fn index(self) -> u32 {
        self.0
    }
}

/// One field of one registered type, resolved once and used for column access.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FieldRef {
    pub component: SchemaId,
    pub field: u32,
    pub ty: FieldType,
}

pub(crate) type AnyValue = Box<dyn Any + Send>;

pub(crate) struct RustType {
    pub decode: fn(&Layout, &[u8]) -> Result<AnyValue>,
    pub encode: fn(&Layout, &(dyn Any + Send)) -> Result<Vec<u8>>,
}

fn decode_any<T: Reflect>(layout: &Layout, bytes: &[u8]) -> Result<AnyValue> {
    Ok(Box::new(layout.decode::<T>(bytes)?))
}

fn encode_any<T: Reflect>(layout: &Layout, value: &(dyn Any + Send)) -> Result<Vec<u8>> {
    let value = value
        .downcast_ref::<T>()
        .ok_or_else(|| Error::new(ErrorKind::Invalid, format!("value is not a {}", T::NAME)))?;
    layout.encode(value)
}

pub(crate) struct TypeInfo {
    pub schema: Schema,
    pub layout: Layout,
    pub layout_hash: u64,
    pub rust: Option<RustType>,
}

#[derive(Default)]
pub(crate) struct Registry {
    pub types: Vec<TypeInfo>,
    by_name: BTreeMap<String, SchemaId>,
    by_type: HashMap<TypeId, SchemaId>,
}

impl Registry {
    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn info(&self, id: SchemaId) -> &TypeInfo {
        &self.types[id.0 as usize]
    }

    pub fn name(&self, id: SchemaId) -> &str {
        &self.info(id).schema.name
    }

    pub fn check(&self, id: SchemaId) -> Result<()> {
        if (id.0 as usize) < self.types.len() {
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::NotFound,
                format!("no type with id {}", id.0),
            ))
        }
    }

    /// Registers a schema. Returns the id and whether it is new. A name that
    /// is already registered with the same layout returns the existing id.
    pub fn register_schema(&mut self, schema: Schema) -> Result<(SchemaId, bool)> {
        schema.validate()?;
        let hash = schema.layout_hash();
        if let Some(&id) = self.by_name.get(&schema.name) {
            if self.info(id).layout_hash != hash {
                return Err(Error::new(
                    ErrorKind::Schema,
                    format!(
                        "'{}' is already registered with a different layout",
                        schema.name
                    ),
                ));
            }
            return Ok((id, false));
        }
        let id = SchemaId(u32::try_from(self.types.len()).expect("fewer than 2^32 types"));
        self.by_name.insert(schema.name.clone(), id);
        self.types.push(TypeInfo {
            layout: Layout::of(&schema),
            layout_hash: hash,
            schema,
            rust: None,
        });
        Ok((id, true))
    }

    /// Registers a Rust type (idempotent). A host schema of the same name and
    /// layout is adopted by the Rust type.
    pub fn register<T: Reflect>(&mut self) -> Result<(SchemaId, bool)> {
        if let Some(&id) = self.by_type.get(&TypeId::of::<T>()) {
            return Ok((id, false));
        }
        let (id, new) = self.register_schema(T::schema())?;
        let info = &mut self.types[id.0 as usize];
        if info.rust.is_some() {
            return Err(Error::new(
                ErrorKind::Schema,
                format!("'{}' is already registered by another Rust type", T::NAME),
            ));
        }
        // A derive that disagrees with its own schema is caught here, once.
        let probe = T::read_fields(&mut |_, buf| buf.fill(0));
        info.layout.encode(&probe)?;
        info.rust = Some(RustType {
            decode: decode_any::<T>,
            encode: encode_any::<T>,
        });
        self.by_type.insert(TypeId::of::<T>(), id);
        Ok((id, new))
    }

    pub fn id_of<T: Reflect>(&self) -> Result<SchemaId> {
        self.by_type
            .get(&TypeId::of::<T>())
            .copied()
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::NotFound,
                    format!(
                        "{} is not registered; call World::register::<{}>() first",
                        T::NAME,
                        T::NAME
                    ),
                )
            })
    }

    pub fn id_by_name(&self, name: &str) -> Result<SchemaId> {
        self.by_name
            .get(name)
            .copied()
            .ok_or_else(|| Error::new(ErrorKind::NotFound, format!("no type named '{name}'")))
    }

    pub fn field(&self, id: SchemaId, field: &str) -> Result<FieldRef> {
        self.check(id)?;
        let schema = &self.info(id).schema;
        let index = schema.field_index(field).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                format!("'{}' has no field '{field}'", schema.name),
            )
        })?;
        Ok(FieldRef {
            component: id,
            field: index as u32,
            ty: schema.fields[index].ty,
        })
    }

    /// `Ok` if `field` names a field of this registry with the type it claims.
    pub fn check_field(&self, field: FieldRef) -> Result<()> {
        self.check(field.component)?;
        let schema = &self.info(field.component).schema;
        match schema.fields.get(field.field as usize) {
            Some(f) if f.ty == field.ty => Ok(()),
            _ => Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "field {} of '{}' is not a {}",
                    field.field,
                    schema.name,
                    field.ty.name()
                ),
            )),
        }
    }

    pub fn field_name(&self, field: FieldRef) -> String {
        let schema = &self.info(field.component).schema;
        match schema.fields.get(field.field as usize) {
            Some(f) => format!("{}.{}", schema.name, f.name),
            None => format!("{}.#{}", schema.name, field.field),
        }
    }

    pub fn encode<T: Reflect>(&self, id: SchemaId, value: &T) -> Result<Vec<u8>> {
        self.info(id).layout.encode(value)
    }

    pub fn decode<T: Reflect>(&self, id: SchemaId, bytes: &[u8]) -> Result<T> {
        self.info(id).layout.decode(bytes)
    }

    /// `Ok` if `bytes` has the packed size of `id`.
    pub fn check_size(&self, id: SchemaId, bytes: &[u8]) -> Result<()> {
        let size = self.info(id).layout.size();
        if bytes.len() == size {
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::Invalid,
                format!(
                    "'{}' is {size} bytes packed, got {}",
                    self.name(id),
                    bytes.len()
                ),
            ))
        }
    }
}
