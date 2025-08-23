use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum RpcId { Number(i64), String(String), Null }

impl RpcId {
    pub fn from_value(v: Value) -> Self {
        match v {
            Value::Null => RpcId::Null,
            Value::Number(n) => RpcId::Number(n.as_i64().unwrap_or(0)),
            Value::String(s) => RpcId::String(s),
            _ => RpcId::Null,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum RpcResponse {
    Result { jsonrpc: &'static str, id: RpcId, result: Value },
    Error { jsonrpc: &'static str, id: RpcId, error: RpcError },
    // JSON-RPC notification (no id)
    Notify { jsonrpc: &'static str, method: String, params: Value },
}

impl RpcResponse {
    pub fn result(id: RpcId, result: Value) -> Self { Self::Result { jsonrpc: "2.0", id, result } }
    pub fn error(id: RpcId, code: i32, message: String) -> Self { Self::Error { jsonrpc: "2.0", id, error: RpcError { code, message } } }
    pub fn notify(method: &str, params: Value) -> Self { Self::Notify { jsonrpc: "2.0", method: method.to_string(), params } }
}

#[derive(Debug, Serialize)]
pub struct RpcError { pub code: i32, pub message: String }
