use crate::rpc::{RpcId, RpcResponse};
use crate::state::App;
use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
struct OpenParams { target: String }

pub fn op_shell_open(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<OpenParams>(params) {
        Ok(p) => {
            match open::that(&p.target) {
                Ok(_) => { let _ = app.tx_out.send(RpcResponse::result(id, json!(true))); }
                Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -33001, e.to_string())); }
            }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

#[derive(Debug, Deserialize)]
struct ClipboardWrite { text: String }

pub fn op_clipboard_write(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<ClipboardWrite>(params) {
        Ok(p) => {
            let r = (|| -> Result<()> { let mut cb = arboard::Clipboard::new()?; cb.set_text(p.text)?; Ok(()) })();
            match r { Ok(()) => { let _ = app.tx_out.send(RpcResponse::result(id, json!(true))); }, Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -33002, e.to_string())); } }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

pub fn op_clipboard_read(app: &mut App, _params: Value, id: RpcId) {
    let r = (|| -> Result<String> { let mut cb = arboard::Clipboard::new()?; Ok(cb.get_text()?) })();
    match r { Ok(text) => { let _ = app.tx_out.send(RpcResponse::result(id, json!({"text": text}))); }, Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -33003, e.to_string())); } }
}
