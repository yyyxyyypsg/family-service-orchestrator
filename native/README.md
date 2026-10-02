# Native host extension

This crate is the native Rust/Makepad track for the family service
orchestrator. It lives beside `bundle/` so the signed App Hub submission stays
reproducible and unchanged.

The crate exposes the same service in two forms:

- `family-orchestrator-native` is a standalone Makepad window for local
  development and video capture.
- `FAMILY_ORCHESTRATOR_MODULE` implements `makepad_app_module::AppModule`, so
  an OctoSense shell can host it in-process and register its typed tools on the
  Makepad AI services bus.

The service manifest offers `current_state`, `parse_notice`, and
`confirm_plan`. The parser is deterministic: missing order, delivery, or
installation fields are reported as `missing`, and every extracted fact keeps
the exact source quote. The destructive confirmation tool only applies the
local timeline and audit record after the host/user confirmation path.

## Local build

The official checkout layout is optional for the native crate itself because
its Makepad dependencies are pinned git revisions. For an OctoSense host build,
keep the app checkout beside the host repository so the host can patch those
revisions to its reviewed `.sources/makepad` checkout:

```text
<workspace>/makepad/
<workspace>/apps/family-orchestrator/
```

From this directory:

```powershell
cargo check --manifest-path native/Cargo.toml
cargo test --manifest-path native/Cargo.toml
$env:MAKEPAD_REMOTE = "8170"
cargo run --manifest-path native/Cargo.toml -- --phone
```

The last command starts a real Makepad window and exposes the standard remote
surface on `127.0.0.1:8170`. Use `/snap`, `/click`, and `/quit` for a repeatable
smoke test. The checked-in script bundle remains the signed competition
baseline; this native crate is an additional host-extension track.

## OctoSense host integration

An OctoSense shell links native modules from its reviewed `native-apps.json`
manifest. The integration entry should use:

- app id: `family-orchestrator`
- crate: `family-orchestrator-native`
- module: `family_orchestrator_native::FAMILY_ORCHESTRATOR_MODULE`
- binary: `family-orchestrator-native`
- capabilities: `storage`
- agent services: the four `octos.session/turn` methods when the shell grants
  an app-owned agent

The shell, not the app, owns the octos kernel and host token. The module does
not spawn a second kernel or open a raw socket. Once registered in a host build,
the module's executor is routed through the shell's approval, audit, and
Octos peer machinery. This repository does not modify the signed bundle or
claim that the standalone window alone is an Octos host build.
