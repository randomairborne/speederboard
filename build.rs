fn main() {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("No git command found, or other git error");
    if !output.status.success() {
        panic!("Bad exit status: {}", output.status)
    }
    let git_output = String::from_utf8(output.stdout).unwrap();
    let git_hash = git_output.trim();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}
