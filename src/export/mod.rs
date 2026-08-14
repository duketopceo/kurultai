//! Offline `.kurultai` pack — export / import a setup for multi-device handoff (#80 thin slice).

use crate::config::{config_path, ensure_storage_parent, expand_path};
use crate::error::{KurultaiError, Result};
use crate::store::migrations::CURRENT_SCHEMA_VERSION;
use crate::store::{SearchFilter, SqliteVecStore, Store};
use crate::types::Config;
use chrono::Utc;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};

/// Pack format version (bump when archive layout changes incompatibly).
pub const PACK_FORMAT_VERSION: u32 = 1;

const MANIFEST_NAME: &str = "manifest.json";
const CONFIG_NAME: &str = "config.toml";
const STORE_NAME: &str = "store.db";
/// Soft ceiling on decompressed pack size (gzip-bomb / runaway unpack guard).
const MAX_PACK_EXPANDED_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// On-disk manifest embedded in a `.kurultai` pack, recording format/schema
/// compatibility and provenance for the exported store.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackManifest {
    pub format_version: u32,
    pub schema_version: i32,
    pub embed_dim: usize,
    pub atom_count: u64,
    pub kurultai_version: String,
    pub created_at: String,
}

/// Result of a successful `export_pack` call.
#[derive(Debug, Clone)]
pub struct ExportReport {
    pub path: PathBuf,
    pub atom_count: u64,
    pub embed_dim: usize,
}

/// Result of a successful `import_pack` call.
#[derive(Debug, Clone)]
pub struct ImportReport {
    pub mode: &'static str,
    pub atoms_upserted: u64,
    pub vectors_copied: u64,
    pub vectors_skipped: bool,
    pub storage_path: PathBuf,
}

fn default_export_path() -> PathBuf {
    let stamp = Utc::now().format("%Y%m%d-%H%M%S");
    PathBuf::from(format!("kurultai-export-{stamp}.kurultai"))
}

/// Create (or truncate) a file with mode `0o600` before writing sensitive content.
fn create_private_file(path: &Path) -> Result<File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .map_err(|e| {
                KurultaiError::Io(std::io::Error::new(
                    e.kind(),
                    format!("{}: {e}", path.display()),
                ))
            })
    }
    #[cfg(not(unix))]
    {
        File::create(path).map_err(|e| {
            KurultaiError::Io(std::io::Error::new(
                e.kind(),
                format!("{}: {e}", path.display()),
            ))
        })
    }
}

fn copy_private(src: &Path, dest: &Path) -> Result<()> {
    let mut reader = File::open(src).map_err(|e| {
        KurultaiError::Io(std::io::Error::new(
            e.kind(),
            format!("{}: {e}", src.display()),
        ))
    })?;
    let mut writer = create_private_file(dest)?;
    std::io::copy(&mut reader, &mut writer).map_err(|e| {
        KurultaiError::Store(format!("copy {} → {}: {e}", src.display(), dest.display()))
    })?;
    Ok(())
}

