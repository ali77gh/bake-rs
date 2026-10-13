use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

const BAKEFILE: &str = r#"
tasks:
  - name: ok
    commands:
      - echo works

  - name: fail
    commands:
      - exit 3

  - name: fail_mid
    commands:
      - echo before
      - exit 5
      - echo SHOULD_NOT_RUN_MARKER

  - name: handled
    commands:
      - exit 2
    on_error: "@this.noop"

  - name: end_only
    commands:
      - exit 4
    on_end: "@this.noop"

  - name: noop
    commands:
      - echo noop
"#;

/// runs `bake <task>` in a fresh temp dir containing BAKEFILE
fn run_task(task: &str) -> Output {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("bake-rs-exit-code-{}-{}", task, std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("bakefile.yaml"), BAKEFILE).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_bake"))
        .arg(task)
        .current_dir(&dir)
        .output()
        .unwrap();

    let _ = fs::remove_dir_all(&dir);
    output
}

#[test]
fn successful_task_exits_zero() {
    assert_eq!(run_task("ok").status.code(), Some(0));
}

#[test]
fn failing_task_propagates_exit_code() {
    assert_eq!(run_task("fail").status.code(), Some(3));
}

#[test]
fn failure_stops_remaining_commands_and_propagates_code() {
    let out = run_task("fail_mid");
    assert_eq!(out.status.code(), Some(5));

    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!all.contains("SHOULD_NOT_RUN_MARKER"));
}

#[test]
fn on_error_handler_counts_as_handled() {
    assert_eq!(run_task("handled").status.code(), Some(0));
}

#[test]
fn on_end_alone_does_not_swallow_failure() {
    let out = run_task("end_only");
    assert_eq!(
        out.status.code(),
        Some(4),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
