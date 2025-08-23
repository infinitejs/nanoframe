mod rpc;
mod state;
mod window_ops;
mod dialogs;
mod system;

use crate::rpc::{RpcId, RpcResponse};
use crate::state::App;
use crate::window_ops::handle_window_event;
use serde_json::{json, Value};
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoop};

fn main() {
    // Init app state and channels
    let mut app = App::new();

    // GUI event loop
    let event_loop = EventLoop::new();

    event_loop.run(move |event, target, control_flow| {
        *control_flow = ControlFlow::Poll;

        // Process incoming commands non-blocking
        while let Ok(req) = app.rx_cmd.try_recv() {
            let id = RpcId::from_value(req.id.clone().unwrap_or(Value::Null));
            let tx_out = app.tx_out.clone();
            let send_err = |code: i32, msg: String| {
                let _ = tx_out.send(RpcResponse::error(id.clone(), code, msg));
            };

            match req.method.as_str() {
                "ping" => {
                    let _ = app.tx_out.send(RpcResponse::result(id, json!("pong")));
                }
                "createWindow" => {
                    match window_ops::create_window_with_target(target, &mut app, req.params) {
                        Ok(val) => { let _ = tx_out.send(RpcResponse::result(id, val)); }
                        Err(e) => send_err(-32000, e.to_string()),
                    }
                }
                // Window controls
                "window.show" => window_ops::op_show(&mut app, req.params, id),
                "window.hide" => window_ops::op_hide(&mut app, req.params, id),
                "window.close" => window_ops::op_close(&mut app, req.params, id, control_flow),
                "window.setIcon" => window_ops::op_set_icon(&mut app, req.params, id),
                "webview.eval" => window_ops::op_eval(&mut app, req.params, id),
                // Extended window ops
                "window.maximize" => window_ops::op_maximize(&mut app, req.params, id),
                "window.minimize" => window_ops::op_minimize(&mut app, req.params, id),
                "window.unminimize" => window_ops::op_unminimize(&mut app, req.params, id),
                "window.focus" => window_ops::op_focus(&mut app, req.params, id),
                "window.setTitle" => window_ops::op_set_title(&mut app, req.params, id),
                "window.setSize" => window_ops::op_set_size(&mut app, req.params, id),
                "window.getSize" => window_ops::op_get_size(&mut app, req.params, id),
                "window.center" => window_ops::op_center(&mut app, req.params, id),
                "window.setAlwaysOnTop" => window_ops::op_set_always_on_top(&mut app, req.params, id),
                "window.setResizable" => window_ops::op_set_resizable(&mut app, req.params, id),
                "window.isVisible" => window_ops::op_is_visible(&mut app, req.params, id),
                "window.setFullscreen" => window_ops::op_set_fullscreen(&mut app, req.params, id),
                "window.isFullscreen" => window_ops::op_is_fullscreen(&mut app, req.params, id),
                "window.setDecorations" => window_ops::op_set_decorations(&mut app, req.params, id),
                "window.setPosition" => window_ops::op_set_position(&mut app, req.params, id),
                "window.getPosition" => window_ops::op_get_position(&mut app, req.params, id),
                // Webview extras
                "webview.openDevtools" => window_ops::op_open_devtools(&mut app, req.params, id),
                "webview.postMessage" => window_ops::op_post_message(&mut app, req.params, id),
                // Dialogs + app paths
                "dialog.open" => dialogs::op_open_dialog(&mut app, req.params, id),
                "dialog.save" => dialogs::op_save_dialog(&mut app, req.params, id),
                "app.getPath" => dialogs::op_app_get_path(&mut app, req.params, id),
                // System helpers
                "shell.openExternal" => system::op_shell_open(&mut app, req.params, id),
                "clipboard.writeText" => system::op_clipboard_write(&mut app, req.params, id),
                "clipboard.readText" => system::op_clipboard_read(&mut app, req.params, id),
                _ => send_err(-32601, "Method not found".to_string()),
            }
        }

        match &event {
            Event::WindowEvent { event, window_id, .. } => handle_window_event(event, *window_id, control_flow, &mut app),
            Event::MainEventsCleared => {}
            _ => {}
        }
    });
}