/// Export the current config + store into a `.kurultai` gzip tar pack.
pub fn export_pack(
    config: &Config,
    config_file: &Path,
    out: Option<&Path>,
) -> Result<ExportReport> {
    let storage = expand_path(&config.storage_path)?;
    if !storage.exists() {
        return Err(KurultaiError::config(format!(
            "storage not found at {} — run `kurultai index` first",
            storage.display()
        )));
    }

    let store = SqliteVecStore::open(storage.clone(), config.embed_dim)?;
    let atom_count = store.count_sync()?;
    let out_path = out
        .map(Path::to_path_buf)
        .unwrap_or_else(default_export_path);

    let tmp =
        tempfile::tempdir().map_err(|e| KurultaiError::Store(format!("export temp dir: {e}")))?;
    let db_snap = tmp.path().join(STORE_NAME);
    store.backup_to_path(&db_snap)?;

    let manifest = PackManifest {
        format_version: PACK_FORMAT_VERSION,
        schema_version: CURRENT_SCHEMA_VERSION,
        embed_dim: config.embed_dim,
        atom_count,
        kurultai_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: Utc::now().to_rfc3339(),
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|e| KurultaiError::Store(format!("manifest serialize: {e}")))?;

    let config_bytes = if config_file.exists() {
        fs::read(config_file).map_err(|e| {
            KurultaiError::config(format!("read config {}: {e}", config_file.display()))
        })?
    } else {
        // Synthesize a minimal portable config matching the live embed settings.
        format!(
            r#"# Exported by kurultai export - remap [sources.*.root_path] on the destination.
environment = "dev"

[storage]
# path omitted - destination uses env-default ~/.local/share/kurultai/.../store.db

[embed]
model = "{model}"
dimension = {dim}

[runtime]
poll_interval_secs = 300
"#,
            model = config.embed_model,
            dim = config.embed_dim,
        )
        .into_bytes()
    };

    // Redact accidental inline secrets if a stale config ever had them.
    let config_text = String::from_utf8_lossy(&config_bytes);
    let config_safe = redact_secret_keys(&config_text);

    write_kurultai_archive(&out_path, &manifest_bytes, config_safe.as_bytes(), &db_snap)?;

    Ok(ExportReport {
        path: out_path,
        atom_count,
        embed_dim: config.embed_dim,
    })
}

fn redact_secret_keys(toml_text: &str) -> String {
    let mut out = String::with_capacity(toml_text.len());
    for line in toml_text.lines() {
        let trimmed = line.trim_start();
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("api_key")
            || lower.starts_with("openai_api_key")
            || lower.starts_with("openrouter_api_key")
            || lower.starts_with("kurultai_api_key")
        {
            out.push_str(
                "# redacted by kurultai export — set keys via environment on destination\n",
            );
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn write_kurultai_archive(
    out: &Path,
    manifest: &[u8],
    config: &[u8],
    store_db: &Path,
) -> Result<()> {
    if let Some(parent) = out.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let file = create_private_file(out)?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(enc);

    append_bytes(&mut builder, MANIFEST_NAME, manifest)?;
    append_bytes(&mut builder, CONFIG_NAME, config)?;
    builder
        .append_path_with_name(store_db, STORE_NAME)
        .map_err(|e| KurultaiError::Store(format!("tar append store: {e}")))?;

    let enc = builder
        .into_inner()
        .map_err(|e| KurultaiError::Store(format!("tar finish: {e}")))?;
    enc.finish()
        .map_err(|e| KurultaiError::Store(format!("gzip finish: {e}")))?;
    Ok(())
}

fn append_bytes<W: Write>(builder: &mut Builder<W>, name: &str, bytes: &[u8]) -> Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, name, bytes)
        .map_err(|e| KurultaiError::Store(format!("tar append {name}: {e}")))?;
    Ok(())
}

#[derive(Debug)]
struct UnpackedPack {
    dir: tempfile::TempDir,
    manifest: PackManifest,
}

fn unpack_kurultai(path: &Path) -> Result<UnpackedPack> {
    let file = File::open(path)
        .map_err(|e| KurultaiError::config(format!("open pack {}: {e}", path.display())))?;
    let dec = GzDecoder::new(file);
    let limited = dec.take(MAX_PACK_EXPANDED_BYTES);
    let mut archive = Archive::new(limited);
    let dir =
        tempfile::tempdir().map_err(|e| KurultaiError::Store(format!("import temp dir: {e}")))?;
    archive.unpack(dir.path()).map_err(|e| {
        KurultaiError::Store(format!(
            "unpack {}: {e} (packs larger than {MAX_PACK_EXPANDED_BYTES} expanded bytes are rejected)",
            path.display()
        ))
    })?;

    let manifest_path = dir.path().join(MANIFEST_NAME);
    let store_path = dir.path().join(STORE_NAME);
    if !manifest_path.exists() || !store_path.exists() {
        return Err(KurultaiError::config(
            "invalid .kurultai pack — missing manifest.json or store.db",
        ));
    }
    let raw = fs::read_to_string(&manifest_path)
        .map_err(|e| KurultaiError::Store(format!("read manifest: {e}")))?;
    let manifest: PackManifest = serde_json::from_str(&raw)
        .map_err(|e| KurultaiError::config(format!("manifest parse: {e}")))?;
    if manifest.format_version != PACK_FORMAT_VERSION {
        return Err(KurultaiError::config(format!(
            "unsupported pack format_version {} (this CLI supports {PACK_FORMAT_VERSION})",
            manifest.format_version
        )));
    }
    if manifest.schema_version > CURRENT_SCHEMA_VERSION {
        return Err(KurultaiError::config(format!(
            "pack schema_version {} is newer than this CLI supports ({CURRENT_SCHEMA_VERSION}) — upgrade kurultai before importing",
            manifest.schema_version
        )));
    }
    Ok(UnpackedPack { dir, manifest })
}

/// Import modes for an existing destination store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    /// Install pack store.db into destination (empty or --force).
    Replace { force: bool },
    /// Upsert atoms from pack into the current store.
    Combine,
}

