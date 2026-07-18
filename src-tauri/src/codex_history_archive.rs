//! Portable Codex workspace archives.
//!
//! Version 2 includes conversation data plus Codex provider settings. Provider
//! settings can contain API keys or OAuth login material, so archives are secrets.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use rusqlite::{backup::Backup, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tempfile::tempdir;
use zip::write::SimpleFileOptions;
use zip::DateTime;

use crate::codex_config::{get_codex_config_dir, read_codex_config_text};
use crate::codex_state_db::codex_state_db_paths;
use crate::config::atomic_write;
use crate::database::Database;
use crate::error::AppError;
use crate::provider::Provider;

const ARCHIVE_FORMAT: &str = "codex-switch-chat-history";
const ARCHIVE_VERSION: u32 = 2;
const LEGACY_ARCHIVE_VERSION: u32 = 1;
const SESSION_INDEX_FILENAME: &str = "session_index.jsonl";
const PROVIDER_SETTINGS_PATH: &str = "providers/codex.json";
const MAX_ARCHIVE_ENTRIES: usize = 20_000;
const MAX_ARCHIVE_BYTES: u64 = 4 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexHistoryExportOutcome {
    pub file_path: String,
    pub session_files: usize,
    pub state_databases: usize,
    pub providers: usize,
    pub contains_secrets: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexHistoryImportOutcome {
    pub imported_session_files: usize,
    pub skipped_session_files: usize,
    pub imported_session_index_entries: usize,
    pub imported_state_threads: usize,
    pub imported_providers: usize,
    pub restored_current_provider: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexProviderArchive {
    providers: Vec<Provider>,
    current_provider_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArchiveManifest {
    format: String,
    version: u32,
    exported_at: String,
    files: Vec<ArchiveFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArchiveFile {
    path: String,
    sha256: String,
    bytes: u64,
}

pub fn export_codex_history_to_file(
    db: &Database,
    destination: &Path,
) -> Result<CodexHistoryExportOutcome, AppError> {
    let config_dir = get_codex_config_dir();
    let stage = tempdir().map_err(|e| AppError::IoContext {
        context: "Failed to create chat-history staging directory".to_string(),
        source: e,
    })?;
    let stage_root = stage.path().join("history");
    fs::create_dir_all(&stage_root).map_err(|e| AppError::io(&stage_root, e))?;

    let mut session_files = 0;
    stage_session_root(&config_dir, &stage_root, "sessions", &mut session_files)?;
    stage_session_root(
        &config_dir,
        &stage_root,
        "archived_sessions",
        &mut session_files,
    )?;

    let session_index = config_dir.join(SESSION_INDEX_FILENAME);
    if session_index.is_file() {
        fs::copy(&session_index, stage_root.join(SESSION_INDEX_FILENAME))
            .map_err(|e| AppError::io(&session_index, e))?;
    }

    let mut state_databases = 0;
    let config_text = read_codex_config_text().unwrap_or_default();
    for (index, source) in codex_state_db_paths(&config_dir, &config_text)
        .into_iter()
        .filter(|path| path.is_file())
        .enumerate()
    {
        let snapshot = stage_root.join("state").join(format!("{index}.sqlite"));
        snapshot_sqlite(&source, &snapshot)?;
        state_databases += 1;
    }

    let providers = stage_codex_providers(db, &stage_root)?;

    let files = archive_files_from_stage(&stage_root)?;
    if files.len() > MAX_ARCHIVE_ENTRIES {
        return Err(AppError::InvalidInput(format!(
            "Too many Codex history files to export: {} (limit {MAX_ARCHIVE_ENTRIES})",
            files.len()
        )));
    }

    let manifest = ArchiveManifest {
        format: ARCHIVE_FORMAT.to_string(),
        version: ARCHIVE_VERSION,
        exported_at: Utc::now().to_rfc3339(),
        files,
    };
    write_history_archive(&stage_root, destination, &manifest)?;

    Ok(CodexHistoryExportOutcome {
        file_path: destination.to_string_lossy().to_string(),
        session_files,
        state_databases,
        providers,
        contains_secrets: providers > 0,
    })
}

pub fn import_codex_history_from_file(
    db: &Database,
    source: &Path,
) -> Result<CodexHistoryImportOutcome, AppError> {
    let stage = extract_history_archive(source)?;
    let stage_root = stage.path().join("history");
    let config_dir = get_codex_config_dir();
    fs::create_dir_all(&config_dir).map_err(|e| AppError::io(&config_dir, e))?;

    let mut outcome = import_session_files(&stage_root, &config_dir)?;
    outcome.imported_session_index_entries = merge_session_index(&stage_root, &config_dir)
        .unwrap_or_else(|error| {
            outcome
                .warnings
                .push(format!("session_index_merge_failed:{error}"));
            0
        });

    let session_paths = collect_session_paths_by_id(&config_dir)?;
    match merge_state_databases(&stage_root, &config_dir, &session_paths) {
        Ok(imported_threads) => outcome.imported_state_threads = imported_threads,
        Err(error) => outcome
            .warnings
            .push(format!("state_database_merge_failed:{error}")),
    }

    let provider_archive = import_codex_providers(db, &stage_root)?;
    outcome.imported_providers = provider_archive.0;
    outcome.restored_current_provider = provider_archive.1;

    Ok(outcome)
}

fn stage_codex_providers(db: &Database, stage_root: &Path) -> Result<usize, AppError> {
    let providers = db
        .get_all_providers("codex")?
        .into_values()
        .collect::<Vec<_>>();
    let payload = CodexProviderArchive {
        current_provider_id: db.get_current_provider("codex")?,
        providers,
    };
    let destination = stage_root.join(PROVIDER_SETTINGS_PATH);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
    }
    let bytes =
        serde_json::to_vec_pretty(&payload).map_err(|source| AppError::JsonSerialize { source })?;
    atomic_write(&destination, &bytes)?;
    Ok(payload.providers.len())
}

fn import_codex_providers(
    db: &Database,
    stage_root: &Path,
) -> Result<(usize, Option<String>), AppError> {
    let source = stage_root.join(PROVIDER_SETTINGS_PATH);
    if !source.is_file() {
        return Ok((0, None));
    }
    let bytes = fs::read(&source).map_err(|e| AppError::io(&source, e))?;
    let payload: CodexProviderArchive = serde_json::from_slice(&bytes).map_err(|e| {
        AppError::InvalidInput(format!("Invalid Codex provider settings in archive: {e}"))
    })?;

    let mut imported_ids = HashSet::new();
    for provider in &payload.providers {
        if provider.id.trim().is_empty() {
            return Err(AppError::InvalidInput(
                "Imported Codex provider has an empty id".to_string(),
            ));
        }
        db.save_provider("codex", provider)?;
        imported_ids.insert(provider.id.clone());
    }

    let restored_current = payload
        .current_provider_id
        .filter(|id| imported_ids.contains(id));
    if let Some(current_id) = restored_current.as_deref() {
        db.set_current_provider("codex", current_id)?;
        crate::settings::set_current_provider(
            &crate::app_config::AppType::Codex,
            Some(current_id),
        )?;
    }
    Ok((payload.providers.len(), restored_current))
}

fn stage_session_root(
    config_dir: &Path,
    stage_root: &Path,
    root_name: &str,
    count: &mut usize,
) -> Result<(), AppError> {
    let source_root = config_dir.join(root_name);
    let mut files = Vec::new();
    collect_jsonl_files(&source_root, &mut files)?;
    for source in files {
        let relative = source
            .strip_prefix(&source_root)
            .map_err(|e| AppError::Message(format!("Failed to build session archive path: {e}")))?;
        let target = stage_root.join(root_name).join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
        }
        fs::copy(&source, &target).map_err(|e| AppError::io(&source, e))?;
        *count += 1;
    }
    Ok(())
}

fn collect_jsonl_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), AppError> {
    if !root.exists() {
        return Ok(());
    }
    let mut entries = fs::read_dir(root)
        .map_err(|e| AppError::io(root, e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::io(root, e))?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().map_err(|e| AppError::io(&path, e))?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_jsonl_files(&path, files)?;
        } else if file_type.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("jsonl")
        {
            files.push(path);
        }
    }
    Ok(())
}

