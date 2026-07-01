//! Builds the Vue frontend and wires rebuild triggers for `rust-embed`.

use std::path::Path;
use std::process::Command;

fn main() {
    let frontend_dir = Path::new("../frontend");
    let dist_dir = frontend_dir.join("dist");

    println!("cargo:rerun-if-changed=../frontend/package.json");
    println!("cargo:rerun-if-changed=../frontend/vite.config.ts");
    println!("cargo:rerun-if-changed=../frontend/index.html");
    println!("cargo:rerun-if-changed=../frontend/src");
    println!("cargo:rerun-if-changed=../frontend/dist/index.html");

    if std::env::var("SKIP_WEB_BUILD").is_ok() {
        assert!(
            dist_dir.join("index.html").exists(),
            "frontend/dist missing; run `npm run build` in frontend/ or unset SKIP_WEB_BUILD"
        );
        return;
    }

    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };

    let status = Command::new(npm)
        .args(["run", "build"])
        .current_dir(frontend_dir)
        .status()
        .expect("failed to spawn npm");

    if !status.success() {
        panic!("frontend build failed");
    }
}
