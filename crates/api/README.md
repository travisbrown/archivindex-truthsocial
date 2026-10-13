# archivindex-truthsocial-api

An asynchronous client for the Truth Social HTTP API, using the shared types from
`archivindex-truthsocial`. It supports account and status lookups, timelines, groups, and search.

Supply a bearer token with `Client::with_token`, or call `Client::login` with application and user
credentials. `Credentials::from_env` reads credentials from environment variables.

For browser-assisted setup, run `cargo run -p archivindex-truthsocial-api-cli -- setup` from the
workspace. Load the saved session with `Config::load` and apply it with `ClientBuilder::config`.
Setup captures a bearer token when it can find one; otherwise, supply a token or call `login`.

## License

Licensed under the [GNU Affero General Public License, version 3 only][agpl-3.0].

[agpl-3.0]: https://www.gnu.org/licenses/agpl-3.0.html