fn snapshot_sqlite(source: &Path, target: &Path) -> Result<(), AppError> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
    }
    let source_connection = Connection::open_with_flags(
        source,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| AppError::Database(format!("Failed to open Codex state database: {e}")))?;
    source_connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|e| AppError::Database(format!("Failed to wait for Codex state database: {e}")))?;
    let mut target_connection = Connection::open(target)
        .map_err(|e| AppError::Database(format!("Failed to create state snapshot: {e}")))?;
    let backup = Backup::new(&source_connection, &mut target_connection)
        .map_err(|e| AppError::Database(format!("Failed to initialise state snapshot: {e}")))?;
    backup
        .run_to_completion(5, Duration::from_millis(25), None)
        .map_err(|e| AppError::Database(format!("Failed to write state snapshot: {e}")))?;
    Ok(())
}

fn archive_files_from_stage(stage_root: &Path) -> Result<Vec<ArchiveFile>, AppError> {
    let mut paths = Vec::new();
    collect_files(stage_root, &mut paths)?;
    let mut total_bytes = 0_u64;
    let mut files = Vec::with_capacity(paths.len());

    for path in paths {
        let metadata = fs::metadata(&path).map_err(|e| AppError::io(&path, e))?;
        total_bytes = total_bytes.saturating_add(metadata.len());
        if total_bytes > MAX_ARCHIVE_BYTES {
            return Err(AppError::InvalidInput(format!(
                "Codex history archive exceeds the {} GiB limit",
                MAX_ARCHIVE_BYTES / 1024 / 1024 / 1024
            )));
        }
        let relative = path
            .strip_prefix(stage_root)
            .map_err(|e| AppError::Message(format!("Failed to build history archive path: {e}")))?;
        files.push(ArchiveFile {
            path: archive_path(relative),
            sha256: sha256_file(&path)?,
            bytes: metadata.len(),
        });
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn collect_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), AppError> {
    if !root.exists() {
        return Ok(());
    }
    let mut entries = fs::read_dir(root)
        .map_err(|e| AppError::io(root, e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::io(root, e))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().map_err(|e| AppError::io(&path, e))?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_files(&path, files)?;
        } else if file_type.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn write_history_archive(
    stage_root: &Path,
    destination: &Path,
    manifest: &ArchiveManifest,
) -> Result<(), AppError> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
    }
    let temporary = destination.with_extension(format!(
        "{}.tmp-{}",
        destination
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("zip"),
        uuid::Uuid::new_v4()
    ));
    let file = fs::File::create(&temporary).map_err(|e| AppError::io(&temporary, e))?;
    let mut writer = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(DateTime::default());

    let manifest_bytes =
        serde_json::to_vec_pretty(manifest).map_err(|source| AppError::JsonSerialize { source })?;
    writer
        .start_file("manifest.json", options)
        .map_err(|e| AppError::Message(format!("Failed to create archive manifest: {e}")))?;
    writer
        .write_all(&manifest_bytes)
        .map_err(|e| AppError::io(&temporary, e))?;

    for entry in &manifest.files {
        let source = stage_root.join(Path::new(&entry.path));
        writer
            .start_file(&entry.path, options)
            .map_err(|e| AppError::Message(format!("Failed to create archive entry: {e}")))?;
        let mut input = fs::File::open(&source).map_err(|e| AppError::io(&source, e))?;
        std::io::copy(&mut input, &mut writer).map_err(|e| AppError::io(&temporary, e))?;
    }

    writer
        .finish()
        .map_err(|e| AppError::Message(format!("Failed to finalise chat-history archive: {e}")))?;
    if destination.exists() {
        fs::remove_file(destination).map_err(|e| AppError::io(destination, e))?;
    }
    fs::rename(&temporary, destination).map_err(|e| AppError::IoContext {
        context: format!(
            "Failed to move chat-history archive into place: {} -> {}",
            temporary.display(),
            destination.display()
        ),
        source: e,
    })?;
    Ok(())
}

