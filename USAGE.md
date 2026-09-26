# Usage

## Connecting

Every command needs five pieces of connection info:

| Value           | Flag                   | Env var         |
|-----------------|------------------------|-----------------|
| Server URL      | `--url` / `-u`         | `DATABASE_URL`  |
| Root username   | `--user` / `-U`        | `SURREAL_USER`  |
| Root password   | `--pass` / `-P`        | `SURREAL_PASS`  |
| Namespace       | `--namespace` / `-n`   | `NAMESPACE`     |
| Database name   | `--database-name`/`-d` | `DATABASE_NAME` |

Easiest way is a `.env` file in your working directory:

```env
DATABASE_URL=ws://localhost:8000
SURREAL_USER=root
SURREAL_PASS=your_password
NAMESPACE=Development
DATABASE_NAME=app
```

**`DATABASE_URL` must include a scheme** — `ws://`, `wss://`, `http://`, or
`https://`. `sudm` will refuse to connect otherwise.

**Never commit your `.env` file.** Add it to `.gitignore` before your first
commit — it contains your database credentials.

### How connection works

- Commands that only need to talk to the cluster itself (`inspect`, `ns`,
  `db`) connect and authenticate, but don't bind to any namespace/database
  up front.
- Commands that operate inside a specific namespace/database (`migrate`,
  and anything using the tracked `sudm_migrations` table) validate that the
  `NAMESPACE`/`DATABASE_NAME` you gave actually exist on the server *before*
  binding to them — `sudm` will never silently create a namespace or
  database just because you typo'd its name.

## Commands

### `sudm inspect`

```bash
sudm inspect tree   # readable namespace -> database -> table view
sudm inspect raw    # full recursive JSON dump of the cluster (debugging)
```

### `sudm ns` — namespaces

```bash
sudm ns new <name>
sudm ns remove <name>     # asks for confirmation before deleting
```

### `sudm db` — databases

```bash
sudm db new <ns_name> <db_name>
sudm db remove <ns_name> <db_name>   # asks for confirmation before deleting
```

### `sudm schema` — schema scaffolding

```bash
sudm schema new <name>     # creates migrations/Schema/<name>/
sudm schema add <name>     # adds a new timestamped migration inside it
```

`schema add` creates a folder like this:

    migrations/Schema/<name>/<timestamp>_<name>/
    ├── <name>-schema.surql      table/field/index definitions
    ├── <name>-functions.surql   functions for this migration
    ├── <name>-events.surql      events (triggers) for this migration
    └── rollback.surql           reverts everything above

Each migration is applied as one atomic unit: schema first, then functions,
then events, in that order — so functions/events can safely reference
tables the schema file just created. Write the full reversal of all three
files in `rollback.surql` if you want this migration to be undoable.

### `sudm migrate` — running migrations

```bash
sudm migrate up             # applies all pending migrations, oldest first
sudm migrate down           # rolls back the most recently applied BATCH
sudm migrate down --force   # skip confirmation prompts on empty rollback.surql
```

Every `migrate up` run is recorded as one **batch**. `migrate down` rolls
back every migration in the most recent batch together, in reverse order —
not just the single last file — so a multi-migration deploy can be undone
as one unit.

`migrate up` stops at the first failing migration rather than skipping
ahead, so migrations never get recorded as applied out of order.

If a migration's `rollback.surql` is empty (or missing), `sudm` warns you
before marking it unapplied, since nothing will actually be undone in the
database.

## Security notes

- Prefer namespace- or database-scoped users over the root credentials for
  everyday use, so a leaked `.env` doesn't expose your whole cluster.
- If credentials are ever exposed publicly (e.g. an accidental commit),
  rotate them immediately — deleting the file or rewriting git history does
  not undo the exposure.