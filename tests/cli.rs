use std::process::Command;

use tempfile::tempdir;

fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_todox")
}

#[test]
fn help_flag_prints_usage() {
    let output = Command::new(bin_path())
        .arg("--help")
        .output()
        .expect("run --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("todox"));
    assert!(stdout.contains("--data-file <PATH>"));
}

#[test]
fn data_file_override_creates_requested_json() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("custom").join("tasks.json");

    let output = Command::new(bin_path())
        .arg("--data-file")
        .arg(&path)
        .env("TODOX_DISABLE_TUI", "1")
        .output()
        .expect("run with --data-file");

    assert!(output.status.success());
    assert!(path.exists());
    assert!(path.parent().unwrap().join("config.json").exists());
}

#[test]
fn default_path_uses_home_hidden_directory() {
    let temp = tempdir().unwrap();
    let expected = temp.path().join(".todox").join("tasks.json");
    let expected_config = temp.path().join(".todox").join("config.json");

    let output = Command::new(bin_path())
        .env("TODOX_DISABLE_TUI", "1")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .output()
        .expect("run with fake home");

    assert!(output.status.success());
    assert!(expected.exists());
    assert!(expected_config.exists());
}