fn extract_history_archive(source: &Path) -> Result<tempfile::TempDir, AppError> {
    let file = fs::File::open(source).map_err(|e| AppError::io(source, e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::InvalidInput(format!("Invalid chat-history archive: {e}")))?;
    if archive.len() > MAX_ARCHIVE_ENTRIES + 1 {
        return Err(AppError::InvalidInput(format!(
            "Chat-history archive has too many entries (limit {MAX_ARCHIVE_ENTRIES})"
        )));
    }

    let manifest = {
        let mut entry = archive
            .by_name("manifest.json")
            .map_err(|_| AppError::InvalidInput("Missing archive manifest".to_string()))?;
        if entry.size() > 4 * 1024 * 1024 {
            return Err(AppError::InvalidInput(
                "Archive manifest is too large".to_string(),
            ));
        }
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        entry
            .read_to_end(&mut bytes)
            .map_err(|e| AppError::Message(format!("Failed to read archive manifest: {e}")))?;
        serde_json::from_slice::<ArchiveManifest>(&bytes)
            .map_err(|e| AppError::InvalidInput(format!("Invalid archive manifest: {e}")))?
    };

    validate_manifest(&manifest, archive.len())?;
    let stage = tempdir().map_err(|e| AppError::IoContext {
        context: "Failed to create chat-history import directory".to_string(),
        source: e,
    })?;
    let stage_root = stage.path().join("history");
    fs::create_dir_all(&stage_root).map_err(|e| AppError::io(&stage_root, e))?;

    let mut total_bytes = 0_u64;
    for expected in &manifest.files {
        let mut entry = archive.by_name(&expected.path).map_err(|_| {
            AppError::InvalidInput(format!("Missing archive entry: {}", expected.path))
        })?;
        let Some(enclosed) = entry.enclosed_name() else {
            return Err(AppError::InvalidInput(format!(
                "Unsafe archive path: {}",
                expected.path
            )));
        };
        if archive_path(&enclosed) != expected.path
            || entry.is_dir()
            || entry.size() != expected.bytes
        {
            return Err(AppError::InvalidInput(format!(
                "Archive entry metadata does not match: {}",
                expected.path
            )));
        }
        total_bytes = total_bytes.saturating_add(entry.size());
        if total_bytes > MAX_ARCHIVE_BYTES {
            return Err(AppError::InvalidInput(format!(
                "Chat-history archive exceeds the {} GiB import limit",
                MAX_ARCHIVE_BYTES / 1024 / 1024 / 1024
            )));
        }
        let destination = stage_root.join(&enclosed);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
        }
        let mut output =
            fs::File::create(&destination).map_err(|e| AppError::io(&destination, e))?;
        std::io::copy(&mut entry, &mut output).map_err(|e| AppError::io(&destination, e))?;
        if sha256_file(&destination)? != expected.sha256 {
            return Err(AppError::InvalidInput(format!(
                "Archive checksum mismatch: {}",
                expected.path
            )));
        }
    }
    Ok(stage)
}

