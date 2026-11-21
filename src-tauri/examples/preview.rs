//! Serves the screens in a normal browser, with the same Rust behind them,
//! for checking the screens without building the app:
//!
//!     cargo run -p real-denim-fit --example preview -- 1420
//!
//! Then open http://localhost:1420. What you enter is kept in
//! target/preview-state.json, not in the app's own data. Set
//! RDF_CATALOGUE_DIR at build time to try another catalogue folder, and
//! RDF_GUIDE_DIR to serve the measuring guide from another folder.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

use real_denim_fit_lib as app;
use serde_json::{json, Value};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn parse<T: serde::de::DeserializeOwned>(args: &Value, k: &str) -> Result<T, String> {
    serde_json::from_value(args.get(k).cloned().unwrap_or(Value::Null)).map_err(|e| format!("{k}: {e}"))
}

fn handle(cmd: &str, args: Value, data: &app::Data) -> Result<Value, String> {
    let state = root().join("target/preview-state.json");
    let arg = |k: &str| args.get(k).cloned().unwrap_or(Value::Null);
    let to = |v: Result<Value, serde_json::Error>| v.map_err(|e| e.to_string());
    match cmd {
        "check_for_update" => Ok(Value::Null),
        "load_state" => Ok(std::fs::read_to_string(&state)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or(Value::Null)),
        "save_state" => {
            std::fs::write(&state, serde_json::to_vec_pretty(&arg("state")).unwrap()).map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        "catalogue_index" => to(serde_json::to_value(app::index(data))),
        "match_catalogue" => to(serde_json::to_value(app::matches(
            data,
            &parse(&args, "profile")?,
            &parse(&args, "filters")?,
        ))),
        "product_details" => {
            let id: String = parse(&args, "productId")?;
            to(serde_json::to_value(app::details(data, &id)?))
        }
        "reference_pair_ease" => to(serde_json::to_value(app::own_ease(
            data,
            &parse(&args, "body")?,
            &parse(&args, "pair")?,
        ))),
        _ => Err(format!("unknown command {cmd}")),
    }
}

fn respond(mut stream: TcpStream, status: &str, kind: &str, body: &[u8]) {
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

fn serve(stream: TcpStream, data: &app::Data) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }
    let mut parts = line.split_whitespace();
    let (method, path) = (parts.next().unwrap_or(""), parts.next().unwrap_or("/").to_string());
    let mut length = 0;
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h).is_err() || h.trim().is_empty() {
            break;
        }
        if let Some(v) = h.to_ascii_lowercase().strip_prefix("content-length:") {
            length = v.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0; length];
    let _ = reader.read_exact(&mut body);

    if method == "POST" {
        if let Some(cmd) = path.strip_prefix("/api/") {
            let args: Value = serde_json::from_slice(&body).unwrap_or(json!({}));
            return match handle(cmd, args, data) {
                Ok(v) => respond(stream, "200 OK", "application/json", v.to_string().as_bytes()),
                Err(e) => respond(stream, "500 Internal Server Error", "text/plain", e.as_bytes()),
            };
        }
    }
    let file = if path == "/" {
        "index.html".to_string()
    } else {
        path.trim_start_matches('/').to_string()
    };
    if file.contains("..") {
        return respond(stream, "404 Not Found", "text/plain", b"not found");
    }
    let kind = match file.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "text/javascript",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    };
    // RDF_GUIDE_DIR serves ui/guide/ from elsewhere, to try a guide that
    // isn't in this checkout yet.
    let path = match (file.strip_prefix("guide/"), std::env::var("RDF_GUIDE_DIR")) {
        (Some(rest), Ok(dir)) => PathBuf::from(dir).join(rest),
        _ => root().join("ui").join(&file),
    };
    match std::fs::read(path) {
        Ok(bytes) => respond(stream, "200 OK", kind, &bytes),
        Err(_) => respond(stream, "404 Not Found", "text/plain", b"not found"),
    }
}

fn main() {
    let port = std::env::args().nth(1).unwrap_or_else(|| "1420".into());
    let data = app::data();
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).expect("port free");
    println!("Preview at http://localhost:{port}");
    for stream in listener.incoming().flatten() {
        serve(stream, &data);
    }
}
