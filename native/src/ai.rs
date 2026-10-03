//! The native service contract exposed to the OctoSense assistant bus.

use crate::{engine, model::OrchestratorDocument, parser};
use makepad_ai_services::wire::{Risk, ServiceCall, ServiceManifest, ToolDef, ToolResult};
use makepad_strict_json::{self as json, Value};

pub fn manifest() -> ServiceManifest {
    ServiceManifest::new(
        "family_orchestrator",
        "家庭服务事件编排器",
        "解析配送/安装通知，给出带原文依据的本地服务方案，并在确认后执行和核验。",
    )
    .with_tool(ToolDef::new(
        "current_state",
        "Read the current service state, facts, and audit summary.",
        r#"{"type":"object","properties":{},"additionalProperties":false}"#,
        Risk::Read,
    ))
    .with_tool(ToolDef::new(
        "parse_notice",
        "Parse user supplied notification text. Missing fields are returned instead of guessed.",
        r#"{"type":"object","properties":{"notice":{"type":"string","maxLength":8192}},"required":["notice"],"additionalProperties":false}"#,
        Risk::Act,
    ))
    .with_tool(ToolDef::new(
        "confirm_plan",
        "Apply the currently selected local plan. The app records an audit entry and refuses duplicates.",
        r#"{"type":"object","properties":{},"additionalProperties":false}"#,
        Risk::Destructive,
    ))
    .with_tool(ToolDef::new(
        "merge_notice",
        "Merge a later delivery or service update into the existing case and re-plan it.",
        r#"{"type":"object","properties":{"notice":{"type":"string","maxLength":8192}},"required":["notice"],"additionalProperties":false}"#,
        Risk::Act,
    ))
    .with_tool(ToolDef::new(
        "create_checklist",
        "Return the preparation checklist for the current family service type.",
        r#"{"type":"object","properties":{},"additionalProperties":false}"#,
        Risk::Act,
    ))
    .with_tool(ToolDef::new(
        "set_recheck",
        "Register a local after-sale recheck reminder. It does not contact an external provider.",
        r#"{"type":"object","properties":{"when":{"type":"string","maxLength":128}},"additionalProperties":false}"#,
        Risk::Act,
    ))
    .with_tool(ToolDef::new(
        "mark_accepted",
        "Record that the user accepted the completed service and enter after-sale follow-up.",
        r#"{"type":"object","properties":{},"additionalProperties":false}"#,
        Risk::Destructive,
    ))
    .with_tool(ToolDef::new(
        "record_service_issue",
        "Record a user-reported service problem for follow-up.",
        r#"{"type":"object","properties":{"issue":{"type":"string","maxLength":512}},"required":["issue"],"additionalProperties":false}"#,
        Risk::Act,
    ))
}

