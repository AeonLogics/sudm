# 🚀 sudm

**`sudm`** is a managed, asynchronous **SurrealDB migration driver and CLI utility** written in Rust. It serves as a seamless bridge between your local schema source files and your live SurrealDB instances, ensuring database evolutionary changes are structured, tracked, and painless.

---

## 📁 Workspace Architecture

When initialized, `sudm` sets up a predictable, modular file directory layout inside your project workspace. This keeps your tables, logic, and side-effects separated:

```text
my-project/
├── migrations/
│   ├── schema/       # Table structures and field definitions (SurrealQL)
│   ├── function/     # Custom scoped database functions
│   └── event/        # Event-driven triggers and automated handlers
```

---

## 🕹️ Conceptual Workflow

`sudm` manages the lifecycle of your database engine in three simple visual phases:

### 1. Structure Scaffolding
Deploy a localized folder hierarchy instantly to map out your codebase logic safely on disk without needing network connections.

### 2. Boilerplate Generation
Generate chronologically timestamped files (e.g., `20260924_user.surql`) pre-configured with explicit starting templates, preventing migration name collisions.

### 3. Ledger Synchronization
Connect to your SurrealDB instance over dynamic protocols (`ws://`, `wss://`, `http://`, `https://`). It compares local timestamps against your remote server and cleanly migrates transactions up or rolls them back down.

---

## ⚡ Built With

* **Rust & Tokio:** Fully asynchronous execution profiles for lightning-fast database updates and disk management.
* **Clap (v4):** Human-centric CLI parsing engine with native help screens and automated environment variable integration.
* **SurrealDB Rust SDK:** Powered natively by the official dynamic dynamic-engine infrastructure.