fn validate_manifest(manifest: &ArchiveManifest, archive_len: usize) -> Result<(), AppError> {
    if manifest.format != ARCHIVE_FORMAT
        || !matches!(manifest.version, LEGACY_ARCHIVE_VERSION | ARCHIVE_VERSION)
    {
        return Err(AppError::InvalidInput(
            "This file is not a compatible Codex Switch chat-history archive".to_string(),
        ));
    }
    if manifest.files.len() > MAX_ARCHIVE_ENTRIES || archive_len != manifest.files.len() + 1 {
        return Err(AppError::InvalidInput(
            "Unexpected chat-history archive entries".to_string(),
        ));
    }
    let mut names = HashSet::new();
    for file in &manifest.files {
        if !names.insert(file.path.clone())
            || !is_allowed_archive_path(&file.path, manifest.version)
        {
            return Err(AppError::InvalidInput(format!(
                "Unsupported archive entry: {}",
                file.path
            )));
        }
        if file.sha256.len() != 64 || !file.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(AppError::InvalidInput(format!(
                "Invalid archive checksum: {}",
                file.path
            )));
        }
    }
    Ok(())
}

fn is_allowed_archive_path(path: &str, archive_version: u32) -> bool {
    let parsed = Path::new(path);
    if parsed.is_absolute()
        || parsed.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return false;
    }
    path == SESSION_INDEX_FILENAME
        || (path.starts_with("sessions/") && path.ends_with(".jsonl"))
        || (path.starts_with("archived_sessions/") && path.ends_with(".jsonl"))
        || (path.starts_with("state/") && path.ends_with(".sqlite"))
        || (archive_version >= 2 && path == PROVIDER_SETTINGS_PATH)
}