/// Import a `.kurultai` pack into the destination resolved by `config`.
///
/// When `write_config_if_missing` is set, `dest_config_path` is where the pack's
/// `config.toml` is written if that path does not already exist.
pub async fn import_pack(
    config: &Config,
    pack_path: &Path,
    mode: ImportMode,
    write_config_if_missing: bool,
    dest_config_path: &Path,
) -> Result<ImportReport> {
    let unpacked = unpack_kurultai(pack_path)?;
    let pack_store_path = unpacked.dir.path().join(STORE_NAME);
    let pack_config_path = unpacked.dir.path().join(CONFIG_NAME);
    let dest = expand_path(&config.storage_path)?;
    ensure_storage_parent(&dest)?;

    match mode {
        ImportMode::Replace { force } => {
            if dest.exists() {
                let meta = fs::metadata(&dest)?;
                if meta.len() > 0 && !force {
                    return Err(KurultaiError::config(format!(
                        "storage already exists at {} ({} bytes). Use --force to replace or --combine to merge atoms.",
                        dest.display(),
                        meta.len()
                    )));
                }
            }
            copy_private(&pack_store_path, &dest)?;
            maybe_write_config(&pack_config_path, dest_config_path, write_config_if_missing)?;
            Ok(ImportReport {
                mode: if force { "replace-force" } else { "replace" },
                atoms_upserted: unpacked.manifest.atom_count,
                vectors_copied: unpacked.manifest.atom_count,
                vectors_skipped: false,
                storage_path: dest,
            })
        }
        ImportMode::Combine => {
            let src = SqliteVecStore::open(pack_store_path, unpacked.manifest.embed_dim)?;
            let dest_store = SqliteVecStore::open(dest.clone(), config.embed_dim)?;
            let skip_vectors = unpacked.manifest.embed_dim != config.embed_dim;
            if skip_vectors {
                tracing::warn!(
                    pack_dim = unpacked.manifest.embed_dim,
                    dest_dim = config.embed_dim,
                    "embed_dim mismatch — combining atoms without vectors (FTS still works)"
                );
            }

            let mut upserted = 0u64;
            let mut vectors = 0u64;
            let mut after_id: Option<String> = None;
            const PAGE: usize = 200;
            loop {
                let mut page = src.list_atoms_page_sync(
                    after_id.as_deref(),
                    PAGE,
                    SearchFilter {
                        trusted_only: false,
                        namespace: None,
                    },
                    !skip_vectors,
                )?;
                if page.is_empty() {
                    break;
                }
                let n = page.len();
                let last_id = page.last().map(|a| a.id.clone());
                if !skip_vectors {
                    for atom in &mut page {
                        if let Some(emb) = atom.embedding.as_ref() {
                            if emb.len() == config.embed_dim {
                                vectors += 1;
                            } else {
                                atom.embedding = None;
                            }
                        }
                    }
                }
                dest_store.upsert_batch(&page).await?;
                upserted += n as u64;
                after_id = last_id;
            }

            maybe_write_config(&pack_config_path, dest_config_path, write_config_if_missing)?;
            Ok(ImportReport {
                mode: "combine",
                atoms_upserted: upserted,
                vectors_copied: vectors,
                vectors_skipped: skip_vectors,
                storage_path: dest,
            })
        }
    }
}

