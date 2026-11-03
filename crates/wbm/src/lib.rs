//! Wayback Machine (WTJ) archive snapshot integration for Truth Social types.
//!
//! This crate plugs [`archivindex_truthsocial::model::Status`] into the `archivindex-wbm-json`
//! snapshot infrastructure. [`validation_context`] builds the Truth Social [`Context`] (with the
//! gzip codec), [`WtjSnapshot`] wraps a snapshot's [`StatusContent`], [`read_statuses`] streams
//! statuses out of a zstd-compressed compact snapshot file, and [`merge_compact`] merges two such
//! files in digest order. The [`cdx`] module reads directories of CDX entry files.
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use archivindex_truthsocial::model::{Status, StatusContent};
use archivindex_wbm::digest::Sha1Digest;
use archivindex_wbm_json::context::Context;
use archivindex_wbm_json::exact::ExactSnapshot;
use archivindex_wbm_json_processing::io::read::SnapshotReader;
use archivindex_wbm_json_processing::io::zst;
use archivindex_wbm_json_processing::process::merge::{self, Collision, SourceLine};
use either::Either;

pub mod cdx;

/// A WTJ snapshot whose content is the deserialized [`StatusContent`]: a single [`Status`] or, for
/// endpoints that return a list (e.g. paged video feeds), an array of them.
pub type WtjSnapshot<'a> = archivindex_wbm_json::Snapshot<'a, StatusContent<'a>>;

/// A WTJ snapshot whose content is the exact serialized bytes, so it can be parsed,
/// digest-validated, and re-emitted byte-for-byte. This is the form used when reading and merging
/// compact files.
pub type CompactSnapshot<'a> = ExactSnapshot<'a>;

/// The bundled Truth Social context configuration (default closing whitespace and the status-URL
/// CEL query). Deserialized into an [`archivindex_wbm_json::context::ContextConfig`].
const CONTEXT_CONFIG: &str = include_str!("truthsocial.toml");

/// Builds a [`Context`] for reading, validating, and serializing compact WTJ snapshot files.
///
/// This is the library's canonical Truth Social context, defined by the bundled `truthsocial.toml`
/// (default closing whitespace `['\n']` and the status-URL CEL inference) and extended with the
/// [`gzip`](archivindex_wbm_json_gzip) codec. A snapshot whose content is decompressed from a gzip
/// archive carries the reproduction parameters in its `format` object, so it validates against the
/// digest of its original compressed bytes.
///
/// # Panics
///
/// Panics if the bundled configuration is not valid (invalid TOML or an invalid CEL `url_query`,
/// both checked while deserializing [`ContextConfig`]). This indicates a bug in the bundled
/// configuration.
///
/// [`ContextConfig`]: archivindex_wbm_json::context::ContextConfig
#[must_use]
pub fn validation_context() -> Context {
    let config: archivindex_wbm_json::context::ContextConfig = toml::from_str(CONTEXT_CONFIG)
        .expect("bundled truthsocial.toml should be valid TOML with a valid url_query");
    let mut context = Context::from_config(config);
    archivindex_wbm_json_gzip::register(&mut context);
    context
}

/// An error encountered while reading a compact WTJ snapshot file.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The file could not be opened, or the decoder could not be initialized.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// A line could not be read or decompressed, or does not hold a snapshot.
    #[error(transparent)]
    Read(#[from] archivindex_wbm_json_processing::io::read::Error),
    /// A snapshot's content could not be parsed as [`StatusContent`].
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Streams Truth Social statuses from a zstd-compressed compact WTJ snapshot file.
///
/// The file is zstd-compressed with one JSON [`WtjSnapshot`] per line, and blank lines are errors.
/// Each yielded item is one owned [`Status`]; a snapshot whose content is an array of statuses
/// yields each of them in turn. Statuses are produced lazily, so the whole file is never held in
/// memory at once.
///
/// # Errors
///
/// Returns [`Error::Io`] if the file cannot be opened or the decoder cannot be initialized. Each
/// iterator item is an [`Error::Read`] for a line that cannot be read or is not a snapshot, or an
/// [`Error::Json`] for a snapshot whose content fails to deserialize. The iterator ends after a
/// read or decompression failure, so a caller may skip errors and keep reading.
pub fn read_statuses<P: AsRef<Path>>(
    path: P,
) -> Result<impl Iterator<Item = Result<Status<'static>, Error>>, Error> {
    Ok(SnapshotReader::open(path)?.flat_map(|snapshot| {
        let content = snapshot.map_err(Error::Read).and_then(|snapshot| {
            serde_json::from_str::<StatusContent<'static>>(snapshot.content.as_str())
                .map_err(Error::Json)
        });

        // Flatten each snapshot's content into its individual statuses (one for a single status,
        // several for an array), threading any error through as a single item.
        match content {
            Ok(content) => Either::Left(content.into_statuses().map(Ok)),
            Err(error) => Either::Right(std::iter::once(Err(error))),
        }
    }))
}

