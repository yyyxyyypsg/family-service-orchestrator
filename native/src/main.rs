//! Standalone native window; the same `FamilyView` is also exposed as a module.

pub use makepad_widgets;
use makepad_ai_services::port::{AiServicePort, PortEvent};
use family_orchestrator_native::{ai, view::FamilyView};
use makepad_widgets::*;

app_main!(App);

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.title: "家庭服务事件编排器"
                window.inner_size: vec2(480, 820)
                pass +: {clear_color: theme.color_bg_app}
                body +: {padding: 0 margin: 0 spacing: 0 family := FamilyView{}}
            }
        }
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    ai_port: Option<AiServicePort>,
    #[rust]
    ai_context: String,
}

impl App {
    fn family(&self, cx: &mut Cx) -> WidgetRef { self.ui.widget(cx, ids!(family)) }

    fn refresh_ai_context(&mut self, cx: &mut Cx) {
        let Some(port) = self.ai_port.as_ref() else { return };
        let text = self.family(cx).borrow::<FamilyView>().map(|view| view.ai_summary()).unwrap_or_default();
        if text != self.ai_context {
            self.ai_context = text.clone();
            port.set_context(&text);
        }
    }

    fn drain_ai(&mut self, cx: &mut Cx, event: &Event) {
        let events = match self.ai_port.as_mut() { Some(port) => port.handle_event(cx, event), None => return };
        for event in events {
            match event {
                PortEvent::Registered(endpoint) => log!("family_orchestrator: AI service registered as {}", endpoint.as_str()),
                PortEvent::Call(call) => {
                    let result = self.family(cx).borrow_mut::<FamilyView>().map(|mut view| view.ai_answer(&call)).unwrap_or_else(|| makepad_ai_services::wire::ToolResult::unavailable(&call.call_id, "family orchestrator is gone"));
                    if let Some(port) = self.ai_port.as_ref() { port.reply(result); }
                }
                PortEvent::Cancel { .. } | PortEvent::ChatOpen { .. } | PortEvent::Subscribe { .. } | PortEvent::Unsubscribe { .. } => {}
            }
        }
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        self.ai_port = AiServicePort::open(cx, ai::manifest());
        if let Some(mut view) = self.family(cx).borrow_mut::<FamilyView>() { view.set_storage(cx.storage("family_orchestrator")); }
    }

    fn handle_actions(&mut self, _cx: &mut Cx, _actions: &Actions) {}
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        makepad_widgets::script_mod(vm);
        makepad_wm_theme::apply(vm);
        family_orchestrator_native::view::script_mod(vm);
        #[cfg(feature = "standalone")]
        makepad_aichat::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.drain_ai(cx, event);
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
        self.refresh_ai_context(cx);
    }
}
