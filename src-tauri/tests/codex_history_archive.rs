use std::fs;

use codex_switch_lib::{
    export_codex_history_to_file, import_codex_history_from_file, AppType, MultiAppConfig,
    Provider, ProviderService,
};
use rusqlite::Connection;
use serde_json::json;

#[path = "support.rs"]
mod support;
use support::{create_test_state_with_config, ensure_test_home, reset_test_fs, test_mutex};

#[test]
fn imported_codex_history_is_merged_and_follows_the_active_provider() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let home = ensure_test_home().to_path_buf();
    let codex_dir = home.join(".codex");
    let session_path = codex_dir
        .join("sessions")
        .join("2026")
        .join("07")
        .join("session-source.jsonl");
    fs::create_dir_all(session_path.parent().expect("session parent")).expect("create sessions");
    fs::write(
        &session_path,
        concat!(
            "{\"timestamp\":\"2026-07-01T00:00:00Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"imported-session\",\"cwd\":\"/tmp/project\",\"model_provider\":\"source-provider\"}}\n",
            "{\"timestamp\":\"2026-07-01T00:00:01Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":\"restore this chat\"}}\n"
        ),
    )
    .expect("write source session");
    fs::write(
        codex_dir.join("session_index.jsonl"),
        "{\"id\":\"imported-session\",\"thread_name\":\"Imported chat\"}\n",
    )
    .expect("write session index");

    let source_state_db = codex_dir.join("state_5.sqlite");
    let source_conn = Connection::open(&source_state_db).expect("create source state db");
    source_conn
        .execute(
            "CREATE TABLE threads (id TEXT PRIMARY KEY, model_provider TEXT NOT NULL, title TEXT)",
            [],
        )
        .expect("create source threads");
    source_conn
        .execute(
            "INSERT INTO threads (id, model_provider, title) VALUES (?1, ?2, ?3)",
            ("imported-session", "source-provider", "Imported chat"),
        )
        .expect("insert source thread");
    drop(source_conn);

    let archive = home.join("portable-codex-history.zip");
    let exported = export_codex_history_to_file(&archive).expect("export history");
    assert_eq!(exported.session_files, 1);
    assert_eq!(exported.state_databases, 1);

    reset_test_fs();
    let imported = import_codex_history_from_file(&archive).expect("import history");
    assert_eq!(imported.imported_session_files, 1);
    assert_eq!(imported.skipped_session_files, 0);
    assert_eq!(imported.imported_state_threads, 1);

    let mut config = MultiAppConfig::default();
    let manager = config
        .get_manager_mut(&AppType::Codex)
        .expect("codex manager");
    manager.providers.insert(
        "target-provider".to_string(),
        Provider::with_id(
            "target-provider".to_string(),
            "Target provider".to_string(),
            json!({
                "auth": { "OPENAI_API_KEY": "test-key" },
                "config": "model_provider = \"target-provider\"\n[model_providers.target-provider]\nname = \"Target\"\nbase_url = \"https://example.test\"\n"
            }),
            None,
        ),
    );
    manager.current = "target-provider".to_string();
    let state = create_test_state_with_config(&config).expect("create test state");
    ProviderService::switch(&state, AppType::Codex, "target-provider")
        .expect("switch target provider and sync imported history");

    let imported_session = fs::read_to_string(
        home.join(".codex")
            .join("sessions")
            .join("2026")
            .join("07")
            .join("session-source.jsonl"),
    )
    .expect("read imported session");
    assert!(imported_session.contains("\"model_provider\":\"target-provider\""));

    let target_conn = Connection::open(home.join(".codex").join("state_5.sqlite"))
        .expect("open imported state db");
    let provider: String = target_conn
        .query_row(
            "SELECT model_provider FROM threads WHERE id = ?1",
            ["imported-session"],
            |row| row.get(0),
        )
        .expect("read imported thread provider");
    assert_eq!(provider, "target-provider");

    let repeated = import_codex_history_from_file(&archive).expect("repeat import");
    assert_eq!(repeated.imported_session_files, 0);
    assert_eq!(repeated.skipped_session_files, 1);
}
