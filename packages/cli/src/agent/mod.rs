//! Thin stdio MCP server exposing plan/apply/doctor/config/sbom over internal APIs.

use crate::commands::{config as config_cmd, doctor, sbom};
use crate::planner;
use crate::session::LunaSession;
use miette::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

pub fn run_mcp_stdio(session: &LunaSession) -> Result<i32> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| miette::miette!("stdin: {e}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let req: JsonRpcRequest =
            serde_json::from_str(&line).map_err(|e| miette::miette!("invalid JSON-RPC: {e}"))?;
        let resp = handle_request(session, req);
        let out = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".into());
        writeln!(stdout, "{out}").ok();
        stdout.flush().ok();
    }
    Ok(0)
}

fn handle_request(session: &LunaSession, req: JsonRpcRequest) -> JsonRpcResponse {
    let id = req.id;
    match req.method.as_str() {
        "initialize" => ok(
            id,
            json!({ "protocolVersion": "2024-11-05", "serverInfo": { "name": "luna", "version": "0.1.0" } }),
        ),
        "tools/list" => ok(
            id,
            json!({
                "tools": [
                    { "name": "plan", "description": "Build execution plan for target" },
                    { "name": "doctor", "description": "Run workspace doctor checks" },
                    { "name": "config", "description": "Validate and print luna.toml" },
                    { "name": "sbom", "description": "Export dependency inventory" },
                ]
            }),
        ),
        "tools/call" => match tool_call(session, req.params) {
            Ok(v) => ok(id, v),
            Err(e) => err(id, -32000, e),
        },
        _ => err(id, -32601, format!("method not found: {}", req.method)),
    }
}

fn tool_call(session: &LunaSession, params: Option<Value>) -> Result<Value, String> {
    let params = params.ok_or_else(|| String::from("missing params"))?;
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| String::from("missing tool name"))?;
    let args = params.get("arguments").cloned().unwrap_or(json!({}));
    let root = session.root.as_path();
    let config = &session.config;
    let mut global = session.cli.global.clone();
    global.json = true;

    match name {
        "plan" => {
            let target = args
                .get("target")
                .and_then(|v| v.as_str())
                .unwrap_or("sync");
            let plan = planner::build_plan(root, config, target).map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(plan).unwrap_or(json!({})))
        }
        "doctor" => {
            let code =
                doctor::run_doctor(root, config, &global, &crate::cli::DoctorArgs { ci: false })
                    .map_err(|e| e.to_string())?;
            Ok(json!({ "exit_code": code }))
        }
        "config" => {
            config_cmd::validate_cmd(root, &global).map_err(|e| e.to_string())?;
            Ok(json!({ "valid": true }))
        }
        "sbom" => sbom::collect_inventory(root, config)
            .map(|items| json!({ "components": items }))
            .map_err(|e| e.to_string()),
        other => Err(format!("unknown tool: {other}")),
    }
}

fn ok(id: Option<Value>, result: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: Some(result),
        error: None,
    }
}

fn err(id: Option<Value>, code: i32, message: String) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(JsonRpcError { code, message }),
    }
}
