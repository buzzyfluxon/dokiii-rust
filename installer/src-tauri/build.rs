use std::fs;
use std::path::Path;

fn main() {
    let payload = Path::new("payload/DOKIII-Setup.exe");
    if !payload.exists() {
        let _ = fs::create_dir_all("payload");
        let _ = fs::write(payload, b"");
    }
    println!("cargo:rerun-if-changed=payload/DOKIII-Setup.exe");
    tauri_build::build()
}
