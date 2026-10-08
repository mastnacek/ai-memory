//! Session reader tests (fixtures written per test).

use super::*;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn write_session(dir: &Path, sub: &str, name: &str, lines: &[&str]) -> PathBuf {
    let path = dir.join(sub);
    std::fs::create_dir_all(&path).unwrap();
    let file = path.join(name);
    std::fs::write(&file, lines.join("\n")).unwrap();
    file
}

const PI_HEADER: &str = r#"{"type":"session","id":"019e3b13-12e9-753f-86ac-6b9bb4638e07","timestamp":"2026-05-18T12:32:46.313Z","cwd":"D:\\proj\\demo"}"#;
const PI_USER: &str = r#"{"type":"message","message":{"role":"user","content":[{"type":"text","text":"jak na to?"}]}}"#;
const PI_ASSISTANT: &str = r#"{"type":"message","message":{"role":"assistant","content":[{"type":"text","text":"answer"}]}}"#;
const PI_TOOL: &str = r#"{"type":"message","message":{"role":"toolResult","content":[{"type":"text","text":"out"}]}}"#;

#[test]
fn test_pi_session_scans_to_a_record() {
    let dir = tempdir().unwrap();
    write_session(
        dir.path(),
        "--D--proj-demo--",
        "2026-05-18T12-32-46-313Z_019e3b13.jsonl",
        &[PI_HEADER, PI_USER, PI_ASSISTANT, PI_TOOL],
    );

    let records = scan_at(dir.path(), SessionSource::PiAgent).unwrap();
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.project, r"D:\proj\demo");
    assert_eq!(record.session_id, "019e3b13-12e9-753f-86ac-6b9bb4638e07");
    assert_eq!(record.user_messages, 1);
    assert_eq!(record.assistant_messages, 1);
    assert_eq!(record.substantive_user_turns, 1);
}

#[test]
fn test_injected_user_turns_do_not_count_as_substantive() {
    let dir = tempdir().unwrap();
    write_session(
        dir.path(),
        "p",
        "2026-05-18T12-32-46-313Z_s.jsonl",
        &[
            PI_HEADER,
            r#"{"type":"message","message":{"role":"user","content":"<local-command-caveat>wrapped</local-command-caveat>"}}"#,
            r#"{"type":"message","message":{"role":"user","content":"   "}}"#,
            PI_USER,
        ],
    );

    let records = scan_at(dir.path(), SessionSource::PiAgent).unwrap();
    assert_eq!(records[0].user_messages, 3, "all user turns counted");
    assert_eq!(records[0].substantive_user_turns, 1, "only the real one");
}

#[test]
fn test_claude_session_takes_title_and_session_id() {
    let dir = tempdir().unwrap();
    write_session(
        dir.path(),
        "D--01-programovani-demo",
        "f120c3aa-c763-4465-8dc6-de281f5a244a.jsonl",
        &[
            r#"{"sessionId":"f120c3aa-c763-4465-8dc6-de281f5a244a","type":"last-prompt"}"#,
            r#"{"type":"ai-title","aiTitle":"Vytvořit asistenta","sessionId":"f120c3aa"}"#,
            r#"{"cwd":"D:\\01_programovani\\demo","timestamp":"2026-08-13T08:09:30.643Z","type":"user","message":{"role":"user","content":"hello there"}}"#,
            r#"{"type":"assistant","message":{"role":"assistant","content":"hi"}}"#,
        ],
    );

    let records = scan_at(dir.path(), SessionSource::ClaudeCode).unwrap();
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.title.as_deref(), Some("Vytvořit asistenta"));
    assert_eq!(record.session_id, "f120c3aa-c763-4465-8dc6-de281f5a244a");
    assert_eq!(record.project, r"D:\01_programovani\demo");
    assert_eq!(record.user_messages, 1);
    assert_eq!(record.assistant_messages, 1);
}

#[test]
fn test_detail_returns_substantive_user_messages_in_order() {
    let dir = tempdir().unwrap();
    let file = write_session(
        dir.path(),
        "p",
        "2026-05-18T12-32-46-313Z_s.jsonl",
        &[
            PI_HEADER,
            PI_USER,
            r#"{"type":"message","message":{"role":"user","content":[{"type":"text","text":"<system>noise</system>"}]}}"#,
            PI_ASSISTANT,
            PI_TOOL,
        ],
    );

    let detail = read_detail(&file, SessionSource::PiAgent).unwrap();
    assert_eq!(detail.user_messages, vec!["jak na to?".to_string()]);
    assert_eq!(detail.assistant_messages, 1);
    assert_eq!(detail.tool_results, 1);
}

#[test]
fn test_truncated_and_unparsable_files_still_list() {
    let dir = tempdir().unwrap();
    write_session(
        dir.path(),
        "p",
        "2026-05-18T12-32-46-313Z_broken.jsonl",
        &[PI_HEADER, "{not json"],
    );

    let records = scan_at(dir.path(), SessionSource::PiAgent).unwrap();
    assert_eq!(records.len(), 1, "a bad line must not drop the session");
    assert_eq!(records[0].session_id, "019e3b13-12e9-753f-86ac-6b9bb4638e07");
}

#[test]
fn test_missing_root_is_not_an_error() {
    assert_eq!(
        scan_at(Path::new("/definitely/not/here"), SessionSource::PiAgent).unwrap(),
        Vec::new()
    );
}

#[test]
fn test_tool_results_are_counted_in_detail() {
    let dir = tempdir().unwrap();
    let file = write_session(
        dir.path(),
        "p",
        "2026-05-18T12-32-46-313Z_s.jsonl",
        &[PI_HEADER, PI_USER, PI_TOOL, PI_TOOL],
    );

    let detail = read_detail(&file, SessionSource::PiAgent).unwrap();
    assert_eq!(detail.tool_results, 2);
}