/// An error encountered while merging two compact WTJ snapshot files.
#[derive(Debug, thiserror::Error)]
pub enum MergeError {
    /// An input file could not be opened.
    #[error("failed to open {}", .path.display())]
    Open {
        /// The file that could not be opened.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// An input could not be read, is not strictly sorted by digest, or has a line with no digest.
    #[error(transparent)]
    Input(#[from] merge::Error),
    /// A line carries a digest but could not be parsed as a compact snapshot.
    #[error("failed to parse the snapshot with digest {digest}")]
    Parse {
        /// The digest the line carries.
        digest: Sha1Digest,
        /// The underlying parse error.
        #[source]
        source: archivindex_wbm_json::Error,
    },
    /// The merged output could not be written.
    #[error("failed to write merged output")]
    Write(#[source] std::io::Error),
    /// Two rows share a digest but disagree about the captured bytes, which means that at least one
    /// of them is corrupt (or that this is a real SHA-1 collision).
    #[error(
        "digest collision: rows for {digest} differ in content or format \
         (line {first_line} of the first input, line {second_line} of the second)"
    )]
    Collision {
        /// The digest shared by the conflicting rows.
        digest: Sha1Digest,
        /// The one-based line number of the row in the first input.
        first_line: usize,
        /// The one-based line number of the row in the second input.
        second_line: usize,
    },
}

/// Summary of a [`merge_compact`] run.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MergeSummary {
    /// Total rows read from both input files.
    pub read: usize,
    /// Rows dropped because the other input had a row for the same digest.
    pub duplicates: usize,
    /// How many of the `duplicates` differed from the row that was kept in capture metadata.
    pub metadata_conflicts: usize,
    /// Distinct rows written to the output.
    pub written: usize,
}

/// Merges two compact WTJ snapshot files into `writer`, ordered by SHA-1 digest byte order.
///
/// Both inputs must be zstd-compressed and strictly sorted by digest bytes, with no blank lines.
/// Each digest is written once. Lines are copied verbatim, so the digests in the output remain
/// valid, though the merge does not itself verify them.
///
/// A digest identifies captured bytes rather than a capture, so two inputs can hold different rows
/// for the same digest. If the rows differ only in capture metadata (`timestamp`, `url`, or
/// `expected_digest`), one row is kept whole and a warning is logged: a row with a timestamp beats
/// one without, an earlier timestamp beats a later one, and among equal timestamps the shorter line
/// wins. If they differ in content or format, the merge aborts with [`MergeError::Collision`],
/// since keeping either row could discard the real capture.
///
/// Output is uncompressed JSON, one snapshot per line. An error may leave partial output; the
/// caller is responsible for flushing or finalizing `writer`.
///
/// # Errors
///
/// [`MergeError::Open`] / [`MergeError::Input`] / [`MergeError::Parse`] for input problems,
/// [`MergeError::Write`] for output problems, and [`MergeError::Collision`] when two rows disagree
/// about the captured bytes.
pub fn merge_compact<W: Write>(
    first: &Path,
    second: &Path,
    mut writer: W,
) -> Result<MergeSummary, MergeError> {
    let lines = |path: &Path| {
        zst::reader(path)
            .map(BufRead::lines)
            .map_err(|source| MergeError::Open {
                path: path.to_path_buf(),
                source,
            })
    };
    let mut summary = MergeSummary::default();

    for item in merge::merge(lines(first)?, lines(second)?) {
        let (digest, source_line) = item?;

        let line: &str = match &source_line {
            SourceLine::First(line) | SourceLine::Second(line) => line,
            SourceLine::Match(line) => {
                summary.duplicates += 1;
                log::warn!("Dropped a duplicate row for digest {digest}");
                line
            }
            SourceLine::Collision(collision) => {
                let line = resolve_collision(collision)?;
                summary.duplicates += 1;
                summary.metadata_conflicts += 1;
                log::warn!("Dropped a row with differing capture metadata for digest {digest}");
                line
            }
        };

        parse_row(digest, line)?;
        writeln!(writer, "{line}").map_err(MergeError::Write)?;
        summary.written += 1;
    }

    summary.read = summary.written + summary.duplicates;

    Ok(summary)
}

