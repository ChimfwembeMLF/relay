use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let frontend_dir = manifest_dir.join("frontend");

    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/package-lock.json");
    println!("cargo:rerun-if-changed=frontend/vite.config.ts");
    println!("cargo:rerun-if-changed=frontend/tsconfig.json");
    println!("cargo:rerun-if-changed=frontend/index.html");
    println!("cargo:rerun-if-changed=frontend/src");

    if env::var("SKIP_FRONTEND_BUILD").is_ok() {
        println!("cargo:warning=SKIP_FRONTEND_BUILD set — skipping React pay page build");
        return;
    }

    if !frontend_dir.join("package.json").exists() {
        println!("cargo:warning=frontend/package.json not found — skipping pay page build");
        return;
    }

    let npm = find_npm();

    let node_modules = frontend_dir.join("node_modules");
    if !node_modules.exists() {
        run(&npm, &["install"], &frontend_dir, "npm install");
    }

    run(&npm, &["run", "build"], &frontend_dir, "npm run build");
}

fn find_npm() -> String {
    if Command::new("npm").arg("--version").output().is_ok() {
        return "npm".into();
    }
    // Common Homebrew path on macOS when cargo's PATH is minimal
    for candidate in ["/opt/homebrew/bin/npm", "/usr/local/bin/npm"] {
        if Path::new(candidate).exists() {
            return candidate.into();
        }
    }
    panic!(
        "npm not found on PATH. Install Node.js, or set SKIP_FRONTEND_BUILD=1 to skip the pay UI build."
    );
}

fn run(npm: &str, args: &[&str], dir: &Path, label: &str) {
    let status = Command::new(npm)
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap_or_else(|e| panic!("failed to run `{label}`: {e}"));

    if !status.success() {
        panic!("`{label}` failed with {status}");
    }
}
