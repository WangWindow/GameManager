# Plugin System

GameManager's plugin system is split into two layers:

1. **Engine Plugins**: TOML profiles that define how games are detected and launched.
2. **Integration Extensions**: External tooling connectors (currently Bottles for Wine-based Windows game execution on Linux).

Both layers are intentionally declarative and data-driven. No dynamic library loading, no WASM sandbox, no third-party marketplace. Every extension capability is an explicitly supported Rust trait or a structured TOML file.

---

## Layer 1: Engine Plugins (TOML Profiles)

### 1.1 Location & Discovery

All engine profiles live as `.toml` files in `src-tauri/engines/`. The `EngineRegistry` scans this directory at startup. Each file defines one engine.

Currently shipped engines (9 profiles):

| ID | Name | Category | Priority | Strategy |
|----|------|----------|----------|----------|
| `rpgmakermz` | RPG Maker MZ | nwjs | 1 | nwjs |
| `rpgmakermv` | RPG Maker MV | nwjs | 2 | nwjs |
| `rpgmakervxace` | RPG Maker VX Ace | nwjs | 3 | native |
| `rpgmakervx` | RPG Maker VX | nwjs | 4 | native |
| `unity` | Unity | other | 5 | native |
| `godot` | Godot | other | 6 | native |
| `html` | HTML | nwjs | 7 | nwjs |
| `renpy` | Ren'Py | renpy | 0 | native |
| `other` | Other | other | 99 | native |

### 1.2 TOML Schema

Every engine profile has three top-level sections.

#### `[meta]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique slug (used as filename and internal key) |
| `name` | string | Yes | Human-readable display name |
| `category` | string | No | Grouping key for frontend display; defaults to `"other"` |
| `icon` | string | No | Iconify icon ID; defaults to `"ri:question-line"` |
| `priority` | int | No | Tiebreaker when two engines score equally; lower value wins |
| `description` | string | No | Optional tooltip text for the plugin list UI |
| `skip_scan` | bool | No | If true, excluded from automatic folder scans; manual import only |

#### `[detection]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `min_score` | int | No | Minimum weighted score to declare a match (default 0) |
| `rules` | array | No | List of detection rule entries |

Each rule entry (`[[detection.rules]]`) has:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | Yes | One of: `file_exists`, `dir_exists`, `glob_match`, `has_extension` |
| `path` | string | Depends | Relative file/dir path (for `file_exists`, `dir_exists`) |
| `pattern` | string | Depends | Glob pattern against directory entries (for `glob_match`) |
| `ext` | string | Depends | File extension without dot (for `has_extension`) |
| `weight` | int | No | Score contributed when the rule matches; defaults to 1 |

Detection works by scoring a game directory against all enabled+valid engines. Each matching rule adds its `weight` to the engine's score. Only engines whose total score reaches `min_score` are candidates. Among candidates, the highest score wins. If scores tie, the engine with lower `priority` wins. Confidence is reported as a percentage (`score / 16 * 100`, clamped to 1-100).

#### `[launch]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `strategy` | string | Yes | One of: `native`, `nwjs`, `external` |
| `entry_patterns` | string[] | No | Prioritized list of executable name patterns to find in the game directory |
| `exclude_patterns` | string[] | No | Glob patterns for executables to skip (e.g., crash handlers) |
| `args` | string[] | No | Additional CLI arguments passed to the game process |
| `sandbox_home` | bool | No | If true, isolate the game's HOME directory from the host |
| `preserve_dirs` | string[] | No | Subdirectories to preserve across runs (relative to game dir) |

**Strategy-specific fields:**

| Field | Used by | Description |
|-------|---------|-------------|
| `runtime_id` | `nwjs` | NW.js runtime identifier (e.g., `"nwjs-sdk"`); required for nwjs |
| `program` | `external` | External launcher program name; required for external |
| `program_args_prefix` | `external` | Args injected before the template arguments |
| `required_integration` | `external` | Integration key (e.g., `"bottles"`) that must be enabled |
| `args_template` | `external` | Template with `{exe}` and `{game_dir}` tokens; default: `"{exe}"` |
| `extras` | any | Freeform engine-specific key=value pairs |

### 1.3 Launch Strategies

| Strategy | Description |
|----------|-------------|
| `native` | Spawns the game's primary executable directly from the game directory |
| `nwjs` | Resolves the installed NW.js runtime by `runtime_id`, then runs it with the game directory as argument |
| `external` | Delegates to an external program (e.g., Bottles CLI). Uses `args_template` with `{exe}` and `{game_dir}` substitution |

The registry compiles these strategy names through `build_strategy()` for validation and future use. The current production launcher still routes through `LauncherService`, which mirrors the same strategy concepts while applying sandbox and process-spawn behavior.

### 1.4 Load-Time Validation

`EngineRegistry::load()` validates each profile at startup:

1. Parse TOML into `EngineProfile` struct (via `serde`).
2. Validate each detection rule: required fields present, weight non-negative, rule type known.
3. Validate launch config: strategy known, `runtime_id` present for nwjs, `program` present for external.
4. Compile rules and strategy into runtime trait objects.

On validation failure, the engine is marked `valid = false` and disabled by default. The user sees an "Invalid" badge in the plugin UI and cannot toggle it on. Warnings are collected and returned but do not block other engines.

### 1.5 Enable/Disable Persistence