fn maybe_write_config(
    pack_config: &Path,
    dest_config: &Path,
    write_if_missing: bool,
) -> Result<()> {
    if !write_if_missing {
        return Ok(());
    }
    if dest_config.exists() {
        return Ok(());
    }
    if let Some(parent) = dest_config.parent() {
        fs::create_dir_all(parent)?;
    }
    copy_private(pack_config, dest_config)?;
    Ok(())
}

/// Resolve the active config file path (CLI `--config` or default).
pub fn resolve_config_file(cli_config: Option<&Path>) -> Result<PathBuf> {
    Ok(cli_config.map(Path::to_path_buf).unwrap_or(config_path()?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::KnowledgeAtom;
    use chrono::Utc;

    fn sample_atom(id: &str, content: &str) -> KnowledgeAtom {
        KnowledgeAtom {
            id: id.into(),
            source: "test".into(),
            source_id: id.into(),
            title: id.into(),
            summary: String::new(),
            content: content.into(),
            tags: vec!["export".into()],
            soft_labels: vec![],
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            last_accessed_at: Utc::now(),
            embedding: Some(vec![0.1, 0.2, 0.3, 0.4]),
            ..KnowledgeAtom::default()
        }
    }

    #[tokio::test]
    async fn export_import_replace_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let db = tmp.path().join("store.db");
        let cfg_path = tmp.path().join("config.toml");
        fs::write(
            &cfg_path,
            format!(
                r#"environment = "dev"
[storage]
path = "{db}"
[embed]
model = "openai/text-embedding-3-large"
dimension = 4
[runtime]
poll_interval_secs = 300
"#,
                db = db.display()
            ),
        )
        .unwrap();

        let store = SqliteVecStore::open(db.clone(), 4).unwrap();
        store
            .upsert(&sample_atom("a1", "hello export world"))
            .await
            .unwrap();

        let config = crate::config::load_config_from(&cfg_path).unwrap();
        let pack = tmp.path().join("brain.kurultai");
        let report = export_pack(&config, &cfg_path, Some(&pack)).unwrap();
        assert_eq!(report.atom_count, 1);
        assert!(pack.exists());

        let dest_dir = tempfile::tempdir().unwrap();
        let dest_db = dest_dir.path().join("store.db");
        let dest_cfg = dest_dir.path().join("config.toml");
        fs::write(
            &dest_cfg,
            format!(
                r#"environment = "dev"
[storage]
path = "{dest_db}"
[embed]
model = "openai/text-embedding-3-large"
dimension = 4
[runtime]
poll_interval_secs = 300
"#,
                dest_db = dest_db.display()
            ),
        )
        .unwrap();
        let dest_config = crate::config::load_config_from(&dest_cfg).unwrap();
        let imp = import_pack(
            &dest_config,
            &pack,
            ImportMode::Replace { force: false },
            false,
            &dest_cfg,
        )
        .await
        .unwrap();
        assert_eq!(imp.atoms_upserted, 1);

        let opened = SqliteVecStore::open(dest_db, 4).unwrap();
        let got = opened.get("a1").await.unwrap().unwrap();
        assert!(got.content.contains("export world"));
    }

    #[tokio::test]
    async fn replace_refuses_nonempty_without_force() {
        let tmp = tempfile::tempdir().unwrap();
        let src_db = tmp.path().join("src.db");
        let dst_db = tmp.path().join("dst.db");
        let src_cfg = tmp.path().join("src.toml");
        let dst_cfg = tmp.path().join("dst.toml");

        for (cfg, db, id, body) in [
            (&src_cfg, &src_db, "from-pack", "alpha"),
            (&dst_cfg, &dst_db, "keep-me", "beta"),
        ] {
            fs::write(
                cfg,
                format!(
                    r#"environment = "dev"
[storage]
path = "{db}"
[embed]
dimension = 4
model = "x"
[runtime]
poll_interval_secs = 300
"#,
                    db = db.display()
                ),
            )
            .unwrap();
            let store = SqliteVecStore::open(db.clone(), 4).unwrap();
            store.upsert(&sample_atom(id, body)).await.unwrap();
        }

        let src_config = crate::config::load_config_from(&src_cfg).unwrap();
        let pack = tmp.path().join("c.kurultai");
        export_pack(&src_config, &src_cfg, Some(&pack)).unwrap();

        let dst_config = crate::config::load_config_from(&dst_cfg).unwrap();
        let err = import_pack(
            &dst_config,
            &pack,
            ImportMode::Replace { force: false },
            false,
            &dst_cfg,
        )
        .await
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("--force") || msg.contains("--combine"),
            "{msg}"
        );
        assert!(SqliteVecStore::open(dst_db, 4)
            .unwrap()
            .get("keep-me")
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn combine_merges_atoms_into_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let src_db = tmp.path().join("src.db");
        let dst_db = tmp.path().join("dst.db");
        let src_cfg = tmp.path().join("src.toml");
        let dst_cfg = tmp.path().join("dst.toml");

        for (cfg, db, id, body) in [
            (&src_cfg, &src_db, "from-pack", "alpha unique phrase"),
            (&dst_cfg, &dst_db, "local-only", "beta local phrase"),
        ] {
            fs::write(
                cfg,
                format!(
                    r#"environment = "dev"
[storage]
path = "{db}"
[embed]
dimension = 4
model = "x"
[runtime]
poll_interval_secs = 300
"#,
                    db = db.display()
                ),
            )
            .unwrap();
            let store = SqliteVecStore::open(db.clone(), 4).unwrap();
            store.upsert(&sample_atom(id, body)).await.unwrap();
        }

        let src_config = crate::config::load_config_from(&src_cfg).unwrap();
        let pack = tmp.path().join("c.kurultai");
        export_pack(&src_config, &src_cfg, Some(&pack)).unwrap();

        let dst_config = crate::config::load_config_from(&dst_cfg).unwrap();
        let report = import_pack(&dst_config, &pack, ImportMode::Combine, false, &dst_cfg)
            .await
            .unwrap();
        assert_eq!(report.atoms_upserted, 1);

        let opened = SqliteVecStore::open(dst_db, 4).unwrap();
        assert!(opened.get("from-pack").await.unwrap().is_some());
        assert!(opened.get("local-only").await.unwrap().is_some());
    }

    #[test]
    fn redact_strips_api_key_lines() {
        let raw = "environment = \"dev\"\napi_key = \"sk-secret\"\n[embed]\ndimension = 4\n";
        let redacted = redact_secret_keys(raw);
        assert!(!redacted.contains("sk-secret"));
        assert!(redacted.contains("redacted"));
    }

    #[test]
    fn synthesized_fallback_uses_live_embed_dim() {
        let tmp = tempfile::tempdir().unwrap();
        let db = tmp.path().join("store.db");
        SqliteVecStore::open(db.clone(), 8).unwrap();

        let missing_cfg = tmp.path().join("missing.toml");
        let config = Config {
            environment: crate::environment::Environment::Dev,
            sources: vec![],
            storage_path: db.to_string_lossy().into_owned(),
            embed_model: "local/test-model".into(),
            embed_dim: 8,
            embed_backend: None,
            reranker_model: None,
            poll_interval_secs: 300,
            nightly_full_sync_hour: None,
            inactivity_threshold_hours: None,
            mcp_http_secret: None,
            banner: crate::art::BannerMode::Auto,
        };

        let pack = tmp.path().join("fb.kurultai");
        export_pack(&config, &missing_cfg, Some(&pack)).unwrap();

        let file = File::open(&pack).unwrap();
        let dec = GzDecoder::new(file);
        let mut archive = Archive::new(dec);
        let dir = tempfile::tempdir().unwrap();
        archive.unpack(dir.path()).unwrap();
        let cfg_text = fs::read_to_string(dir.path().join(CONFIG_NAME)).unwrap();
        assert!(cfg_text.contains("dimension = 8"), "{cfg_text}");
        assert!(cfg_text.contains("local/test-model"), "{cfg_text}");
        assert!(!cfg_text.contains("dimension = 3072"));
    }
}