fn import_session_files(
    stage_root: &Path,
    config_dir: &Path,
) -> Result<CodexHistoryImportOutcome, AppError> {
    let mut existing_ids = collect_existing_session_ids(config_dir)?;
    let mut imported = CodexHistoryImportOutcome::default();

    for root_name in ["sessions", "archived_sessions"] {
        let source_root = stage_root.join(root_name);
        let mut files = Vec::new();
        collect_jsonl_files(&source_root, &mut files)?;
        for source in files {
            let session_id = session_id_from_jsonl(&source)?.ok_or_else(|| {
                AppError::InvalidInput(format!(
                    "Imported session has no session_meta id: {}",
                    source.display()
                ))
            })?;
            if !existing_ids.insert(session_id) {
                imported.skipped_session_files += 1;
                continue;
            }

            let relative = source.strip_prefix(&source_root).map_err(|e| {
                AppError::Message(format!("Failed to resolve imported session path: {e}"))
            })?;
            let destination_root = config_dir.join(root_name);
            let destination = unique_session_destination(
                &destination_root.join(relative),
                &sha256_file(&source)?,
            );
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
            }
            fs::copy(&source, &destination).map_err(|e| AppError::io(&source, e))?;
            imported.imported_session_files += 1;
        }
    }
    Ok(imported)
}

fn collect_existing_session_ids(config_dir: &Path) -> Result<HashSet<String>, AppError> {
    Ok(collect_session_paths_by_id(config_dir)?
        .into_keys()
        .collect())
}

fn collect_session_paths_by_id(config_dir: &Path) -> Result<HashMap<String, PathBuf>, AppError> {
    let mut sessions = HashMap::new();
    for root_name in ["sessions", "archived_sessions"] {
        let mut files = Vec::new();
        collect_jsonl_files(&config_dir.join(root_name), &mut files)?;
        for file in files {
            if let Some(id) = session_id_from_jsonl(&file)? {
                sessions.entry(id).or_insert(file);
            }
        }
    }
    Ok(sessions)
}

fn session_id_from_jsonl(path: &Path) -> Result<Option<String>, AppError> {
    let file = fs::File::open(path).map_err(|e| AppError::io(path, e))?;
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|e| AppError::io(path, e))?;
        if !line.contains("\"session_meta\"") {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(&line).map_err(|e| {
            AppError::InvalidInput(format!("Invalid session JSONL {}: {e}", path.display()))
        })?;
        if value.get("type").and_then(serde_json::Value::as_str) != Some("session_meta") {
            continue;
        }
        return Ok(value
            .get("payload")
            .and_then(|payload| payload.get("id"))
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .map(str::to_string));
    }
    Ok(None)
}

fn unique_session_destination(destination: &Path, checksum: &str) -> PathBuf {
    if !destination.exists() {
        return destination.to_path_buf();
    }
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let stem = destination
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("session");
    let suffix = checksum.get(..12).unwrap_or("imported");
    let mut candidate = parent.join(format!("{stem}-imported-{suffix}.jsonl"));
    let mut index = 2;
    while candidate.exists() {
        candidate = parent.join(format!("{stem}-imported-{suffix}-{index}.jsonl"));
        index += 1;
    }
    candidate
}

