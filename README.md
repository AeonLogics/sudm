# sudm

Async SurrealDB migration CLI.

`sudm` bridges your local schema files with a live SurrealDB instance —
scaffold schema folders, generate timestamped migrations, and apply or roll
them back against your database, with everything tracked in a
`sudm_migrations` table so `dev` and `prod` each keep their own independent
migration history.

Built in Rust with async Tokio, Clap, and the official SurrealDB SDK.

## Quick start

```bash
git clone https://github.com/AeonLogics/sudm.git
cd sudm
cargo build --release
```

See [USAGE.md](USAGE.md) for connection setup and the full command list.

## License

MIT — free to use, fork, and experiment with. See [LICENSE](LICENSE).

## Contributing

There's no strict roadmap here — if you have an idea, feel free to build it
and open a PR.

See [USAGE.md](USAGE.md) for connection setup and the full command list.