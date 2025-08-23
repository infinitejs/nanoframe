use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use tao::event::WindowEvent;
use tao::event_loop::{ControlFlow, EventLoopWindowTarget};
use tao::window::{Icon, WindowBuilder};
use uuid::Uuid;
use wry::WebViewBuilder;

use crate::rpc::{RpcId, RpcResponse};
use crate::state::App;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")] // accept camelCase from JS
struct CreateWindowParams {
    title: Option<String>,
    url: Option<String>,
    html: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    icon_path: Option<String>,
    resizable: Option<bool>,
    always_on_top: Option<bool>,
    fullscreen: Option<bool>,
    decorations: Option<bool>,
    center: Option<bool>,
    preload: Option<String>,
}

pub fn create_window_with_target(
    target: &EventLoopWindowTarget<()>,
    app: &mut App,
    params: Value,
) -> Result<Value> {
    let p: CreateWindowParams = serde_json::from_value(params)?;

    // Build tao window
    let mut wb = WindowBuilder::new();
    if let Some(title) = p.title { wb = wb.with_title(title); }
    if let (Some(w), Some(h)) = (p.width, p.height) { wb = wb.with_inner_size(tao::dpi::LogicalSize::new(w as f64, h as f64)); }
    if let Some(icon_path) = p.icon_path.as_deref() { if let Ok(icon) = load_icon(icon_path) { wb = wb.with_window_icon(Some(icon)); } }
    if let Some(v) = p.resizable { wb = wb.with_resizable(v); }
    if let Some(v) = p.always_on_top { wb = wb.with_always_on_top(v); }
    if let Some(v) = p.fullscreen { if v { wb = wb.with_fullscreen(Some(tao::window::Fullscreen::Borderless(None))); } }
    if let Some(v) = p.decorations { wb = wb.with_decorations(v); }

    let window = wb.build(target)?;

    // Build webview
    let mut wvb = WebViewBuilder::new();
    if let Some(script) = p.preload.as_deref() { wvb = wvb.with_initialization_script(script); }
    if let Some(url) = p.url { wvb = wvb.with_url(&url); }
    if let Some(html) = p.html { wvb = wvb.with_html(&html); }
    let webview = wvb.build(&window)?;

    let id = Uuid::new_v4().to_string();
    app.windows.insert(id.clone(), window);
    app.webviews.insert(id.clone(), webview);

    // Center after creation if requested
    if p.center.unwrap_or(false) {
        let _ = op_center(app, json!({"windowId": id.clone()}), RpcId::Null);
    }

    Ok(json!({ "windowId": id }))
}

