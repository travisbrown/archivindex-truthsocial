# archivindex-truthsocial-api

An asynchronous client for the Truth Social HTTP API, using the shared types from
`archivindex-truthsocial`. It supports account and status lookups, timelines, groups, and search.

Supply a bearer token with `Client::with_token`, or call `Client::login` with application and user
credentials. `Credentials::from_env` reads credentials from environment variables.

For browser-assisted setup, run `cargo run -p archivindex-truthsocial-api-cli -- setup` from the
workspace. Load the saved session with `Config::load` and apply it with `ClientBuilder::config`.
Setup captures a bearer token when it can find one; otherwise, supply a token or call `login`.

## Dependency patches

The workspace pins upstream `wreq-util` to include unreleased Zstandard dependency fixes, together
with the matching `wreq` and TLS revisions. Applications depending on this crate must copy the
`[patch.crates-io]` entries from the [workspace manifest](../../Cargo.toml) into their own root
manifest. Cargo does not inherit patches from dependencies.

## License

Licensed under the [GNU Affero General Public License, version 3 only][agpl-3.0].

[agpl-3.0]: https://www.gnu.org/licenses/agpl-3.0.html
