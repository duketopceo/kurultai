//! Yurt / Kurultai terminal art — human TTY surfaces only.
//!
//! Variants are caller-selected (no terminal-width auto-layout). Banner policy
//! centralizes TTY / `--plain` / `NO_COLOR` / config gating (KTD3).

use std::io::{self, IsTerminal, Write};

/// Compact banner for `status` / `init` (and similar short human output).
pub const YURT_COMPACT: &str = "\
  ╭── kurultai ──╮
  │  ⌂  yurt  ⌂  │
  ╰──────────────╯";

/// Wide banner aligned with the README header (optional `--help`).
pub const YURT_WIDE: &str = "\
                    ╭──────────────────────────╮
                   ╱    ·    kurultai    ·    ╲
                  │    ╭──────────────────╮    │
                  │   ╱   assemble what   ╲   │
                  │  │    you know  ·  yurt │  │
                  │   ╲   from wherever   ╱   │
                  │    ╰──────────────────╯    │
                   ╲         ⌂ ⌂ ⌂         ╱
                    ╰──────────────────────────╯";

/// ASCII-lighter sibling (no box-drawing / yurt glyphs).
pub const YURT_PLAIN: &str = "\
  +-- kurultai --+
  |   /\\ yurt /\\  |
  +--------------+";

/// Stable markers used by smoke tests to detect art leakage.
pub const ART_MARKER_BOX: &str = "╭";
pub const ART_MARKER_YURT: &str = "⌂";
pub const ART_MARKER_ASSEMBLE: &str = "assemble what";

/// Which art string to print when policy allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtVariant {
    Compact,
    Wide,
    Plain,
}

impl ArtVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            ArtVariant::Compact => YURT_COMPACT,
            ArtVariant::Wide => YURT_WIDE,
            ArtVariant::Plain => YURT_PLAIN,
        }
    }
}

/// Config `[cli] banner` / runtime presentation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BannerMode {
    /// Show only when stdout is a TTY (default).
    #[default]
    Auto,
    /// Always attempt to show (still suppressed by plain / NO_COLOR).
    #[serde(alias = "true", alias = "always", alias = "on")]
    Always,
    /// Never show.
    #[serde(alias = "false", alias = "never", alias = "off")]
    Never,
}

/// Whether art should be printed given config + suppress flags + TTY.
///
/// `plain` and `no_color` win over [`BannerMode::Always`].
pub fn should_show_banner(
    mode: BannerMode,
    plain: bool,
    no_color: bool,
    stdout_is_tty: bool,
) -> bool {
    if plain || no_color {
        return false;
    }
    match mode {
        BannerMode::Never => false,
        BannerMode::Always => true,
        BannerMode::Auto => stdout_is_tty,
    }
}

/// Write the selected variant followed by a newline when policy allows.
pub fn write_banner<W: Write>(
    out: &mut W,
    variant: ArtVariant,
    mode: BannerMode,
    plain: bool,
    no_color: bool,
    stdout_is_tty: bool,
) -> io::Result<bool> {
    if !should_show_banner(mode, plain, no_color, stdout_is_tty) {
        return Ok(false);
    }
    writeln!(out, "{}", variant.as_str())?;
    Ok(true)
}

/// Convenience: print compact/wide/plain to stdout when policy allows.
pub fn print_banner_stdout(
    variant: ArtVariant,
    mode: BannerMode,
    plain: bool,
    no_color: bool,
) -> io::Result<bool> {
    let mut out = io::stdout().lock();
    let tty = io::stdout().is_terminal();
    write_banner(&mut out, variant, mode, plain, no_color, tty)
}

/// `NO_COLOR` is set (any value) ⇒ treat as no-color / suppress art.
pub fn env_no_color_set() -> bool {
    std::env::var_os("NO_COLOR").is_some()
}

/// `KURULTAI_PLAIN` truthy: present and not empty / `0` / `false` / `no` / `off`.
pub fn env_kurultai_plain() -> bool {
    match std::env::var("KURULTAI_PLAIN") {
        Ok(v) => {
            let t = v.trim();
            !(t.is_empty()
                || t.eq_ignore_ascii_case("0")
                || t.eq_ignore_ascii_case("false")
                || t.eq_ignore_ascii_case("no")
                || t.eq_ignore_ascii_case("off"))
        }
        Err(_) => false,
    }
}

/// Effective plain flag from CLI `--plain` and `KURULTAI_PLAIN`.
pub fn effective_plain(cli_plain: bool) -> bool {
    cli_plain || env_kurultai_plain()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_wide_plain_strings_are_stable() {
        assert_eq!(
            YURT_COMPACT,
            "\
  ╭── kurultai ──╮
  │  ⌂  yurt  ⌂  │
  ╰──────────────╯"
        );
        assert_eq!(
            YURT_WIDE,
            "\
                    ╭──────────────────────────╮
                   ╱    ·    kurultai    ·    ╲
                  │    ╭──────────────────╮    │
                  │   ╱   assemble what   ╲   │
                  │  │    you know  ·  yurt │  │
                  │   ╲   from wherever   ╱   │
                  │    ╰──────────────────╯    │
                   ╲         ⌂ ⌂ ⌂         ╱
                    ╰──────────────────────────╯"
        );
        assert_eq!(
            YURT_PLAIN,
            "\
  +-- kurultai --+
  |   /\\ yurt /\\  |
  +--------------+"
        );
        assert_eq!(ArtVariant::Compact.as_str(), YURT_COMPACT);
        assert_eq!(ArtVariant::Wide.as_str(), YURT_WIDE);
        assert_eq!(ArtVariant::Plain.as_str(), YURT_PLAIN);
    }

    #[test]
    fn policy_matrix_tty_and_modes() {
        assert!(should_show_banner(BannerMode::Auto, false, false, true));
        assert!(!should_show_banner(BannerMode::Auto, false, false, false));
        assert!(should_show_banner(BannerMode::Always, false, false, false));
        assert!(!should_show_banner(BannerMode::Never, false, false, true));
    }

    #[test]
    fn plain_and_no_color_win_over_always() {
        assert!(!should_show_banner(BannerMode::Always, true, false, true));
        assert!(!should_show_banner(BannerMode::Always, false, true, true));
        assert!(!should_show_banner(BannerMode::Always, true, true, true));
    }

    #[test]
    fn write_banner_writes_or_suppresses() {
        let mut buf = Vec::new();
        let shown = write_banner(
            &mut buf,
            ArtVariant::Compact,
            BannerMode::Always,
            false,
            false,
            false,
        )
        .unwrap();
        assert!(shown);
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains(ART_MARKER_BOX));
        assert!(s.contains(ART_MARKER_YURT));
        assert!(s.ends_with('\n'));

        let mut buf = Vec::new();
        let shown = write_banner(
            &mut buf,
            ArtVariant::Compact,
            BannerMode::Always,
            true,
            false,
            true,
        )
        .unwrap();
        assert!(!shown);
        assert!(buf.is_empty());
    }
}
