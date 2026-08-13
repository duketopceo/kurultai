//! Dump ingestion — shared atomizer for inbox, loopback webhook, and folder sources.

pub mod dump;

pub use dump::{
    atomize_bytes, atomize_path, compute_quality_score, detect_format, DumpFormat, INBOX_META_PATH,
    INBOX_META_ROOT, QUALITY_SCORE_KEY,
};
