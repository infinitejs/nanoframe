use crate::rpc::{RpcId, RpcResponse};
use crate::state::App;
use directories::BaseDirs;
use directories::ProjectDirs;
use rfd::FileDialog;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
struct FileFilter { name: Option<String>, extensions: Option<Vec<String>> }

#[derive(Debug, Deserialize)]
struct OpenDialogParams {
    title: Option<String>,
    directory: Option<bool>,
    multiple: Option<bool>,
    filters: Option<Vec<FileFilter>>,
}

pub fn op_open_dialog(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<OpenDialogParams>(params) {
        Ok(p) => {
            let mut dlg = FileDialog::new();
            if let Some(t) = p.title { dlg = dlg.set_title(&t); }
            if let Some(filters) = p.filters {
                for f in filters {
                    if let Some(exts) = f.extensions { dlg = dlg.add_filter(f.name.as_deref().unwrap_or(""), &exts); }
                }
            }
            let directory = p.directory.unwrap_or(false);
            let multiple = p.multiple.unwrap_or(false);
            let result = if directory {
                if multiple { dlg.pick_folders().unwrap_or_default().into_iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>() }
                else { dlg.pick_folder().map(|p| vec![p.to_string_lossy().to_string()]).unwrap_or_default() }
            } else {
                if multiple { dlg.pick_files().unwrap_or_default().into_iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>() }
                else { dlg.pick_file().map(|p| vec![p.to_string_lossy().to_string()]).unwrap_or_default() }
            };
            let _ = app.tx_out.send(RpcResponse::result(id, json!({ "paths": result })));
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

#[derive(Debug, Deserialize)]
struct SaveDialogParams { title: Option<String>, default_file_name: Option<String> }

pub fn op_save_dialog(app: &mut App, params: Value, id: RpcId) {
    match serde_json::from_value::<SaveDialogParams>(params) {
        Ok(p) => {
            let mut dlg = FileDialog::new();
            if let Some(t) = p.title { dlg = dlg.set_title(&t); }
            if let Some(name) = p.default_file_name { dlg = dlg.set_file_name(&name); }
            let result = dlg.save_file().map(|p| p.to_string_lossy().to_string());
            let _ = app.tx_out.send(RpcResponse::result(id, json!({ "path": result })));
        }
        Err(e) => { let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string())); }
    }
}

#[derive(Debug, Deserialize)]
struct GetPathParams { name: String, app_name: Option<String> }

pub fn op_app_get_path(app: &mut App, params: Value, id: RpcId) {
    let res = match serde_json::from_value::<GetPathParams>(params) {
        Ok(p) => {
            let name = p.name.as_str();
            let app_name = p.app_name.as_deref().unwrap_or("nanoframe-app");
            let val = match name {
                "home" => BaseDirs::new().map(|b| b.home_dir().to_path_buf()),
                "temp" => Some(std::env::temp_dir()),
                "appData" => ProjectDirs::from("", "", app_name).map(|p| p.data_dir().to_path_buf()),
                "userData" => ProjectDirs::from("", "", app_name).map(|p| p.data_dir().join("User Data")),
                _ => None,
            };
            json!({ "path": val.map(|p| p.to_string_lossy().to_string()) })
        }
        Err(e) => {
            let _ = app.tx_out.send(RpcResponse::error(id, -32602, e.to_string()));
            return;
        }
    };
    let _ = app.tx_out.send(RpcResponse::result(id, res));
}
