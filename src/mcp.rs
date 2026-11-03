//! Minimal MCP stdio transport. Stdout contains newline-delimited JSON-RPC only.
use crate::{
    engine::Request,
    model::Result,
    service::{Reply, Service},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{
    io::{BufRead, Write},
    sync::{Arc, Mutex},
};

pub const MAX_MESSAGE: usize = 2 * 1024 * 1024;
const PROTOCOL: &str = "2025-11-25";

fn error(id: Value, code: i32, message: impl Into<String>) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message.into()}})
}

#[derive(Default)]
pub struct Session {
    initialized: bool,
    ready: bool,
}
impl Session {
    pub fn handle(&mut self, message: Value, service: &mut Service) -> Option<Value> {
        self.handle_with(message, |request| service.execute(request))
    }
    fn handle_with(
        &mut self,
        message: Value,
        mut execute: impl FnMut(Request) -> Result<Reply>,
    ) -> Option<Value> {
        let id = message.get("id").cloned();
        let Some(method) = message.get("method").and_then(Value::as_str) else {
            return Some(error(id.unwrap_or(Value::Null), -32600, "Missing method"));
        };
        if !message.is_object()
            || message.get("jsonrpc") != Some(&json!("2.0"))
            || id
                .as_ref()
                .is_some_and(|v| !v.is_string() && !v.is_i64() && !v.is_u64())
        {
            return Some(error(Value::Null, -32600, "Invalid JSON-RPC request"));
        }
        if id.is_none() {
            if method == "notifications/initialized" && self.initialized {
                self.ready = true;
            }
            return None;
        }
        let id = id.unwrap();
        let params = message.get("params").cloned().unwrap_or_else(|| json!({}));
        let result = match method {
            "initialize" => {
                if self.initialized {
                    return Some(error(id, -32600, "Session is already initialized"));
                }
                let Some(version) = params.get("protocolVersion").and_then(Value::as_str) else {
                    return Some(error(id, -32602, "protocolVersion is required"));
                };
                let negotiated =
                    if ["2024-11-05", "2025-03-26", "2025-06-18", PROTOCOL].contains(&version) {
                        version
                    } else {
                        PROTOCOL
                    };
                self.initialized = true;
                json!({"protocolVersion":negotiated,"capabilities":{"tools":{"listChanged":false}},"serverInfo":{"name":"ignis-engine","version":env!("CARGO_PKG_VERSION")},"instructions":"Use engine_command. Inspect before edits; expected_revision guards stale changes. Editing resets/pauses the simulation. Pause before deterministic stepping. Scene and artifact paths are relative to the project root. create_project/open_project explicitly select an absolute project folder. Capture returns real PNG pixels plus state metadata."})
            }
            "ping" => json!({}),
            _ if !self.ready => {
                return Some(error(
                    id,
                    -32002,
                    "Initialize and send notifications/initialized first",
                ));
            }
            "tools/list" => {
                json!({"tools":[{"name":"engine_command","description":"Inspect, edit, generate, save, play, pause, step, capture and replay the same 3D world used by the native window. All commands use the shared typed engine API. Editing resets the runtime; saves use the authoring scene. Invoke inspect to obtain revision, then provide expected_revision for guarded edits.","inputSchema":schemars::schema_for!(Request),"annotations":{"readOnlyHint":false,"destructiveHint":true,"idempotentHint":false,"openWorldHint":true}}]})
            }
            "tools/call" => {
                if params.get("name").and_then(Value::as_str) != Some("engine_command") {
                    return Some(error(id, -32602, "Unknown tool"));
                }
                let request = serde_json::from_value::<Request>(
                    params.get("arguments").cloned().unwrap_or(Value::Null),
                );
                match request {
                    Err(e) => {
                        json!({"isError":true,"content":[{"type":"text","text":json!({"code":"invalid_argument","message":e.to_string()}).to_string()}]})
                    }
                    Ok(request) => match execute(request) {
                        Ok(reply) => {
                            let mut content =
                                vec![json!({"type":"text","text":reply.data.to_string()})];
                            if let Some(png) = reply.png {
                                content.push(json!({"type":"image","mimeType":"image/png","data":STANDARD.encode(png)}));
                            }
                            json!({"isError":false,"content":content,"structuredContent":reply.data})
                        }
                        Err(e) => {
                            json!({"isError":true,"content":[{"type":"text","text":json!({"code":e.code,"message":e.message}).to_string()}],"structuredContent":{"code":e.code,"message":e.message}})
                        }
                    },
                }
            }
            _ => return Some(error(id, -32601, format!("Unknown method: {method}"))),
        };
        Some(json!({"jsonrpc":"2.0","id":id,"result":result}))
    }
}

pub fn serve(
    reader: impl BufRead,
    mut writer: impl Write,
    service: Arc<Mutex<Service>>,
) -> std::io::Result<()> {
    let mut reader = reader;
    let mut session = Session::default();
    let mut bytes = Vec::new();
    loop {
        bytes.clear();
        let read = std::io::Read::take(&mut reader, (MAX_MESSAGE + 1) as u64)
            .read_until(b'\n', &mut bytes)?;
        if read == 0 {
            break;
        }
        let response = if bytes.len() > MAX_MESSAGE {
            Some(error(Value::Null, -32600, "Message exceeds 2 MiB"))
        } else {
            match serde_json::from_slice(&bytes) {
                Ok(message) => session.handle_with(message, |request| {
                    // Only command execution needs the world lock. JSON text/base64 and
                    // protocol-only requests are handled after releasing it.
                    service.lock().unwrap().execute(request)
                }),
                Err(_) => Some(error(Value::Null, -32700, "Invalid JSON")),
            }
        };
        if let Some(response) = response {
            serde_json::to_writer(&mut writer, &response)?;
            writer.write_all(b"\n")?;
            writer.flush()?;
        }
        if bytes.len() > MAX_MESSAGE || service.lock().unwrap().engine.exit_requested {
            break;
        }
    }
    Ok(())
}
