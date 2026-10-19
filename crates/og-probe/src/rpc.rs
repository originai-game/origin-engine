//! JSON-RPC 2.0 framing without batching (`engine/docs/probe-wire.md`):
//!
//! `{"jsonrpc":"2.0","id":<n|s>,"method":"<member>","params":{...}}` →
//! `{"jsonrpc":"2.0","id":..,"result":<value>}` or
//! `{"jsonrpc":"2.0","id":..,"error":{"code":<int>,"message":<string>}}`.
//!
//! A refusal is `result: false` or `null`, exactly as in-page; `error` is only
//! for a malformed call (`-32700`, `-32600`, `-32602`), an unknown method
//! (`-32601`, also every member a `publish` build drops), or a fault inside
//! the world (`-32603`: a system that failed, a panic).

use crate::input::InjectedInput;
use crate::probe::{
    DumpKind, MAX_CAPTURE_EDGE, MAX_SWEEP_VALUES, PLAYTEST_MAX_TICKS, Probe, SweepDesc,
    served_members,
};
use og_ecs::World;
use serde_json::{Map, Value, json};

pub const PARSE_ERROR: i64 = -32700;
pub const INVALID_REQUEST: i64 = -32600;
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;
pub const INTERNAL_ERROR: i64 = -32603;

/// A parsed call: the id to answer with, the member and its params.
#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    pub id: Value,
    pub method: String,
    pub params: Value,
}

/// Why a call has no result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

impl RpcError {
    fn invalid(message: impl Into<String>) -> RpcError {
        RpcError {
            code: INVALID_PARAMS,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> RpcError {
        RpcError {
            code: INTERNAL_ERROR,
            message: message.into(),
        }
    }
}

/// The result frame.
pub fn result_frame(id: &Value, value: Value) -> String {
    json!({"jsonrpc": "2.0", "id": id, "result": value}).to_string()
}

/// The error frame.
pub fn error_frame(id: &Value, code: i64, message: &str) -> String {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}}).to_string()
}

/// Parse one text frame. `Err` is the complete error frame to send back.
pub fn parse(text: &str) -> Result<Call, String> {
    let value: Value = serde_json::from_str(text)
        .map_err(|_| error_frame(&Value::Null, PARSE_ERROR, "parse error"))?;
    if value.is_array() {
        return Err(error_frame(
            &Value::Null,
            INVALID_REQUEST,
            "batching is not supported",
        ));
    }
    let Some(object) = value.as_object() else {
        return Err(error_frame(
            &Value::Null,
            INVALID_REQUEST,
            "a call is a JSON object",
        ));
    };
    let id = match object.get("id") {
        Some(id @ (Value::Number(_) | Value::String(_))) => id.clone(),
        _ => {
            return Err(error_frame(
                &Value::Null,
                INVALID_REQUEST,
                "a numeric or string id is required (notifications are not served)",
            ));
        }
    };
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(error_frame(&id, INVALID_REQUEST, "jsonrpc must be \"2.0\""));
    }
    let Some(method) = object.get("method").and_then(Value::as_str) else {
        return Err(error_frame(&id, INVALID_REQUEST, "method must be a string"));
    };
    let params = match object.get("params") {
        None | Some(Value::Null) => Value::Object(Map::new()),
        Some(p @ Value::Object(_)) => p.clone(),
        Some(_) => return Err(error_frame(&id, INVALID_PARAMS, "params must be an object")),
    };
    Ok(Call {
        id,
        method: method.to_owned(),
        params,
    })
}

fn string<'a>(params: &'a Value, key: &str) -> Result<&'a str, RpcError> {
    params
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| RpcError::invalid(format!("{key} must be a string")))
}

fn steps(params: &Value) -> Result<u64, RpcError> {
    params
        .get("steps")
        .and_then(Value::as_u64)
        .filter(|s| *s <= PLAYTEST_MAX_TICKS)
        .ok_or_else(|| {
            RpcError::invalid(format!(
                "steps must be an integer in [0, {PLAYTEST_MAX_TICKS}]"
            ))
        })
}

/// `width`/`height`: absent, null or 0 is the render target's; otherwise
/// both, each in [1, 4096].
fn size(params: &Value) -> Result<(u32, u32), RpcError> {
    let edge = |key: &str| -> Result<u32, RpcError> {
        match params.get(key) {
            None | Some(Value::Null) => Ok(0),
            Some(v) => v
                .as_u64()
                .filter(|n| *n <= u64::from(MAX_CAPTURE_EDGE))
                .map(|n| n as u32)
                .ok_or_else(|| RpcError::invalid("width and height must be integers in [0, 4096]")),
        }
    };
    match (edge("width")?, edge("height")?) {
        (0, 0) => Ok((0, 0)),
        (w, h) if w > 0 && h > 0 => Ok((w, h)),
        _ => Err(RpcError::invalid("width and height go together")),
    }
}

fn or_null<T: Into<Value>>(value: Option<T>) -> Value {
    value.map_or(Value::Null, Into::into)
}

fn coordinate(params: &Value, key: &str) -> Option<f32> {
    params
        .get(key)
        .and_then(Value::as_f64)
        .filter(|v| v.is_finite() && v.abs() <= f64::from(f32::MAX))
        .map(|v| v as f32)
}