pub fn answer(doc: &mut OrchestratorDocument, call: &ServiceCall) -> ToolResult {
    // Peer-link tools are namespaced by the host (`family-orchestrator.*`),
    // while the local AI bus manifest keeps the short names. Accept both
    // forms so the same executor serves the native peer and in-process bus.
    let tool = call.tool.rsplit('.').next().unwrap_or(call.tool.as_str());
    match tool {
        "current_state" => {
            let data = json::obj(vec![
                ("state", json::s(doc.state.label())),
                ("summary", json::s(engine::state_summary(doc))),
            ])
            .to_json();
            ToolResult::ok(&call.call_id, engine::state_summary(doc), "").with_data(data)
        }
        "parse_notice" => {
            let Ok(value) = json::parse(call.args.as_bytes()) else {
                return ToolResult::refused(&call.call_id, "invalid JSON arguments");
            };
            let Some(notice) = value.get("notice").and_then(|v| v.as_str()) else {
                return ToolResult::refused(&call.call_id, "notice must be a string");
            };
            doc.notice = notice.to_string();
            engine::parse_and_plan(doc);
            let parsed = parser::parse_notice(notice, "agent_notice");
            let data = json::obj(vec![
                ("state", json::s(doc.state.label())),
                ("facts", Value::Int(parsed.facts.len() as i64)),
                ("missing", Value::Int(parsed.missing.len() as i64)),
            ])
            .to_json();
            ToolResult::ok(&call.call_id, doc.message.clone(), "").with_data(data)
        }
        "confirm_plan" => {
            engine::confirm(doc);
            ToolResult::ok(&call.call_id, doc.message.clone(), "")
        }
        "merge_notice" => {
            let Ok(value) = json::parse(call.args.as_bytes()) else {
                return ToolResult::refused(&call.call_id, "invalid JSON arguments");
            };
            let Some(notice) = value.get("notice").and_then(|v| v.as_str()) else {
                return ToolResult::refused(&call.call_id, "notice must be a string");
            };
            engine::merge_notice(doc, notice);
            ToolResult::ok(&call.call_id, doc.message.clone(), "").with_data(
                json::obj(vec![
                    ("state", json::s(doc.state.label())),
                    ("summary", json::s(engine::state_summary(doc))),
                ])
                .to_json(),
            )
        }
        "create_checklist" => {
            let checklist = doc
                .checklist
                .iter()
                .map(|item| format!("{}:{}", item.status, item.title))
                .collect::<Vec<_>>()
                .join(" | ");
            ToolResult::ok(&call.call_id, checklist.clone(), "").with_data(
                json::obj(vec![
                    ("service_type", json::s(doc.service_type.clone())),
                    ("checklist", json::s(checklist)),
                ])
                .to_json(),
            )
        }
        "set_recheck" => {
            let Ok(value) = json::parse(call.args.as_bytes()) else {
                return ToolResult::refused(&call.call_id, "invalid JSON arguments");
            };
            let when = value.get("when").and_then(|v| v.as_str());
            engine::set_recheck(doc, when);
            ToolResult::ok(&call.call_id, doc.message.clone(), "")
        }
        "mark_accepted" => {
            engine::mark_accepted(doc);
            ToolResult::ok(&call.call_id, doc.message.clone(), "")
        }
        "record_service_issue" => {
            let Ok(value) = json::parse(call.args.as_bytes()) else {
                return ToolResult::refused(&call.call_id, "invalid JSON arguments");
            };
            let Some(issue) = value.get("issue").and_then(|v| v.as_str()) else {
                return ToolResult::refused(&call.call_id, "issue must be a string");
            };
            engine::record_issue(doc, issue);
            ToolResult::ok(&call.call_id, doc.message.clone(), "")
        }
        other => ToolResult::refused(&call.call_id, format!("unknown tool `{other}`")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makepad_ai_services::wire::ToolOutcome;

    fn call(tool: &str, args: &str) -> ServiceCall {
        ServiceCall {
            call_id: "c1".into(),
            tool: tool.into(),
            args: args.into(),
        }
    }

    #[test]
    fn manifest_has_read_parse_and_confirm_tools() {
        let m = manifest();
        assert_eq!(m.id, "family_orchestrator");
        assert!(m.validate().is_ok());
        assert_eq!(m.tools.len(), 8);
    }

    #[test]
    fn agent_can_parse_and_confirm_without_bypassing_the_state_machine() {
        let mut doc = OrchestratorDocument::default();
        let parsed = answer(
            &mut doc,
            &call(
                "parse_notice",
                r#"{"notice":"订单号：AC-1\n配送预计：9月28日\n安装预约：9月27日 15:00\n配送延迟"}"#,
            ),
        );
        assert_eq!(parsed.outcome, ToolOutcome::Ok);
        assert_eq!(doc.state.label(), "AwaitingConfirm");
        let confirmed = answer(&mut doc, &call("confirm_plan", "{}"));
        assert_eq!(confirmed.outcome, ToolOutcome::Ok);
        assert_eq!(doc.state.label(), "Following");
    }

    #[test]
    fn peer_can_merge_an_update_and_enter_after_sale() {
        let mut doc = OrchestratorDocument::default();
        answer(
            &mut doc,
            &call(
                "parse_notice",
                r#"{"notice":"订单号：AC-1\n配送预计：10月3日\n安装预约：10月4日 15:00\n服务类型：空调配送安装"}"#,
            ),
        );
        answer(&mut doc, &call("confirm_plan", "{}"));
        let update = answer(
            &mut doc,
            &call(
                "merge_notice",
                r#"{"notice":"安装预约：10月5日 10:00\n配送延迟"}"#,
            ),
        );
        assert_eq!(update.outcome, ToolOutcome::Ok);
        assert_eq!(doc.state.label(), "AwaitingConfirm");
        answer(&mut doc, &call("confirm_plan", "{}"));
        let accepted = answer(&mut doc, &call("mark_accepted", "{}"));
        assert_eq!(accepted.outcome, ToolOutcome::Ok);
        assert_eq!(doc.state.label(), "PostSale");
    }
}