pub fn handle_window_event(event: &WindowEvent, window_id: tao::window::WindowId, control_flow: &mut ControlFlow, app: &mut App) {
    if let WindowEvent::CloseRequested = event {
        if let Some((key, _)) = app.windows.iter().find(|(_, w)| w.id() == window_id).map(|(k, v)| (k.clone(), v.id())) {
            app.windows.remove(&key);
            app.webviews.remove(&key);
            // Notify JS bridge that a window closed
            let _ = app.tx_out.send(RpcResponse::notify("window.closed", json!({ "windowId": key })));
            if app.windows.is_empty() {
                *control_flow = ControlFlow::Exit;
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")] // accept windowId from JS
struct WithWindowId { window_id: String }

pub fn op_show(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<WithWindowId>(params) {
        Ok(p) => {
            if let Some(win) = app.windows.get(&p.window_id) {
                win.set_visible(true);
                let _ = app.tx_out.send(RpcResponse::result(id, json!(true)));
            } else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

pub fn op_hide(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<WithWindowId>(params) {
        Ok(p) => {
            if let Some(win) = app.windows.get(&p.window_id) {
                win.set_visible(false);
                let _ = app.tx_out.send(RpcResponse::result(id, json!(true)));
            } else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

pub fn op_close(app: &mut App, params: Value, id: RpcId, control_flow: &mut ControlFlow) {
    match serde_json::from_value::<WithWindowId>(params) {
        Ok(p) => {
            if let Some(win) = app.windows.remove(&p.window_id) {
                // remove associated webview
                app.webviews.remove(&p.window_id);
                win.set_visible(false);
                let _ = app.tx_out.send(RpcResponse::result(id, json!(true)));
                if app.windows.is_empty() { *control_flow = ControlFlow::Exit; }
            } else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")] // accept windowId
struct EvalParams { window_id: String, code: String }

pub fn op_eval(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<EvalParams>(params) {
        Ok(p) => {
            if let Some(wv) = app.webviews.get(&p.window_id) {
                if let Err(e) = wv.evaluate_script(&p.code) { let _ = app.tx_out.send(RpcResponse::error(id, -32002, e.to_string())); }
                else { let _ = app.tx_out.send(RpcResponse::result(id, json!(true))); }
            } else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")] // accept windowId, iconPath
struct SetIconParams { window_id: String, icon_path: String }

pub fn op_set_icon(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<SetIconParams>(params) {
        Ok(p) => {
            if let Some(win) = app.windows.get(&p.window_id) {
                match load_icon(&p.icon_path) {
                    Ok(icon) => { win.set_window_icon(Some(icon)); let _ = app.tx_out.send(RpcResponse::result(id, json!(true))); }
                    Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32003, e.to_string())); }
                }
            } else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FullscreenParams { window_id: String, value: bool }

pub fn op_set_fullscreen(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<FullscreenParams>(params) {
        Ok(p) => {
            if let Some(win) = app.windows.get(&p.window_id) {
                if p.value { win.set_fullscreen(Some(tao::window::Fullscreen::Borderless(None))); }
                else { win.set_fullscreen(None); }
                let _ = app.tx_out.send(RpcResponse::result(id, json!(true)));
            } else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WithWindowIdOnly { window_id: String }

pub fn op_is_fullscreen(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<WithWindowIdOnly>(params) {
        Ok(p) => {
            if let Some(win) = app.windows.get(&p.window_id) {
                let on = win.fullscreen().is_some();
                let _ = app.tx_out.send(RpcResponse::result(id, json!(on)));
            } else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DecorationsParams { window_id: String, value: bool }

pub fn op_set_decorations(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<DecorationsParams>(params) {
        Ok(p) => {
            if let Some(win) = app.windows.get(&p.window_id) { win.set_decorations(p.value); let _ = app.tx_out.send(RpcResponse::result(id, json!(true))); }
            else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PositionParams { window_id: String, x: i32, y: i32 }

pub fn op_set_position(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<PositionParams>(params) {
        Ok(p) => {
            if let Some(win) = app.windows.get(&p.window_id) {
                use tao::dpi::PhysicalPosition;
                win.set_outer_position(PhysicalPosition::new(p.x, p.y));
                let _ = app.tx_out.send(RpcResponse::result(id, json!(true)));
            } else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

pub fn op_get_position(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<WithWindowIdOnly>(params) {
        Ok(p) => {
            if let Some(win) = app.windows.get(&p.window_id) {
                if let Ok(pos) = win.outer_position() { let _ = app.tx_out.send(RpcResponse::result(id, json!({"x": pos.x, "y": pos.y}))); }
                else { let _ = app.tx_out.send(RpcResponse::error(id, -32004, "Position unavailable".into())); }
            } else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostMessageParams { window_id: String, payload: serde_json::Value }

pub fn op_post_message(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<PostMessageParams>(params) {
        Ok(p) => {
            if let Some(wv) = app.webviews.get(&p.window_id) {
                let code = format!("window.dispatchEvent(new MessageEvent('message', {{ data: {} }}));", p.payload);
                if let Err(e) = wv.evaluate_script(&code) { let _ = app.tx_out.send(RpcResponse::error(id, -32002, e.to_string())); }
                else { let _ = app.tx_out.send(RpcResponse::result(id, json!(true))); }
            } else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

fn load_icon(path: &str) -> Result<Icon> {
    use std::fs::File;
    use std::io::BufReader;
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let img = image::load(reader, image::ImageFormat::from_path(path).unwrap_or(image::ImageFormat::Png))?
        .into_rgba8();
    let (w, h) = img.dimensions();
    let rgba = img.into_raw();
    Icon::from_rgba(rgba, w, h).map_err(|e| anyhow!(e))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetTitleParams { window_id: String, title: String }

pub fn op_set_title(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<SetTitleParams>(params) {
        Ok(p) => {
            if let Some(win) = app.windows.get(&p.window_id) { win.set_title(&p.title); let _ = app.tx_out.send(RpcResponse::result(id, json!(true))); }
            else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SizeParams { window_id: String, width: u32, height: u32 }

pub fn op_set_size(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<SizeParams>(params) {
        Ok(p) => {
            if let Some(win) = app.windows.get(&p.window_id) {
                use tao::dpi::PhysicalSize;
                win.set_inner_size(PhysicalSize::new(p.width, p.height));
                let _ = app.tx_out.send(RpcResponse::result(id, json!(true)));
            } else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetSizeParams { window_id: String }

pub fn op_get_size(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<GetSizeParams>(params) {
        Ok(p) => {
            if let Some(win) = app.windows.get(&p.window_id) {
                let size = win.inner_size();
                let _ = app.tx_out.send(RpcResponse::result(id, json!({"width": size.width, "height": size.height})));
            } else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

pub fn op_maximize(app: &mut App, params: Value, id: RpcId) { with_window(app, params, id, |w| { w.set_maximized(true); Ok(json!(true)) }); }
pub fn op_minimize(app: &mut App, params: Value, id: RpcId) { with_window(app, params, id, |w| { w.set_minimized(true); Ok(json!(true)) }); }
pub fn op_unminimize(app: &mut App, params: Value, id: RpcId) { with_window(app, params, id, |w| { w.set_minimized(false); Ok(json!(true)) }); }
pub fn op_focus(app: &mut App, params: Value, id: RpcId) { with_window(app, params, id, |w| { w.set_focus(); Ok(json!(true)) }); }
pub fn op_center(app: &mut App, params: Value, id: RpcId) { with_window(app, params, id, |w| {
    use tao::dpi::{PhysicalPosition};
    if let Some(m) = w.current_monitor() {
        let mpos = m.position();
        let msize = m.size();
        let wsize = w.outer_size();
        let x = mpos.x + ((msize.width as i32 - wsize.width as i32) / 2);
        let y = mpos.y + ((msize.height as i32 - wsize.height as i32) / 2);
        w.set_outer_position(PhysicalPosition::new(x, y));
        Ok(json!(true))
    } else { Ok(json!(false)) }
}); }
pub fn op_set_always_on_top(app: &mut App, params: Value, id: RpcId) { with_window_bool(app, params, id, |w, v| { w.set_always_on_top(v); Ok(json!(true)) }); }
pub fn op_set_resizable(app: &mut App, params: Value, id: RpcId) { with_window_bool(app, params, id, |w, v| { w.set_resizable(v); Ok(json!(true)) }); }
pub fn op_is_visible(app: &mut App, params: Value, id: RpcId) { with_window(app, params, id, |w| { Ok(json!(w.is_visible())) }); }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WithWindowIdBool { window_id: String, value: bool }

fn with_window<F>(app: &mut App, params: Value, id: RpcId, f: F)
where F: FnOnce(&tao::window::Window) -> Result<Value> {
    match serde_json::from_value::<WithWindowId>(params) {
        Ok(p) => {
            if let Some(win) = app.windows.get(&p.window_id) {
                match f(win) { Ok(v) => { let _ = app.tx_out.send(RpcResponse::result(id, v)); }, Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32099, e.to_string())); } }
            } else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

fn with_window_bool<F>(app: &mut App, params: Value, id: RpcId, f: F)
where F: FnOnce(&tao::window::Window, bool) -> Result<Value> {
    match serde_json::from_value::<WithWindowIdBool>(params) {
        Ok(p) => {
            if let Some(win) = app.windows.get(&p.window_id) { match f(win, p.value) { Ok(v) => { let _ = app.tx_out.send(RpcResponse::result(id, v)); }, Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32099, e.to_string())); } } }
            else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DevtoolsParams { window_id: String }

pub fn op_open_devtools(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<DevtoolsParams>(params) {
        Ok(p) => {
            if let Some(wv) = app.webviews.get(&p.window_id) {
                #[cfg(target_os = "windows")]
                let _ = wv.open_devtools();
                #[cfg(not(target_os = "windows"))]
                let _ = wv.open_devtools();
                let _ = app.tx_out.send(RpcResponse::result(id, json!(true)));
            } else { let _ = app.tx_out.send(RpcResponse::error(id, -32001, "Window not found".into())); }
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}
