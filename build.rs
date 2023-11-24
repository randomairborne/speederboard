fn main() {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("No git command found, or other git error!");
    let git_output = String::from_utf8(output.stdout).unwrap();
    let git_hash = git_output.trim();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}
