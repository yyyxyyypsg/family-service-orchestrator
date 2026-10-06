//! Native Makepad/OctoSense implementation of the family service orchestrator.
//!
//! The signed `bundle/` remains the competition baseline. This crate is a
//! parallel native-host extension: the deterministic parser and state machine
//! are Rust code, while `FamilyView` is usable both in a standalone window and
//! as an in-process `AppModule`.

pub use makepad_widgets;

pub mod ai;
pub mod engine;
pub mod model;
pub mod module;
pub mod parser;
pub mod view;

pub use module::{FamilyOrchestratorModule, FAMILY_ORCHESTRATOR_MODULE};
pub use view::FamilyView;

pub fn script_mod(vm: &mut makepad_widgets::ScriptVm) -> makepad_widgets::ScriptValue {
    view::script_mod(vm)
}