impl Probe {
    /// The transport-free entry: one JSON-RPC text frame in, one out. The web
    /// host drives it from its request ring; the WebSocket server from its
    /// queue. Never call it while a system runs.
    pub fn call(&mut self, world: &mut World, frame: &str) -> String {
        match parse(frame) {
            Ok(call) => self.answer(world, &call),
            Err(error) => error,
        }
    }

    /// Answer a parsed call with its complete frame. A fault inside the world
    /// still answers (`-32603`), so a caller never waits forever.
    pub fn answer(&mut self, world: &mut World, call: &Call) -> String {
        let answered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.invoke(world, &call.method, &call.params)
        }));
        match answered {
            Ok(Ok(value)) => result_frame(&call.id, value),
            Ok(Err(err)) => error_frame(&call.id, err.code, &err.message),
            Err(panic) => {
                let detail = panic
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| panic.downcast_ref::<&str>().copied())
                    .unwrap_or("unknown fault");
                error_frame(
                    &call.id,
                    INTERNAL_ERROR,
                    &format!("{} failed: {detail}", call.method),
                )
            }
        }
    }

    /// One member by name with JSON params, as the wire carries it.
    pub fn invoke(
        &mut self,
        world: &mut World,
        method: &str,
        p: &Value,
    ) -> Result<Value, RpcError> {
        if !served_members().contains(&method) {
            return Err(RpcError {
                code: METHOD_NOT_FOUND,
                message: format!("method not found: {method}"),
            });
        }
        let internal =
            |err: og_core::Error| RpcError::internal(format!("{method}: {}", err.reason()));
        Ok(match method {
            "hello" => self.hello(world),
            "tick" => json!(self.tick(world)),
            "state" => self.state(world),
            "advance" => json!(self.advance(world, steps(p)?).map_err(internal)?),
            "pause" => json!(self.pause()),
            "resume" => json!(self.resume()),
            "accepted" => json!(self.accepted()),
            "input" => {
                let event = p
                    .get("event")
                    .ok_or_else(|| RpcError::invalid("event is required"))?;
                let at = crate::probe::stamp_of(event);
                json!(InjectedInput::from_json(event).is_some_and(|e| self.input_at(world, &e, at)))
            }
            "pick" => match (coordinate(p, "x"), coordinate(p, "y")) {
                (Some(x), Some(y)) => or_null(self.pick(world, x, y)),
                _ => return Err(RpcError::invalid("x and y must be finite numbers")),
            },
            "entity" => or_null(self.entity(world, string(p, "id")?)),
            "bounds" => or_null(self.bounds(world, string(p, "id")?)),
            "patch" => {
                let id = string(p, "id")?.to_owned();
                let parameter = string(p, "parameter")?.to_owned();
                let value = p
                    .get("value")
                    .ok_or_else(|| RpcError::invalid("value is required"))?;
                json!(self.patch(world, &id, &parameter, value))
            }
            "run" => {
                let program = p
                    .get("program")
                    .and_then(Value::as_array)
                    .ok_or_else(|| RpcError::invalid("program must be an array"))?;
                self.run(world, program).map_err(internal)?
            }
            "capture" => {
                let (width, height) = size(p)?;
                or_null(self.capture(world, width, height))
            }
            "sweep" => {
                let values = p
                    .get("values")
                    .and_then(Value::as_array)
                    .filter(|v| !v.is_empty() && v.len() <= MAX_SWEEP_VALUES)
                    .ok_or_else(|| {
                        RpcError::invalid(format!(
                            "values must be an array of 1 to {MAX_SWEEP_VALUES} values"
                        ))
                    })?;
                let (width, height) = size(p)?;
                let desc = SweepDesc {
                    id: string(p, "id")?.to_owned(),
                    parameter: string(p, "parameter")?.to_owned(),
                    values: values.clone(),
                    steps: steps(p)?,
                    width,
                    height,
                };
                or_null(self.sweep(world, &desc).map_err(internal)?)
            }
            "reload" => or_null(self.reload(world)),
            "snapshot" => self.snapshot(world),
            "restore" => json!(self.restore(world, string(p, "id")?)),
            "dump" => {
                let kind = match string(p, "what")? {
                    "world" => DumpKind::World,
                    "entity" => DumpKind::Entity,
                    "ui" => DumpKind::Ui,
                    _ => return Err(RpcError::invalid("what must be world, entity or ui")),
                };
                let id = if kind == DumpKind::Entity {
                    string(p, "id")?
                } else {
                    ""
                };
                or_null(self.dump(world, kind, id))
            }
            "commands" => or_null(self.commands(world, string(p, "script")?)),
            "listen" => {
                let seconds = match p.get("seconds") {
                    None | Some(Value::Null) => 1.0,
                    Some(v) => v
                        .as_f64()
                        .filter(|s| s.is_finite() && (0.0..=2.0).contains(s))
                        .ok_or_else(|| RpcError::invalid("seconds must be a number from 0 to 2"))?
                        as f32,
                };
                or_null(self.listen(world, seconds))
            }
            _ => unreachable!("served_members lists only implemented members"),
        })
    }
}
