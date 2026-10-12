# sudm

Async SurrealDB schema and function CLI.

`sudm` bridges your local `.surql` files with a live SurrealDB instance. It automatically provisions your namespaces and databases on the fly, reads your ordered schema files, and pushes them straight to the cluster using SurrealDB's native idempotent upserts. No rollback scripts, no migration state tables—just pure declarative definitions.

Built in Rust with async Tokio, Clap, and the official SurrealDB SDK.

## Quick start

Clone and build from source:

```bash
git clone https://github.com/AeonLogics/sudm.git
cd sudm
cargo build --release
```

> **Note:** Pre-built release binaries will be available directly on [aeonlogics.dev](https://aeonlogics.dev) soon.

## Commands

* `sudm add <name>` — Creates a new schema and function `.surql` file template.
* `sudm run mig` — Connects to the cluster, auto-provisions the namespace/database if missing, and applies all local `.surql` files in alphanumeric order.

## License

MIT — free to use, fork, and experiment with. See [LICENSE](LICENSE).