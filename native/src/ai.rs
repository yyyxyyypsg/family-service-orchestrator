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
}

pub fn answer(doc: &mut OrchestratorDocument, call: &ServiceCall) -> ToolResult {
    match call.tool.as_str() {
        "current_state" => {
            let data = json::obj(vec![
                ("state", json::s(doc.state.label())),
                ("summary", json::s(engine::state_summary(doc))),
            ]).to_json();
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
            ]).to_json();
            ToolResult::ok(&call.call_id, doc.message.clone(), "").with_data(data)
        }
        "confirm_plan" => {
            engine::confirm(doc);
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
        ServiceCall { call_id: "c1".into(), tool: tool.into(), args: args.into() }
    }

    #[test]
    fn manifest_has_read_parse_and_confirm_tools() {
        let m = manifest();
        assert_eq!(m.id, "family_orchestrator");
        assert!(m.validate().is_ok());
        assert_eq!(m.tools.len(), 3);
    }

    #[test]
    fn agent_can_parse_and_confirm_without_bypassing_the_state_machine() {
        let mut doc = OrchestratorDocument::default();
        let parsed = answer(&mut doc, &call("parse_notice", r#"{"notice":"订单号：AC-1\n配送预计：9月28日\n安装预约：9月27日 15:00\n配送延迟"}"#));
        assert_eq!(parsed.outcome, ToolOutcome::Ok);
        assert_eq!(doc.state.label(), "AwaitingConfirm");
        let confirmed = answer(&mut doc, &call("confirm_plan", "{}"));
        assert_eq!(confirmed.outcome, ToolOutcome::Ok);
        assert_eq!(doc.state.label(), "Following");
    }
}
