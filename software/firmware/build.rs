use std::process::Command;

fn main() {
    let datetime = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    println!("cargo:rustc-env=COMPILE_TIME={}", datetime);

    set_commit_hash();
}

fn set_commit_hash() {
    let output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .expect("Failed to execute git command");

    let git_status = String::from_utf8_lossy(&output.stdout);

    if git_status.trim().is_empty() {
        let commit_hash = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()
            .expect("Failed to get commit hash");

        let commit_hash = String::from_utf8_lossy(&commit_hash.stdout).trim().to_string();
        println!("cargo:rustc-env=COMMIT_HASH={}", commit_hash);
    } else {
        println!("cargo:rustc-env=COMMIT_HASH=uncommitted changes");
    }
}