fn merge_session_index(stage_root: &Path, config_dir: &Path) -> Result<usize, AppError> {
    let imported_index = stage_root.join(SESSION_INDEX_FILENAME);
    if !imported_index.is_file() {
        return Ok(0);
    }
    let destination = config_dir.join(SESSION_INDEX_FILENAME);
    let mut existing_ids = BTreeSet::new();
    let mut merged = if destination.is_file() {
        fs::read_to_string(&destination).map_err(|e| AppError::io(&destination, e))?
    } else {
        String::new()
    };
    for line in merged.lines() {
        if let Some(id) = session_index_id(line) {
            existing_ids.insert(id);
        }
    }
    if !merged.is_empty() && !merged.ends_with('\n') {
        merged.push('\n');
    }

    let mut imported = 0;
    for line in BufReader::new(
        fs::File::open(&imported_index).map_err(|e| AppError::io(&imported_index, e))?,
    )
    .lines()
    {
        let line = line.map_err(|e| AppError::io(&imported_index, e))?;
        let Some(id) = session_index_id(&line) else {
            continue;
        };
        if existing_ids.insert(id) {
            merged.push_str(&line);
            merged.push('\n');
            imported += 1;
        }
    }
    if imported > 0 {
        atomic_write(&destination, merged.as_bytes())?;
    }
    Ok(imported)
}

fn session_index_id(line: &str) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(line).ok()?;
    let id = value.get("id")?.as_str()?.trim();
    (!id.is_empty()).then(|| id.to_string())
}

fn merge_state_databases(
    stage_root: &Path,
    config_dir: &Path,
    session_paths: &HashMap<String, PathBuf>,
) -> Result<usize, AppError> {
    let source_root = stage_root.join("state");
    let mut snapshots = Vec::new();
    collect_files(&source_root, &mut snapshots)?;
    snapshots
        .retain(|path| path.extension().and_then(|extension| extension.to_str()) == Some("sqlite"));
    if snapshots.is_empty() {
        return Ok(0);
    }

    let config_text = read_codex_config_text().unwrap_or_default();
    let targets = codex_state_db_paths(config_dir, &config_text);
    let mut imported_threads = 0;
    for target in targets {
        if !target.exists() {
            if let Some(first_snapshot) = snapshots.first() {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
                }
                fs::copy(first_snapshot, &target).map_err(|e| AppError::io(first_snapshot, e))?;
                imported_threads += count_threads(first_snapshot)?;
                repair_thread_rollout_paths(&target, session_paths)?;
            }
            continue;
        }
        for snapshot in &snapshots {
            imported_threads += merge_threads_from_snapshot(snapshot, &target, session_paths)?;
        }
    }
    Ok(imported_threads)
}

fn count_threads(path: &Path) -> Result<usize, AppError> {
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| AppError::Database(format!("Failed to open state snapshot: {e}")))?;
    if table_columns(&connection)?.is_empty() {
        return Ok(0);
    }
    let count = connection
        .query_row("SELECT COUNT(*) FROM threads", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|e| AppError::Database(format!("Failed to count imported threads: {e}")))?;
    Ok(usize::try_from(count).unwrap_or(0))
}

fn merge_threads_from_snapshot(
    source: &Path,
    target: &Path,
    session_paths: &HashMap<String, PathBuf>,
) -> Result<usize, AppError> {
    let source_connection = Connection::open_with_flags(
        source,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| AppError::Database(format!("Failed to open imported state snapshot: {e}")))?;
    let source_columns = table_columns(&source_connection)?;
    if source_columns.is_empty() {
        return Ok(0);
    }
    drop(source_connection);

    let target_connection = Connection::open(target)
        .map_err(|e| AppError::Database(format!("Failed to open local state database: {e}")))?;
    target_connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|e| AppError::Database(format!("Failed to wait for local state database: {e}")))?;
    let target_columns = table_columns(&target_connection)?;
    if target_columns.is_empty() || !target_columns.iter().any(|column| column == "id") {
        return Ok(0);
    }

    let source_set: HashSet<_> = source_columns.iter().cloned().collect();
    let common_columns: Vec<String> = target_columns
        .into_iter()
        .filter(|column| source_set.contains(column))
        .collect();
    if !common_columns.iter().any(|column| column == "id") {
        return Ok(0);
    }

    let before = target_connection
        .query_row("SELECT COUNT(*) FROM threads", [], |row| {
            row.get::<_, usize>(0)
        })
        .map_err(|e| AppError::Database(format!("Failed to count local threads: {e}")))?;
    target_connection
        .execute(
            "ATTACH DATABASE ?1 AS imported_history",
            [source.to_string_lossy().to_string()],
        )
        .map_err(|e| {
            AppError::Database(format!("Failed to attach imported state snapshot: {e}"))
        })?;
    let columns = common_columns
        .iter()
        .map(|column| quote_identifier(column))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "INSERT OR IGNORE INTO threads ({columns}) SELECT {columns} FROM imported_history.threads"
    );
    let merge_result = target_connection.execute(&sql, []);
    let detach_result = target_connection.execute_batch("DETACH DATABASE imported_history");
    merge_result
        .map_err(|e| AppError::Database(format!("Failed to merge imported threads: {e}")))?;
    detach_result.map_err(|e| {
        AppError::Database(format!("Failed to detach imported state snapshot: {e}"))
    })?;
    let after = target_connection
        .query_row("SELECT COUNT(*) FROM threads", [], |row| {
            row.get::<_, usize>(0)
        })
        .map_err(|e| AppError::Database(format!("Failed to count merged threads: {e}")))?;
    drop(target_connection);
    repair_thread_rollout_paths(target, session_paths)?;
    Ok(after.saturating_sub(before))
}

