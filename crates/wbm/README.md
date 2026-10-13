# archivindex-truthsocial-wbm

Wayback Machine archive integration for Truth Social data, built on `archivindex-wbm-json` and the
shared types from `archivindex-truthsocial`.

`validation_context` configures snapshot validation and gzip reproduction for Truth Social
responses. `read_statuses` streams statuses from zstd-compressed compact snapshot files, including
responses containing arrays of statuses. `merge_compact` merges two compact files in digest order,
removes duplicates, and rejects digest collisions or unsorted input. The `cdx` module reads
directories of CDX entry files.

## License

Licensed under the [GNU Affero General Public License, version 3 only][agpl-3.0].

[agpl-3.0]: https://www.gnu.org/licenses/agpl-3.0.html
