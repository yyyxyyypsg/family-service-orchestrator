//! Small native UI surface. The state machine lives in `engine`; this widget
//! only projects it into Makepad controls and forwards user actions.

use crate::{engine, model::OrchestratorDocument};
use makepad_ai_services::peer::{OctosPeer, PeerEvent};
use makepad_ai_services::wire::{ServiceCall, ToolOutcome, ToolResult};
use makepad_widgets::makepad_platform::storage::StorageHandle;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    let Action = Button{
        width: Fill height: 32 margin: 0 padding: Inset{left: 8 right: 8}
        draw_bg +: {color: #EDF2F8 color_hover: #DCE9F7 color_down: #C9DDF2 border_radius: 8 border_size: 1 border_color: #C8D6E5}
        draw_text +: {color: #26445D text_style: theme.font_regular{font_size: 10}}
    }

    let PrimaryAction = Action{
        draw_bg +: {color: #3F67F5 color_hover: #3156D6 color_down: #2949B8 border_color: #3F67F5}
        draw_text +: {color: #FFFFFF text_style: theme.font_bold{font_size: 10}}
    }

    let SuccessAction = Action{
        draw_bg +: {color: #DDF5EC color_hover: #C6EBDD color_down: #AEE0CC border_color: #A9DCC9}
        draw_text +: {color: #167454 text_style: theme.font_bold{font_size: 10}}
    }

    let WarningAction = Action{
        draw_bg +: {color: #FFF0DF color_hover: #FFE2C3 color_down: #FFD3A2 border_color: #F2C28E}
        draw_text +: {color: #A65B1E text_style: theme.font_bold{font_size: 10}}
    }

    let MetricCard = RoundedView{
        width: Fill height: 42 padding: Inset{left: 10 right: 10 top: 6 bottom: 6}
        draw_bg +: {color: #FFFFFF border_radius: 9 border_size: 1 border_color: #D6E1ED}
    }

    let MetricValue = Label{
        width: Fill height: 18 padding: 0
        draw_text +: {color: #19324A text_style: theme.font_bold{font_size: 12}}
    }

    let FactLine = Label{
        width: Fill height: Fit padding: 0 margin: Inset{top: 2 bottom: 2}
        draw_text +: {color: #58728B text_style: theme.font_regular{font_size: 11}}
    }

    let Panel = RoundedView{
        width: Fill height: Fit padding: 12 margin: 0
        draw_bg +: {color: #FFFFFF border_radius: 12 border_size: 1 border_color: #D6E1ED}
    }

    let EvidencePanel = Panel{
        draw_bg +: {color: #F4F8FF border_color: #C7D8F2}
    }

    let CasePanel = Panel{
        draw_bg +: {color: #FBF9FF border_color: #D9D0F1}
    }

    let PlanPanel = Panel{
        draw_bg +: {color: #F3F6FF border_color: #C7D5F5}
    }

    let AfterSalePanel = Panel{
        draw_bg +: {color: #FFF9F2 border_color: #F0D5B8}
    }

    mod.family_orchestrator = {}
    mod.family_orchestrator.ink = #19324A
    mod.family_orchestrator.muted = #668096
    mod.family_orchestrator.accent = #3F67F5
    mod.family_orchestrator.success = #159A72
    mod.family_orchestrator.header = #183B56
    mod.family_orchestrator.header_muted = #BBD1E1
    mod.family_orchestrator.warning = #D97724

    mod.widgets.FamilyViewBase = #(FamilyView::register_widget(vm))
    mod.widgets.FamilyView = set_type_default() do mod.widgets.FamilyViewBase{
        width: Fill height: Fill flow: Down padding: 12 spacing: 6
        draw_bg +: {color: #F3F7FB}
        header := RoundedView{
            width: Fill height: 54 padding: Inset{left: 14 right: 12} flow: Right spacing: 8
            draw_bg +: {color: mod.family_orchestrator.header border_radius: 12 border_size: 1 border_color: #102D45}
            brand := View{width: Fill height: 48 flow: Down spacing: 2
                title := Label{text: "家庭服务事件编排器" padding: 0 draw_text +: {color: #FFFFFF text_style: theme.font_bold{font_size: 20}}}
                subtitle := Label{text: "把配送、安装、验收和售后串成一条可核验的服务链" padding: 0 draw_text +: {color: mod.family_orchestrator.header_muted text_style: theme.font_regular{font_size: 10}}}
            }
            status_pill := RoundedView{width: 260 height: 36 padding: 8
                draw_bg +: {color: #E6F5F2 border_radius: 10 border_size: 1 border_color: #75CDBD}
                state := Label{text: "Idle · 等待通知" width: Fill height: Fill padding: 0 draw_text +: {color: #087F75 text_style: theme.font_bold{font_size: 10}}}
            }
        }
        summary := View{width: Fill height: 42 flow: Right spacing: 6
            stage := MetricCard{
                caption := Label{text: "当前阶段" padding: 0 draw_text +: {color: #668096 text_style: theme.font_regular{font_size: 8}}}
                value := MetricValue{text: "Idle"}
            }
            evidence := MetricCard{
                caption := Label{text: "事实证据" padding: 0 draw_text +: {color: #668096 text_style: theme.font_regular{font_size: 8}}}
                value := MetricValue{text: "0 条"}
            }
            progress := MetricCard{
                caption := Label{text: "链路进度" padding: 0 draw_text +: {color: #668096 text_style: theme.font_regular{font_size: 8}}}
                value := MetricValue{text: "0%"}
            }
            risk := MetricCard{
                caption := Label{text: "风险与待办" padding: 0 draw_text +: {color: #668096 text_style: theme.font_regular{font_size: 8}}}
                value := MetricValue{text: "等待通知"}
            }
        }
        input_panel := Panel{flow: Down height: 126
            heading := Label{text: "输入服务通知" padding: 0 draw_text +: {color: mod.family_orchestrator.ink text_style: theme.font_bold{font_size: 11}}}
            notice := TextInput{width: Fill height: 48 empty_text: "粘贴配送 / 安装 / 改期通知，应用只使用文本中出现的事实" margin: Inset{top: 6}}
            actions := View{width: Fill height: 30 flow: Right spacing: 6 margin: Inset{top: 6}
                sample := Action{width: 108 text: "切换场景样例"}
                merge := Action{width: 98 text: "合并更新"}
                update := Action{width: 108 text: "填入延期更新"}
                parse := PrimaryAction{width: Fill text: "解析并生成方案"}
            }
        }
        columns := View{width: Fill height: 332 flow: Right spacing: 8
            left := View{width: Fill height: Fill flow: Down spacing: 8
                facts_panel := EvidencePanel{flow: Down height: 82
                    heading := Label{text: "事实证据 · 可回溯原文" padding: 0 draw_text +: {color: mod.family_orchestrator.ink text_style: theme.font_bold{font_size: 11}}}
                    facts := Label{width: Fill height: 48 padding: 0 draw_text +: {color: mod.family_orchestrator.muted text_style: theme.font_regular{font_size: 9}}}
                }
                case_panel := CasePanel{flow: Down height: 242
                    heading := Label{text: "服务案件 · 当前编排进度" padding: 0 draw_text +: {color: mod.family_orchestrator.ink text_style: theme.font_bold{font_size: 11}}}
                    case := Label{width: Fill height: 24 padding: 0 draw_text +: {color: mod.family_orchestrator.muted text_style: theme.font_regular{font_size: 10}}}
                    timeline_caption := Label{text: "服务时间线" padding: 0 margin: Inset{top: 8} draw_text +: {color: mod.family_orchestrator.muted text_style: theme.font_bold{font_size: 9}}}
                    timeline := Label{width: Fill height: 36 padding: 0 draw_text +: {color: mod.family_orchestrator.accent text_style: theme.font_regular{font_size: 10}}}
                    checklist_caption := Label{text: "准备清单" padding: 0 margin: Inset{top: 8} draw_text +: {color: mod.family_orchestrator.muted text_style: theme.font_bold{font_size: 9}}}
                    checklist := Label{width: Fill height: 90 padding: 0 draw_text +: {color: mod.family_orchestrator.muted text_style: theme.font_regular{font_size: 9}}}
                }
            }
            right := View{width: 382 height: Fill flow: Down spacing: 8
                plan_panel := PlanPanel{flow: Down height: 154
                    heading := Label{text: "下一步方案" padding: 0 draw_text +: {color: mod.family_orchestrator.ink text_style: theme.font_bold{font_size: 11}}}
                    plan := Label{width: Fill height: 58 padding: 0 margin: Inset{top: 8} draw_text +: {color: mod.family_orchestrator.muted text_style: theme.font_regular{font_size: 10}}}
                    actions := View{width: Fill height: 30 flow: Right spacing: 6 margin: Inset{top: 8}
                        cycle := Action{width: 98 text: "切换方案"}
                        confirm := PrimaryAction{width: Fill text: "确认执行"}
                    }
                }
                after_sale_panel := AfterSalePanel{flow: Down height: 170
                    heading := Label{text: "验收与售后" padding: 0 draw_text +: {color: mod.family_orchestrator.ink text_style: theme.font_bold{font_size: 11}}}
                    status := Label{width: Fill height: 54 padding: 0 margin: Inset{top: 8} draw_text +: {color: mod.family_orchestrator.muted text_style: theme.font_regular{font_size: 9}}}
                    issue_input := TextInput{width: Fill height: 24 empty_text: "输入售后问题，例如：安装后有异响" margin: Inset{top: 4}}
                    actions := View{width: Fill height: 30 flow: Right spacing: 6 margin: Inset{top: 8}
                        accepted := SuccessAction{width: Fill text: "完成验收"}
                        issue := WarningAction{width: Fill text: "记录问题"}
                        recheck := Action{width: Fill text: "登记复查"}
                    }
                }
            }
        }
        footer := Label{width: Fill height: 16 padding: 0 draw_text +: {color: mod.family_orchestrator.muted text_style: theme.font_regular{font_size: 9}}}
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
    #[rust]
    sample_variant: usize,
    #[rust]
    agent_confirmation_pending: bool,
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

    pub fn ai_answer(
        &mut self,
        call: &makepad_ai_services::wire::ServiceCall,
    ) -> makepad_ai_services::wire::ToolResult {
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
        let Some(mut peer) = self.peer.take() else {
            return;
        };
        for incoming in peer.handle_event(cx, event) {
            match incoming {
                PeerEvent::Reply { req_id, result } => {
                    log!(
                        "family-orchestrator: peer request {req_id} reply: {:?}",
                        result
                    );
                }
                PeerEvent::ToolCall(call) => {
                    let tool_name = call.name.clone();
                    log!("family-orchestrator: peer tool call {}", tool_name);
                    let call_id = call.call_id.clone();
                    let short_name = tool_name.rsplit('.').next().unwrap_or(tool_name.as_str());
                    // A peer may propose a destructive action, but it cannot
                    // self-approve it. The visible button is the only path
                    // that calls engine::confirm and changes the timeline.
                    let result = if short_name == "confirm_plan" {
                        self.agent_confirmation_pending = true;
                        self.doc.message = "Agent 请求执行，请在应用内点击“确认执行”".into();
                        ToolResult::ok(&call_id, self.doc.message.clone(), "")
                            .with_data(makepad_strict_json::obj(vec![
                                ("state", makepad_strict_json::s(self.doc.state.label())),
                                ("awaiting_user_confirmation", makepad_strict_json::Value::Bool(true)),
                            ]).to_json())
                    } else {
                        let service_call = ServiceCall {
                            call_id: call_id.clone(),
                            tool: call.name,
                            args: call.args.to_json(),
                        };
                        self.ai_answer(&service_call)
                    };
                    let outcome = if result.outcome == ToolOutcome::Ok {
                        let value = if result.data.is_empty() {
                            makepad_strict_json::obj(vec![(
                                "text",
                                makepad_strict_json::s(result.text),
                            )])
                        } else {
                            makepad_strict_json::parse(result.data.as_bytes()).unwrap_or_else(
                                |_| {
                                    makepad_strict_json::obj(vec![(
                                        "text",
                                        makepad_strict_json::s(result.text),
                                    )])
                                },
                            )
                        };
                        Ok(value)
                    } else {
                        Err(result.text)
                    };
                    log!(
                        "family-orchestrator: peer tool result {} -> {:?}",
                        tool_name,
                        outcome
                    );
                    let _ = peer.tool_result(&call_id, outcome);
                }
                _ => {}
            }
        }
        self.peer = Some(peer);
    }

    fn refresh(&mut self, cx: &mut Cx) {
        let status_hint = match self.doc.state {
            crate::model::ServiceState::NeedInfo => "需要补充信息",
            crate::model::ServiceState::AwaitingConfirm => "等待用户确认",
            crate::model::ServiceState::Following => "执行后持续跟进",
            crate::model::ServiceState::Partial => "部分成功，等待重试",
            crate::model::ServiceState::PostSale => "售后复查中",
            crate::model::ServiceState::ParseFailed => "解析失败",
            _ => self.doc.message.as_str(),
        };
        self.view.label(cx, ids!(header.status_pill.state)).set_text(
            cx,
            &format!("{} · {}", self.doc.state.label(), status_hint),
        );
        self.view
            .label(cx, ids!(summary.stage.value))
            .set_text(cx, self.doc.state.label());
        self.view
            .label(cx, ids!(summary.evidence.value))
            .set_text(cx, &format!("{} 条", self.doc.parsed.facts.len()));
        self.view
            .label(cx, ids!(summary.progress.value))
            .set_text(cx, &format!("{}%", engine::completion_percent(&self.doc)));
        self.view
            .label(cx, ids!(summary.risk.value))
            .set_text(cx, &engine::risk_summary(&self.doc));
        let facts = if self.doc.parsed.facts.is_empty() {
            if self.doc.parsed.missing.is_empty() {
                "尚未解析".to_string()
            } else {
                format!("缺少：{}", self.doc.parsed.missing.join("、"))
            }
        } else {
            self.doc
                .parsed
                .facts
                .iter()
                .map(|f| format!("{} = {} 〔{}〕", f.field, f.value, f.source_quote))
                .collect::<Vec<_>>()
                .join("\n")
        };
        self.view
            .label(cx, ids!(columns.left.facts_panel.facts))
            .set_text(cx, &facts);
        let plan = self
            .doc
            .plans
            .get(self.doc.selected_plan)
            .map(|p| format!("{}\n{}", p.title, p.detail))
            .unwrap_or_else(|| "等待解析后生成方案".into());
        self.view
            .label(cx, ids!(columns.right.plan_panel.plan))
            .set_text(cx, &plan);
        let checklist = self
            .doc
            .checklist
            .iter()
            .map(|item| {
                format!(
                    "{} {}",
                    if item.status == "done" { "✓" } else { "○" },
                    item.title
                )
            })
            .collect::<Vec<_>>()
            .join("  ");
        let timeline = self
            .doc
            .timeline
            .iter()
            .map(|step| {
                format!(
                    "{} {}",
                    if step.status == "done" { "●" } else { "○" },
                    step.title
                )
            })
            .collect::<Vec<_>>()
            .join("  →  ");
        self.view.label(cx, ids!(columns.left.case_panel.case)).set_text(
            cx,
            &format!(
                "类型：{} · 保修：{}",
                self.doc.service_type, self.doc.warranty.expires
            ),
        );
        self.view
            .label(cx, ids!(columns.left.case_panel.timeline))
            .set_text(cx, &timeline);
        self.view
            .label(cx, ids!(columns.left.case_panel.checklist))
            .set_text(cx, &format!("清单：{}", checklist));
        let latest = self
            .doc
            .audit
            .iter()
            .rev()
            .take(2)
            .map(|entry| format!("{}: {}", entry.action, entry.result))
            .collect::<Vec<_>>()
            .join("\n");
        self.view
            .label(cx, ids!(columns.right.after_sale_panel.status))
            .set_text(
                cx,
                &format!(
                    "{}\n复查：{}{}{}",
                    self.doc.message,
                    self.doc.follow_up,
                    if self.doc.service_issue.is_empty() {
                        String::new()
                    } else {
                        format!("\n问题：{}", self.doc.service_issue)
                    },
                    if latest.is_empty() {
                        String::new()
                    } else {
                        format!("\n最近动作：{}", latest)
                    }
                ),
            );
        self.view.label(cx, ids!(footer)).set_text(
            cx,
            &format!(
                "重规划 {} · 审计 {} 条",
                self.doc.replan_count,
                self.doc.audit.len()
            ),
        );
        self.view
            .button(cx, ids!(columns.right.plan_panel.actions.confirm))
            .set_enabled(
                cx,
                matches!(
                    self.doc.state,
                    crate::model::ServiceState::AwaitingConfirm
                        | crate::model::ServiceState::Partial
                ),
            );
        self.view
            .button(cx, ids!(columns.right.plan_panel.actions.cycle))
            .set_enabled(cx, !self.doc.plans.is_empty());
        self.view
            .button(cx, ids!(columns.right.after_sale_panel.actions.accepted))
            .set_enabled(
                cx,
                matches!(
                    self.doc.state,
                    crate::model::ServiceState::Following | crate::model::ServiceState::Completed
                ),
            );
        self.view
            .button(cx, ids!(columns.right.after_sale_panel.actions.issue))
            .set_enabled(
                cx,
                matches!(
                    self.doc.state,
                    crate::model::ServiceState::Following | crate::model::ServiceState::PostSale
                ),
            );
        self.view
            .button(cx, ids!(columns.right.after_sale_panel.actions.recheck))
            .set_enabled(
                cx,
                matches!(
                    self.doc.state,
                    crate::model::ServiceState::Following
                        | crate::model::ServiceState::Completed
                        | crate::model::ServiceState::PostSale
                ),
            );
    }

    fn fill_sample(&mut self, cx: &mut Cx) {
        let samples = [
            "订单号：AC-20261002\n服务类型：空调配送安装\n配送预计：10月3日（配送延迟）\n安装预约：10月4日 15:00\n安装地址：深圳市南山区科苑路1号",
            "订单号：FN-20261003\n服务类型：家具配送组装\n配送预计：10月5日\n安装预约：10月6日 10:00\n送货地址：深圳市福田区中心路8号",
            "订单号：HS-20261004\n服务类型：家政预约\n配送预计：10月7日\n安装预约：10月7日 14:00\n地址：深圳市南山区后海大道18号",
            "订单号：RP-20261005\n服务类型：家电维修\n配送预计：10月8日\n安装预约：10月8日 09:30\n地址：深圳市宝安区创业路6号\n保修至：2027年10月5日",
        ];
        let sample = samples[self.sample_variant % samples.len()];
        self.sample_variant = (self.sample_variant + 1) % samples.len();
        self.view
            .text_input(cx, ids!(input_panel.notice))
            .set_text(cx, sample);
    }

    fn fill_update(&mut self, cx: &mut Cx) {
        let update = "配送通知：因仓库调度，配送延迟至10月5日\n安装预约：10月6日 10:00\n配送地址：深圳市南山区科苑路1号";
        self.view
            .text_input(cx, ids!(input_panel.notice))
            .set_text(cx, update);
    }
}

impl Widget for FamilyView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.drain_peer(cx, event);
        self.refresh(cx);
        self.view.handle_event(cx, event, scope);
        if let Event::Actions(actions) = event {
            if self
                .view
                .button(cx, ids!(input_panel.actions.sample))
                .clicked(actions)
            {
                self.fill_sample(cx);
            }
            if self
                .view
                .button(cx, ids!(input_panel.actions.parse))
                .clicked(actions)
            {
                self.doc.notice = self.view.text_input(cx, ids!(input_panel.notice)).text();
                engine::parse_and_plan(&mut self.doc);
            }
            if self
                .view
                .button(cx, ids!(input_panel.actions.merge))
                .clicked(actions)
            {
                let notice = self.view.text_input(cx, ids!(input_panel.notice)).text();
                engine::merge_notice(&mut self.doc, &notice);
            }
            if self
                .view
                .button(cx, ids!(input_panel.actions.update))
                .clicked(actions)
            {
                self.fill_update(cx);
            }
            if self
                .view
                .button(cx, ids!(columns.right.plan_panel.actions.cycle))
                .clicked(actions)
            {
                engine::select_next_plan(&mut self.doc);
            }
            if self
                .view
                .button(cx, ids!(columns.right.plan_panel.actions.confirm))
                .clicked(actions)
            {
                self.agent_confirmation_pending = false;
                engine::confirm(&mut self.doc);
            }
            if self
                .view
                .button(cx, ids!(columns.right.after_sale_panel.actions.accepted))
                .clicked(actions)
            {
                engine::mark_accepted(&mut self.doc);
            }
            if self
                .view
                .button(cx, ids!(columns.right.after_sale_panel.actions.issue))
                .clicked(actions)
            {
                let issue = self
                    .view
                    .text_input(cx, ids!(columns.right.after_sale_panel.issue_input))
                    .text();
                engine::record_issue(&mut self.doc, &issue);
            }
            if self
                .view
                .button(cx, ids!(columns.right.after_sale_panel.actions.recheck))
                .clicked(actions)
            {
                engine::set_recheck(&mut self.doc, Some("安装完成后 7 天复查"));
            }
            self.refresh(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