fn repair_thread_rollout_paths(
    target: &Path,
    session_paths: &HashMap<String, PathBuf>,
) -> Result<usize, AppError> {
    if session_paths.is_empty() {
        return Ok(0);
    }
    let mut connection = Connection::open(target)
        .map_err(|e| AppError::Database(format!("Failed to repair imported thread paths: {e}")))?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|e| {
            AppError::Database(format!("Failed to wait for imported state database: {e}"))
        })?;
    let columns = table_columns(&connection)?;
    if !columns.iter().any(|column| column == "rollout_path") {
        return Ok(0);
    }

    let transaction = connection
        .transaction()
        .map_err(|e| AppError::Database(format!("Failed to start thread path repair: {e}")))?;
    let mut repaired = 0;
    for (session_id, path) in session_paths {
        repaired += transaction
            .execute(
                "UPDATE threads SET rollout_path = ?1 WHERE id = ?2 AND rollout_path <> ?1",
                (path.to_string_lossy().to_string(), session_id),
            )
            .map_err(|e| {
                AppError::Database(format!("Failed to rewrite imported thread path: {e}"))
            })?;
    }
    transaction
        .commit()
        .map_err(|e| AppError::Database(format!("Failed to commit thread path repair: {e}")))?;
    Ok(repaired)
}

fn table_columns(connection: &Connection) -> Result<Vec<String>, AppError> {
    let mut statement = connection
        .prepare("PRAGMA table_info(threads)")
        .map_err(|e| AppError::Database(format!("Failed to inspect thread schema: {e}")))?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| AppError::Database(format!("Failed to read thread schema: {e}")))?;
    Ok(rows.flatten().collect())
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn sha256_file(path: &Path) -> Result<String, AppError> {
    let mut file = fs::File::open(path).map_err(|e| AppError::io(path, e))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|e| AppError::io(path, e))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn archive_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_versioned_workspace_archive_paths() {
        assert!(is_allowed_archive_path(
            "sessions/2026/06/01/session.jsonl",
            2
        ));
        assert!(is_allowed_archive_path("archived_sessions/one.jsonl", 2));
        assert!(is_allowed_archive_path("state/0.sqlite", 2));
        assert!(is_allowed_archive_path("session_index.jsonl", 2));
        assert!(is_allowed_archive_path(PROVIDER_SETTINGS_PATH, 2));
        assert!(!is_allowed_archive_path(PROVIDER_SETTINGS_PATH, 1));
        assert!(!is_allowed_archive_path("../auth.json", 2));
        assert!(!is_allowed_archive_path("config.toml", 2));
        assert!(!is_allowed_archive_path("sessions/../auth.json", 2));
    }

    #[test]
    fn session_index_id_reads_only_valid_ids() {
        assert_eq!(
            session_index_id(r#"{"id":"thread-1","thread_name":"Name"}"#).as_deref(),
            Some("thread-1")
        );
        assert_eq!(session_index_id("not json"), None);
        assert_eq!(session_index_id(r#"{"thread_name":"Name"}"#), None);
    }
}
