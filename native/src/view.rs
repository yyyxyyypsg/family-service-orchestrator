//! Small native UI surface. The state machine lives in `engine`; this widget
//! only projects it into Makepad controls and forwards user actions.

use crate::{engine, model::OrchestratorDocument};
use makepad_ai_services::peer::{OctosPeer, PeerEvent};
use makepad_ai_services::wire::{ServiceCall, ToolOutcome};
use makepad_widgets::makepad_platform::storage::StorageHandle;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    let Action = Button{
        width: Fill height: 42 margin: 0 padding: Inset{left: 12 right: 12}
        draw_bg +: {color: theme.color_inset color_hover: theme.color_inset_hover color_down: theme.color_inset_down border_radius: 10}
        draw_text +: {color: theme.color_text text_style: theme.font_regular{font_size: 13}}
    }

    let FactLine = Label{
        width: Fill height: Fit padding: 0 margin: Inset{top: 2 bottom: 2}
        draw_text +: {color: theme.color_text_meta text_style: theme.font_regular{font_size: 11}}
    }

    let Panel = RoundedView{
        width: Fill height: Fit padding: 12 margin: Inset{top: 8}
        draw_bg +: {color: theme.color_bg_container border_radius: 12 border_size: 0}
    }

    mod.family_orchestrator = {}
    mod.family_orchestrator.ink = theme.color_text
    mod.family_orchestrator.muted = theme.color_text_meta
    mod.family_orchestrator.accent = theme.color_focus
    mod.family_orchestrator.success = theme.color_success

    mod.widgets.FamilyViewBase = #(FamilyView::register_widget(vm))
    mod.widgets.FamilyView = set_type_default() do mod.widgets.FamilyViewBase{
        width: Fill height: Fill flow: Down padding: 16 spacing: 8
        header := View{
            width: Fill height: 58 flow: Down spacing: 4
            title := Label{text: "家庭服务事件编排器" padding: 0 draw_text +: {color: mod.family_orchestrator.ink text_style: theme.font_bold{font_size: 22}}}
            state := Label{text: "Idle · 等待通知" padding: 0 draw_text +: {color: mod.family_orchestrator.muted text_style: theme.font_regular{font_size: 12}}}
        }
        input_panel := Panel{
            notice := TextInput{width: Fill height: 86 empty_text: "粘贴配送/安装通知" margin: 0}
            actions := View{width: Fill height: 42 flow: Right spacing: 8 margin: Inset{top: 8}
                sample := Action{width: 120 text: "填入样例"}
                parse := Action{text: "解析通知"}
            }
        }
        facts_panel := Panel{
            heading := Label{text: "事实（可回溯原文）" padding: 0 draw_text +: {color: mod.family_orchestrator.ink text_style: theme.font_bold{font_size: 13}}}
            facts := Label{width: Fill height: Fit padding: 0 draw_text +: {color: mod.family_orchestrator.muted text_style: theme.font_regular{font_size: 11}}}
        }
        plan_panel := Panel{
            heading := Label{text: "方案" padding: 0 draw_text +: {color: mod.family_orchestrator.ink text_style: theme.font_bold{font_size: 13}}}
            plan := Label{width: Fill height: Fit padding: 0 draw_text +: {color: mod.family_orchestrator.muted text_style: theme.font_regular{font_size: 11}}}
            confirm := Action{text: "确认执行" margin: Inset{top: 8}}
        }
        footer := Label{width: Fill height: Fit padding: 0 draw_text +: {color: mod.family_orchestrator.muted text_style: theme.font_regular{font_size: 11}}}
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct FamilyView {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[rust]
    doc: OrchestratorDocument,
    #[rust]
    storage: Option<StorageHandle>,
    #[rust]
    peer: Option<OctosPeer>,
}

impl FamilyView {
    pub fn set_storage(&mut self, storage: StorageHandle) {
        self.storage = Some(storage);
    }

    pub fn document(&self) -> &OrchestratorDocument {
        &self.doc
    }

    pub fn ai_summary(&self) -> String {
        engine::state_summary(&self.doc)
    }

    pub fn ai_answer(&mut self, call: &makepad_ai_services::wire::ServiceCall) -> makepad_ai_services::wire::ToolResult {
        crate::ai::answer(&mut self.doc, call)
    }

    /// Open the app's Octos peer once. The first session request causes the
    /// host broker to prepare the peer and publish its slug to the system
    /// agent; later tool calls arrive on the same in-process link.
    fn ensure_peer(&mut self, cx: &mut Cx) {
        if self.peer.is_none() {
            let mut peer = OctosPeer::open(cx);
            peer.open_session(None);
            log!("family-orchestrator: opened OctosPeer and requested session");
            self.peer = Some(peer);
        }
    }

    fn drain_peer(&mut self, cx: &mut Cx, event: &Event) {
        self.ensure_peer(cx);
        let Some(mut peer) = self.peer.take() else { return };
        for incoming in peer.handle_event(cx, event) {
            match incoming {
                PeerEvent::Reply { req_id, result } => {
                    log!("family-orchestrator: peer request {req_id} reply: {:?}", result);
                }
                PeerEvent::ToolCall(call) => {
                    let tool_name = call.name.clone();
                    log!("family-orchestrator: peer tool call {}", tool_name);
                    let call_id = call.call_id.clone();
                    let service_call = ServiceCall {
                        call_id: call_id.clone(),
                        tool: call.name,
                        args: call.args.to_json(),
                    };
                    let result = self.ai_answer(&service_call);
                    let outcome = if result.outcome == ToolOutcome::Ok {
                        let value = if result.data.is_empty() {
                            makepad_strict_json::obj(vec![("text", makepad_strict_json::s(result.text))])
                        } else {
                            makepad_strict_json::parse(result.data.as_bytes()).unwrap_or_else(|_| {
                                makepad_strict_json::obj(vec![("text", makepad_strict_json::s(result.text))])
                            })
                        };
                        Ok(value)
                    } else {
                        Err(result.text)
                    };
                    log!("family-orchestrator: peer tool result {} -> {:?}", tool_name, outcome);
                    let _ = peer.tool_result(&call_id, outcome);
                }
                _ => {}
            }
        }
        self.peer = Some(peer);
    }

    fn refresh(&mut self, cx: &mut Cx) {
        self.view.label(cx, ids!(header.state)).set_text(cx, &format!("{} · {}", self.doc.state.label(), self.doc.message));
        let facts = if self.doc.parsed.facts.is_empty() {
            if self.doc.parsed.missing.is_empty() { "尚未解析".to_string() } else { format!("缺少：{}", self.doc.parsed.missing.join("、")) }
        } else {
            self.doc.parsed.facts.iter().map(|f| format!("{} = {} 〔{}〕", f.field, f.value, f.source_quote)).collect::<Vec<_>>().join("\n")
        };
        self.view.label(cx, ids!(facts_panel.facts)).set_text(cx, &facts);
        let plan = self.doc.plans.get(self.doc.selected_plan).map(|p| format!("{}\n{}", p.title, p.detail)).unwrap_or_else(|| "等待解析后生成方案".into());
        self.view.label(cx, ids!(plan_panel.plan)).set_text(cx, &plan);
        self.view.label(cx, ids!(footer)).set_text(cx, &format!("重规划 {} · 审计 {} 条", self.doc.replan_count, self.doc.audit.len()));
        self.view.button(cx, ids!(plan_panel.confirm)).set_enabled(cx, matches!(self.doc.state, crate::model::ServiceState::AwaitingConfirm | crate::model::ServiceState::Partial));
    }

    fn fill_sample(&mut self, cx: &mut Cx) {
        let sample = "订单号：AC-20261002\n配送预计：10月3日（配送延迟）\n安装预约：10月4日 15:00\n安装地址：深圳市南山区科苑路1号";
        self.view.text_input(cx, ids!(input_panel.notice)).set_text(cx, sample);
    }
}

impl Widget for FamilyView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.drain_peer(cx, event);
        self.refresh(cx);
        self.view.handle_event(cx, event, scope);
        if let Event::Actions(actions) = event {
            if self.view.button(cx, ids!(input_panel.actions.sample)).clicked(actions) {
                self.fill_sample(cx);
            }
            if self.view.button(cx, ids!(input_panel.actions.parse)).clicked(actions) {
                self.doc.notice = self.view.text_input(cx, ids!(input_panel.notice)).text();
                engine::parse_and_plan(&mut self.doc);
            }
            if self.view.button(cx, ids!(plan_panel.confirm)).clicked(actions) {
                engine::confirm(&mut self.doc);
            }
            self.refresh(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