/// Selects which of two differing rows with the same digest to keep, in the order documented on
/// [`merge_compact`].
///
/// Rows are never combined, so the kept metadata always describes a single capture. The first input
/// breaks any tie that remains after comparing timestamps and line lengths.
fn resolve_collision(collision: &Collision) -> Result<&str, MergeError> {
    let digest = collision.digest;
    let first = parse_row(digest, &collision.first_value)?;
    let second = parse_row(digest, &collision.second_value)?;

    if first.content != second.content || first.format != second.format {
        Err(MergeError::Collision {
            digest,
            first_line: collision.first_line_number,
            second_line: collision.second_line_number,
        })
    } else {
        let rank = |row: &CompactSnapshot<'_>, line: &str| {
            (row.timestamp.is_none(), row.timestamp, line.len())
        };

        Ok(
            if rank(&second, &collision.second_value) < rank(&first, &collision.first_value) {
                &collision.second_value
            } else {
                &collision.first_value
            },
        )
    }
}

/// Parses a line that carries `digest` as a compact snapshot.
fn parse_row(digest: Sha1Digest, line: &str) -> Result<CompactSnapshot<'_>, MergeError> {
    CompactSnapshot::parse(line).map_err(|source| MergeError::Parse { digest, source })
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::unreadable_literal,
        reason = "status IDs are long opaque integers; digit separators do not aid readability"
    )]

    use super::*;

    /// The curated Truth Social snapshot corpus, shared with the `archivindex-truthsocial` crate's
    /// fixtures. Each file is named by the uppercase Base32 SHA-1 digest of its original bytes and
    /// holds one snapshot's `StatusContent` payload; some are gzip-compressed (magic `1f 8b`).
    const CURATED_SNAPSHOTS_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../core/tests/data/wbm/snapshots"
    );

    /// Decodes a curated snapshot payload to its JSON text, transparently gunzipping when the gzip
    /// magic bytes are present.
    ///
    /// Decoding routes through the archivindex gzip codec rather than `flate2` directly; the
    /// corpus's byte-for-byte reproducibility is guarded separately by
    /// [`validation_context_reproduces_gzip_snapshots`].
    fn decode_snapshot(bytes: &[u8]) -> String {
        if bytes.starts_with(&[0x1f, 0x8b]) {
            archivindex_wbm_json_gzip::decompress(bytes)
                .expect("gzip snapshot decompresses to text")
        } else {
            String::from_utf8(bytes.to_vec()).expect("snapshot is UTF-8 JSON")
        }
    }

    /// Build test envelopes from single-status fixtures, sorted by decoded digest.
    /// Content is reserialized, so these envelopes are suitable for parsing and merge tests,
    /// but their digests may no longer match the content bytes. Array fixtures are skipped.
    fn curated_single_status_lines() -> Vec<String> {
        let mut paths = std::fs::read_dir(CURATED_SNAPSHOTS_DIR)
            .expect("curated snapshots directory exists")
            .map(|entry| entry.expect("readable directory entry").path())
            .filter(|path| path.is_file())
            .collect::<Vec<_>>();
        // Sort by the decoded digest, not the filename: Base32's digits (`2`–`7`) encode the high
        // values 26–31 yet sort below `A` in ASCII, so a lexical filename sort is not digest order,
        // which the streaming merge requires.
        paths.sort_by_key(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .expect("filename is valid UTF-8")
                .parse::<Sha1Digest>()
                .expect("filename is a Base32 SHA-1 digest")
        });

        paths
            .iter()
            .filter_map(|path| {
                let digest = path.file_name()?.to_str()?;
                let content = decode_snapshot(&std::fs::read(path).expect("read snapshot"));
                let value: serde_json::Value =
                    serde_json::from_str(&content).expect("snapshot is JSON");
                // Re-serialize compactly so each line is a single line of valid JSON with the
                // `digest` field first and `content` last, as the exact-bytes parser requires.
                value.is_object().then(|| {
                    format!(
                        r#"{{"digest":"{digest}","content":{}}}"#,
                        serde_json::to_string(&value).expect("re-serialize content")
                    )
                })
            })
            .collect()
    }

    /// A curated single-status snapshot wrapped as a compact envelope deserializes as a
    /// [`WtjSnapshot`], exercising the core `StatusContent` schema of its content.
    #[test]
    fn parses_snapshot_envelope() {
        let lines = curated_single_status_lines();
        let line = lines
            .first()
            .expect("at least one curated single-status snapshot");
        let snapshot: WtjSnapshot<'_> = serde_json::from_str(line).expect("snapshot deserializes");
        assert!(matches!(snapshot.content, StatusContent::Single(_)));
    }

    /// The context infers a status's canonical URL (via its CEL query) from its content.
    #[test]
    fn context_infers_status_url() {
        // The status ID in this fixture determines the inferred URL.
        let json = decode_snapshot(include_bytes!(
            "../../core/tests/data/wbm/snapshots/5QIKZOW6KFPXX3Q2UNR4HORDC2IPHFSZ"
        ));

        let url = validation_context().infer_url(&json);
        assert_eq!(
            url.as_deref(),
            Some("https://truthsocial.com/api/v1/statuses/114433942825461076")
        );
    }

    /// The context defaults to a single trailing newline when reproducing captured content.
    #[test]
    fn default_closing_whitespace_is_newline() {
        assert_eq!(validation_context().default_closing_whitespace(), &['\n']);
    }

    /// The registered gzip codec reproduces each compressed fixture byte-for-byte.
    #[test]
    fn validation_context_reproduces_gzip_snapshots() {
        use archivindex_wbm::digest::Sha1Digest;
        use archivindex_wbm_json::format::Format;
        use archivindex_wbm_json_gzip::{FORMAT, GzipParams};

        let context = validation_context();
        let format_name = Format::from(FORMAT);
        let mut hasher = sha1::Sha1::default();

        let mut count: usize = 0;
        let mut compressors = std::collections::BTreeSet::new();
        for entry in std::fs::read_dir(CURATED_SNAPSHOTS_DIR).expect("read curated snapshots") {
            let path = entry.unwrap().path();
            let bytes = std::fs::read(&path).expect("read snapshot");
            // The curated corpus stores original response bytes; only the gzip-compressed ones
            // (magic `1f 8b`) exercise the gzip codec.
            if !bytes.starts_with(&[0x1f, 0x8b]) {
                continue;
            }
            let name = path.file_name().unwrap().to_string_lossy().into_owned();

            let params = GzipParams::infer(&bytes)
                .unwrap_or_else(|| panic!("{name}: no gzip parameters inferred"));
            compressors.insert(params.compressor);

            // Build the snapshot the way `compact` does: decode under the gzip format, then attach
            // the inferred reproduction metadata.
            let mut snapshot = context
                .unprocessed_snapshot(&format_name, &bytes)
                .unwrap_or_else(|error| panic!("{name}: unprocessed_snapshot: {error}"));
            snapshot.format.metadata = params.metadata();

            assert_eq!(
                snapshot.digest,
                Sha1Digest::compute(&bytes),
                "{name}: digest"
            );
            assert_eq!(
                context.verify(&snapshot, &mut hasher),
                Ok(()),
                "{name}: verify"
            );
            count += 1;
        }

        // Require at least one gzip fixture and report the compressors covered.
        assert!(
            count > 0,
            "no gzip snapshots found in {CURATED_SNAPSHOTS_DIR}"
        );
        let _ = writeln!(
            std::io::stderr(),
            "reproduced {count} gzip curated snapshot(s); compressors seen: {compressors:?}"
        );
    }

    /// Every line of a compact snapshot file (built from the curated single-status snapshots)
    /// parses into a status, in ascending digest order.
    #[test]
    fn read_statuses_from_compact_file() {
        let lines = curated_single_status_lines();
        let refs = lines.iter().map(String::as_str).collect::<Vec<_>>();
        let file = compact_file(&refs);

        let statuses = read_statuses(file.path())
            .expect("open compact snapshot file")
            .collect::<Result<Vec<_>, _>>()
            .expect("every line parses into a status");

        // One status per curated single-status snapshot; the first is `AAACL3SP…` in digest order.
        assert_eq!(statuses.len(), lines.len());
        assert_eq!(statuses[0].id, 115864791948135518);
    }

    /// `read_statuses` flattens array content: a snapshot whose content is an array of statuses
    /// yields each of them. The array form previously failed to parse as a single `Status` (a
    /// derived struct also accepts sequence form, so the array was read as a positional status
    /// whose first field, the integer-string `id`, received the first element, an object).
    #[test]
    fn read_statuses_flattens_array_content() {
        // Seed from a curated single-status line, then reuse its content as an array of two.
        let lines = curated_single_status_lines();
        let seed = lines
            .first()
            .expect("at least one curated single-status snapshot");
        let single: serde_json::Value = serde_json::from_str(seed).expect("parse seed line");
        let status = single["content"].clone();
        let array_line = serde_json::json!({
            "digest": single["digest"],
            "content": [status.clone(), status],
        })
        .to_string();

        let file = compact_file(&[seed, &array_line]);
        let statuses = read_statuses(file.path())
            .expect("open compact snapshot file")
            .collect::<Result<Vec<_>, _>>()
            .expect("array content flattens into statuses");

        // One status from the single-status line, two from the array line.
        assert_eq!(statuses.len(), 3);
    }

    /// A truncated file yields one error and then ends. The reader used to repeat the same
    /// decompression error forever, so a caller that skipped errors never finished.
    #[test]
    fn read_statuses_ends_after_corrupt_input() {
        let lines = curated_single_status_lines();
        let refs = lines.iter().map(String::as_str).collect::<Vec<_>>();
        let bytes = std::fs::read(compact_file(&refs).path()).expect("read compact file");
        let truncated = tempfile::NamedTempFile::new().expect("temp file");
        std::fs::write(truncated.path(), &bytes[..bytes.len() / 2]).expect("write truncated file");

        let items = read_statuses(truncated.path())
            .expect("open truncated file")
            .take(lines.len() + 1)
            .collect::<Vec<_>>();

        assert_eq!(items.iter().filter(|item| item.is_err()).count(), 1);
        assert!(
            matches!(items.last(), Some(Err(Error::Read(_)))),
            "expected a final read error, got {:?}",
            items.last()
        );
    }

    /// Writes the given lines to a fresh zstd-compressed temp file.
    fn compact_file(lines: &[&str]) -> tempfile::NamedTempFile {
        let temp = tempfile::NamedTempFile::new().expect("temp file");
        let mut encoder = zstd::Encoder::new(temp.as_file(), 0).expect("zstd encoder");
        for line in lines {
            writeln!(encoder, "{line}").expect("write line");
        }
        encoder.finish().expect("finish zstd");
        temp
    }

    /// Merging a compact file (built from the curated single-status snapshots) with itself drops
    /// one copy of each row and emits the remaining rows in ascending digest order.
    #[test]
    fn merge_dedups_and_orders() {
        let lines = curated_single_status_lines();
        let refs = lines.iter().map(String::as_str).collect::<Vec<_>>();
        let file = compact_file(&refs);
        let n = lines.len();

        let mut output = Vec::new();
        let summary = merge_compact(file.path(), file.path(), &mut output).expect("merge succeeds");

        assert_eq!(summary.read, 2 * n);
        assert_eq!(summary.written, n);
        assert_eq!(summary.duplicates, n);
        assert_eq!(summary.read, summary.written + summary.duplicates);
        assert_eq!(summary.metadata_conflicts, 0);

        // The output is itself a valid compact file whose digests ascend.
        let text = std::str::from_utf8(&output).expect("utf-8 output");
        assert_eq!(text.lines().collect::<Vec<_>>(), refs);
        let mut previous: Option<Sha1Digest> = None;
        let mut count = 0;
        for line in text.lines() {
            let snapshot = ExactSnapshot::parse(line).expect("parse output line");
            if let Some(previous) = previous {
                assert!(snapshot.digest >= previous, "output not in digest order");
            }
            previous = Some(snapshot.digest);
            count += 1;
        }
        assert_eq!(count, n);
    }

    /// Two rows that share a digest but differ in content or format are a collision, even when
    /// their metadata would otherwise select one of them. Keeping either could lose the real
    /// capture.
    #[test]
    fn merge_detects_collision() {
        for (first, second) in [
            (
                r#"{"digest":"AAAA3HVFIBJARGQ4ISEHROP6XWNULWTC","content":{"x":1}}"#,
                r#"{"digest":"AAAA3HVFIBJARGQ4ISEHROP6XWNULWTC","content":{"x":2}}"#,
            ),
            (
                r#"{"digest":"AAAA3HVFIBJARGQ4ISEHROP6XWNULWTC","timestamp":"20220101000000","content":{"x":1}}"#,
                r#"{"digest":"AAAA3HVFIBJARGQ4ISEHROP6XWNULWTC","content":{"x":2}}"#,
            ),
            (
                r#"{"digest":"AAAA3HVFIBJARGQ4ISEHROP6XWNULWTC","content":{"x":1}}"#,
                r#"{"digest":"AAAA3HVFIBJARGQ4ISEHROP6XWNULWTC","format":{"type":"gzip"},"content":{"x":1}}"#,
            ),
        ] {
            let first = compact_file(&[first]);
            let second = compact_file(&[second]);

            let mut output = Vec::new();
            let result = merge_compact(first.path(), second.path(), &mut output);

            assert!(
                matches!(
                    result,
                    Err(MergeError::Collision {
                        first_line: 1,
                        second_line: 1,
                        ..
                    })
                ),
                "expected Collision, got {result:?}"
            );
        }
    }

    /// The same bytes captured twice, or a row seen before and after enhancement, is the ordinary
    /// case when merging independently built files, not an error. The surviving row must not
    /// depend on the order of the inputs.
    #[test]
    fn merge_resolves_metadata_differences() {
        let earlier = r#"{"digest":"AAAA3HVFIBJARGQ4ISEHROP6XWNULWTC","timestamp":"20220101000000","content":{"x":1}}"#;
        let later = r#"{"digest":"AAAA3HVFIBJARGQ4ISEHROP6XWNULWTC","timestamp":"20230202000000","content":{"x":1}}"#;
        let unenhanced = r#"{"digest":"AAAA3HVFIBJARGQ4ISEHROP6XWNULWTC","content":{"x":1}}"#;
        let with_expected_digest = r#"{"digest":"AAAA3HVFIBJARGQ4ISEHROP6XWNULWTC","expected_digest":"BAAA3HVFIBJARGQ4ISEHROP6XWNULWTC","timestamp":"20220101000000","content":{"x":1}}"#;

        for (kept, dropped) in [
            (earlier, later),
            (later, unenhanced),
            (earlier, with_expected_digest),
        ] {
            for (first, second) in [(kept, dropped), (dropped, kept)] {
                let first = compact_file(&[first]);
                let second = compact_file(&[second]);

                let mut output = Vec::new();
                let summary = merge_compact(first.path(), second.path(), &mut output)
                    .expect("merge succeeds");

                assert_eq!(
                    summary,
                    MergeSummary {
                        read: 2,
                        duplicates: 1,
                        metadata_conflicts: 1,
                        written: 1,
                    }
                );
                assert_eq!(
                    std::str::from_utf8(&output).expect("utf-8 output"),
                    format!("{kept}\n")
                );
            }
        }
    }

    /// A line whose digest prefix is intact but whose remainder is not a snapshot must not reach
    /// the output, whether or not the other input has a row for that digest.
    #[test]
    fn merge_rejects_malformed_rows() {
        let malformed = r#"{"digest":"AAAA3HVFIBJARGQ4ISEHROP6XWNULWTC","content":{"x":1"#;
        let valid = r#"{"digest":"AAAA3HVFIBJARGQ4ISEHROP6XWNULWTC","content":{"x":1}}"#;

        for other in [vec![], vec![valid]] {
            let first = compact_file(&[malformed]);
            let second = compact_file(&other);

            let mut output = Vec::new();
            let result = merge_compact(first.path(), second.path(), &mut output);

            assert!(
                matches!(result, Err(MergeError::Parse { .. })),
                "expected Parse, got {result:?}"
            );
            assert_eq!(output, [] as [u8; 0]);
        }
    }

    /// A file whose digests are not ascending is rejected rather than silently mis-merged (the
    /// streaming merge requires sorted input).
    #[test]
    fn merge_detects_unsorted_input() {
        // `BAAA…` sorts after `AAAA…` by digest bytes, so this file is descending.
        let unsorted = compact_file(&[
            r#"{"digest":"BAAA3HVFIBJARGQ4ISEHROP6XWNULWTC","content":{"x":1}}"#,
            r#"{"digest":"AAAA3HVFIBJARGQ4ISEHROP6XWNULWTC","content":{"x":2}}"#,
        ]);
        let sorted =
            compact_file(&[r#"{"digest":"AAAA3HVFIBJARGQ4ISEHROP6XWNULWTC","content":{"x":3}}"#]);

        let mut output = Vec::new();
        let result = merge_compact(unsorted.path(), sorted.path(), &mut output);

        assert!(
            matches!(
                result,
                Err(MergeError::Input(merge::Error::Order {
                    side: merge::Side::First,
                    line_number: 2,
                    ..
                }))
            ),
            "expected an ordering error, got {result:?}"
        );
    }
}
