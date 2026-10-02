//! In-process module contract used by an OctoSense/Makepad host.

use crate::{ai, view::FamilyView};
use makepad_ai_services::wire::{ServiceCall, ServiceManifest};
use makepad_app_module::*;
use makepad_widgets::*;

pub struct FamilyOrchestratorModule;
pub static FAMILY_ORCHESTRATOR_MODULE: FamilyOrchestratorModule = FamilyOrchestratorModule;

impl AppModule for FamilyOrchestratorModule {
    // This id is the host registry key. It must match the native-apps.json
    // entry exactly; the AI service id remains snake_case because the service
    // wire format only accepts [a-z0-9_].
    fn id(&self) -> &'static str { "family-orchestrator" }
    fn label(&self) -> &'static str { "家庭服务事件编排器" }

    fn register(&self, vm: &mut ScriptVm) {
        crate::view::script_mod(vm);
    }

    fn open_schema(&self) -> OpenSchema { OpenSchema::new(1) }

    fn create(&self, vm: &mut ScriptVm, _open: ValidatedOpen, handles: InstanceHandles) -> InstanceParts {
        let value = script_eval!(vm, {
            use mod.prelude.widgets.*
            FamilyView {}
        });
        let root = WidgetRef::script_from_value(vm, value);
        if let Some(mut view) = root.borrow_mut::<FamilyView>() {
            view.set_storage(handles.storage);
        }
        let shutdown_root = root.clone();
        InstanceParts {
            root: root.clone(),
            executor: Box::new(FamilyExecutor { root }),
            shutdown: Box::new(move |_vm| drop(shutdown_root)),
        }
    }

    fn capabilities(&self) -> &'static [&'static str] { &["storage"] }
}

struct FamilyExecutor { root: WidgetRef }

impl ServiceExecutor for FamilyExecutor {
    fn manifest(&self) -> ServiceManifest { ai::manifest() }

    fn execute(&mut self, _cx: &mut Cx, call: &ServiceCall) -> ExecOutcome {
        let result = self.root.borrow_mut::<FamilyView>().map(|mut view| view.ai_answer(call)).unwrap_or_else(|| makepad_ai_services::wire::ToolResult::unavailable(&call.call_id, "family orchestrator is gone"));
        ExecOutcome::Done(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_contract_is_stable() {
        assert_eq!(FAMILY_ORCHESTRATOR_MODULE.id(), "family-orchestrator");
        assert_eq!(FAMILY_ORCHESTRATOR_MODULE.label(), "家庭服务事件编排器");
        assert!(FAMILY_ORCHESTRATOR_MODULE.capabilities().contains(&"storage"));
        assert!(FAMILY_ORCHESTRATOR_MODULE.open_schema().empty_open().is_ok());
    }
}