Per-engine enabled state is stored in SQLite via the settings table with key format `engine.<id>.enabled`. On subsequent launches, the stored state overrides the default (true for valid engines, false for invalid ones).

### 1.6 The Detection Trait (Safety Gate)

The `DetectionContext` trait (in `context.rs`) is the only API engine profiles can use during detection. It exposes four methods:

- `file_exists(relative_path)` -- check for a file
- `dir_exists(relative_path)` -- check for a directory
- `glob_match(pattern)` -- check for a matching filename
- `has_extension(ext)` -- check for files with a given extension

All path operations are relative to the game directory being scanned. The engine TOML cannot access the filesystem directly; it can only declare which detection rules to apply.

---

## Layer 2: Integration Extensions

### 2.1 Architecture

Integration extensions are Rust service modules under `src-tauri/src/service/extension/`. Each integration connects GameManager to an external tool and is optional. The current implementation has one integration: Bottles.

Integrations expose their status via Tauri commands (`get_capabilities`, `get_integration_status`, `set_integration_settings`) and are identified by a string key. The frontend queries available integrations dynamically.

### 2.2 Bottles Integration

Bottles is a Wine prefix manager for Linux. GameManager uses it to run Windows games that cannot run natively.

**Location:** `src-tauri/src/service/extension/bottles.rs`

**Capabilities:**

- Detects whether Bottles is installed (Flatpak or native `bottles-cli`)
- Lists available Wine bottles
- Launches executables inside a specified bottle via `bottles-cli run`
- Supports both absolute executable paths (`-e`) and relative paths (`-p`)

**Platform gating:**

- Full implementation compiles only on `cfg(target_os = "linux")`
- Non-Linux builds get stub methods that return "Bottles is Linux-only"

**Persistence:**

- `bottles_enabled` -- stored in SQLite settings, controls whether the integration is active
- `bottles_default` -- the default bottle to use when launching

**How an engine uses Bottles:**

Set the engine's launch strategy to `external` with `required_integration = "bottles"`. Example TOML snippet:

```toml
[launch]
strategy = "external"
program = "bottles-cli"
program_args_prefix = ["run", "-b"]
args_template = "{bottle} -e {exe}"
required_integration = "bottles"
```

The current Bottles launch path is handled by `LauncherService` and `BottlesService`; the TOML `external` strategy fields document the intended connector shape but are not yet the only production launch path. Treat additional template tokens beyond `{exe}` and `{game_dir}` as future integration work unless the corresponding Rust service implements them.

### 2.3 Integration Trait (Not Yet Extracted)

Currently, integrations are handled via explicit functions (`get_bottles_integration_status`, `set_bottles_integration_settings`) dispatched by string key in `integrations.rs`. There is no formal `Integration` trait yet. A future refactor could extract:

```rust
trait Integration {
    fn key(&self) -> &str;
    fn name(&self) -> &str;
    async fn status(&self, db: &mut Db) -> Result<IntegrationStatus>;
    async fn configure(&self, input: IntegrationSettingsInput, db: &mut Db) -> Result<()>;
}
```

This would make adding new integrations (e.g., Lutris, Heroic) a matter of implementing the trait and registering in a `Vec<Box<dyn Integration>>`. Do not pre-build this until at least one more integration is needed.

### 2.4 Settings Keys

| Key | Purpose |
|-----|---------|
| `bottles_enabled` | Whether the Bottles integration is active |
| `bottles_default` | Default bottle name for game launches |
| `engine.<id>.enabled` | Per-engine enable/disable state |

---

## Adding a New Engine Plugin

1. Create a new `.toml` file in `src-tauri/engines/` (e.g., `myengine.toml`).
2. Fill out the `[meta]`, `[detection]`, and `[launch]` sections using the schema above.
3. Set `[meta].id` to match the filename without extension.
4. Pick `[meta].priority` to control tie-breaking (lower = preferred).
5. Define at least one detection rule so the engine can match games.
6. Rebuild. The `EngineRegistry` scans the directory at startup; no code changes needed.

If the new engine needs a launch strategy beyond `native` / `nwjs` / `external`, you must add a new `LaunchStrategy` implementation in `launch.rs` and register it in `build_strategy()`. This requires a code change.

## What Not to Build (Yet)

- **No WASM plugin sandbox.** Engine profiles are data, not code. All executable logic stays in compiled Rust.
- **No third-party marketplace.** Plugins ship with the application. There is no download, update, or publish mechanism.
- **No dynamic library loading (`dlopen`).** All strategies and rules are statically compiled. Adding a new rule type requires editing `detection.rs`.
- **No hot-reload of TOML files.** Changes take effect on application restart. The registry is loaded once at startup.
- **No formal `Integration` trait.** Keep the string-key dispatch in `integrations.rs` until a second integration creates pressure to generalize.

## Frontend: Plugin Management UI

The plugin panel lives in `src/components/settings/PluginsDialog.tsx`. It queries the backend via `getEngineRegistryDetail` and `getEngineProfileDetail` Tauri commands. Each engine is shown with:

- Name, icon, category
- Rule count and strategy label
- Enable/disable toggle (disabled if invalid)
- Expandable detail panel showing detection rules, launch config, and validation errors

When a plugin is toggled, the UI dispatches `gm:refresh-engines`; engine registry consumers use that event to invalidate their cached registry data.
