use crate::rpc::{RpcRequest, RpcResponse};
use crossbeam_channel::{unbounded, Sender, Receiver};
use std::collections::HashMap;

pub struct App {
    pub tx_out: Sender<RpcResponse>,
    pub tx_cmd: Sender<RpcRequest>,
    pub rx_cmd: Receiver<RpcRequest>,

    pub windows: HashMap<String, tao::window::Window>,
    pub webviews: HashMap<String, wry::WebView>,
}

impl App {
    pub fn new() -> Self {
        let (tx_cmd, rx_cmd) = unbounded::<RpcRequest>();
    let (tx_out, rx_out) = unbounded::<RpcResponse>();

        // IO read thread
        {
            let tx_cmd_in = tx_cmd.clone();
            let tx_out_in = tx_out.clone();
            std::thread::spawn(move || {
                use std::io::{self, BufRead};
                let stdin = io::stdin();
                for line in stdin.lock().lines() {
                    match line {
                        Ok(line) => {
                            if line.trim().is_empty() { continue; }
                            match serde_json::from_str::<RpcRequest>(&line) {
                                Ok(req) => { let _ = tx_cmd_in.send(req); }
                                Err(err) => {
                                    let _ = tx_out_in.send(RpcResponse::error(crate::rpc::RpcId::Null, -32700, format!("Parse error: {}", err)));
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        // IO write thread
        {
            std::thread::spawn(move || {
                use std::io::{self, Write};
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                while let Ok(resp) = rx_out.recv() {
                    if let Ok(s) = serde_json::to_string(&resp) {
                        let _ = writeln!(handle, "{}", s);
                        let _ = handle.flush();
                    }
                }
            });
        }

        Self {
            tx_out,
            tx_cmd,
            rx_cmd,
            windows: HashMap::new(),
            webviews: HashMap::new(),
        }
    }
}
