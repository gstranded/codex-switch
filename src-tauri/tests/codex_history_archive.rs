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
            "CREATE TABLE threads (
                id TEXT PRIMARY KEY,
                rollout_path TEXT NOT NULL,
                model_provider TEXT NOT NULL,
                title TEXT
            )",
            [],
        )
        .expect("create source threads");
    source_conn
        .execute(
            "INSERT INTO threads (id, rollout_path, model_provider, title) VALUES (?1, ?2, ?3, ?4)",
            (
                "imported-session",
                r"C:\Users\source\.codex\sessions\2026\07\session-source.jsonl",
                "source-provider",
                "Imported chat",
            ),
        )
        .expect("insert source thread");
    drop(source_conn);

    let mut source_config = MultiAppConfig::default();
    let source_manager = source_config
        .get_manager_mut(&AppType::Codex)
        .expect("source codex manager");
    source_manager.providers.insert(
        "source-provider".to_string(),
        Provider::with_id(
            "source-provider".to_string(),
            "Source provider".to_string(),
            json!({
                "auth": { "OPENAI_API_KEY": "archive-test-key" },
                "config": "model_provider = \"source-provider\"\n[model_providers.source-provider]\nname = \"Source\"\nbase_url = \"https://source.example.test\"\n"
            }),
            None,
        ),
    );
    source_manager.current = "source-provider".to_string();
    let source_state = create_test_state_with_config(&source_config).expect("source app state");

    let archive = home.join("portable-codex-history.zip");
    let exported = export_codex_history_to_file(source_state.db.as_ref(), &archive)
        .expect("export history and providers");
    assert_eq!(exported.session_files, 1);
    assert_eq!(exported.state_databases, 1);
    assert_eq!(exported.providers, 1);
    assert!(exported.contains_secrets);
    drop(source_state);

    reset_test_fs();
    let target_state =
        create_test_state_with_config(&MultiAppConfig::default()).expect("target app state");
    let imported = import_codex_history_from_file(target_state.db.as_ref(), &archive)
        .expect("import history and providers");
    assert_eq!(imported.imported_session_files, 1);
    assert_eq!(imported.skipped_session_files, 0);
    assert_eq!(imported.imported_state_threads, 1);
    assert_eq!(imported.imported_providers, 1);
    assert_eq!(
        imported.restored_current_provider.as_deref(),
        Some("source-provider")
    );

    let restored_provider = target_state
        .db
        .get_provider_by_id("source-provider", "codex")
        .expect("read restored provider")
        .expect("restored provider exists");
    assert_eq!(
        restored_provider
            .settings_config
            .pointer("/auth/OPENAI_API_KEY")
            .and_then(|value| value.as_str()),
        Some("archive-test-key")
    );

    let imported_session_path = home
        .join(".codex")
        .join("sessions")
        .join("2026")
        .join("07")
        .join("session-source.jsonl");
    let imported_state = Connection::open(home.join(".codex").join("state_5.sqlite"))
        .expect("open imported state for portable path check");
    let rollout_path: String = imported_state
        .query_row(
            "SELECT rollout_path FROM threads WHERE id = ?1",
            ["imported-session"],
            |row| row.get(0),
        )
        .expect("read repaired rollout path");
    assert_eq!(
        rollout_path,
        imported_session_path.to_string_lossy().to_string()
    );
    drop(imported_state);

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
    let target_provider = manager
        .providers
        .get("target-provider")
        .expect("target provider");
    target_state
        .db
        .save_provider("codex", target_provider)
        .expect("save target provider");
    ProviderService::switch(&target_state, AppType::Codex, "target-provider")
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

    let repeated =
        import_codex_history_from_file(target_state.db.as_ref(), &archive).expect("repeat import");
    assert_eq!(repeated.imported_session_files, 0);
    assert_eq!(repeated.skipped_session_files, 1);
}
