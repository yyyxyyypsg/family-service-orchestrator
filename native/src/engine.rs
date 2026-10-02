//! Deterministic state transitions for the native implementation.

use crate::{model::*, parser::parse_notice};

pub fn parse_and_plan(doc: &mut OrchestratorDocument) {
    doc.state = ServiceState::Parsing;
    doc.parsed = parse_notice(&doc.notice, "native_notice");
    doc.plans.clear();
    doc.selected_plan = 0;
    if !doc.parsed.errors.is_empty() {
        doc.state = ServiceState::ParseFailed;
        doc.message = doc.parsed.errors.join("；");
        return;
    }
    if !doc.parsed.missing.is_empty() {
        doc.state = ServiceState::NeedInfo;
        doc.message = format!("还缺少：{}", doc.parsed.missing.join("、"));
        return;
    }
    let delivery = doc.parsed.fact("delivery_date").map(|f| f.value.as_str()).unwrap_or("待定");
    let install = doc.parsed.fact("installation_time").map(|f| f.value.as_str()).unwrap_or("待定");
    doc.plans = vec![
        Plan {
            title: "方案 A · 保留安装时段".into(),
            detail: format!("配送：{delivery}；安装：{install}"),
            source_ids: doc.parsed.facts.iter().map(|f| f.source_id.clone()).collect(),
        },
        Plan {
            title: "方案 B · 顺延并登记复查".into(),
            detail: "更新本地服务时间线，加入一次复查提醒".into(),
            source_ids: doc.parsed.facts.iter().map(|f| f.source_id.clone()).collect(),
        },
    ];
    doc.state = ServiceState::AwaitingConfirm;
    doc.message = "方案已生成，请选择方案并确认执行".into();
}

pub fn confirm(doc: &mut OrchestratorDocument) {
    match doc.state {
        ServiceState::AwaitingConfirm => {
            doc.state = ServiceState::Executing;
            doc.audit.push(AuditEntry { action: "reschedule_installation".into(), result: "success".into() });
            doc.audit.push(AuditEntry { action: "create_reminder".into(), result: "success".into() });
            for step in &mut doc.timeline {
                step.status = "done".into();
            }
            doc.state = ServiceState::Following;
            doc.message = "已更新本地服务时间线并登记复查".into();
        }
        ServiceState::Partial => retry(doc),
        _ => doc.audit.push(AuditEntry { action: "confirm".into(), result: "duplicate_or_invalid_intercepted".into() }),
    }
}

pub fn retry(doc: &mut OrchestratorDocument) {
    if doc.state != ServiceState::Partial {
        return;
    }
    doc.audit.push(AuditEntry { action: "create_reminder".into(), result: "retry_success".into() });
    doc.state = ServiceState::Following;
    doc.message = "已补齐未完成的复查提醒".into();
}

pub fn state_summary(doc: &OrchestratorDocument) -> String {
    format!("state={} replan={} facts={} audit={}", doc.state.label(), doc.replan_count, doc.parsed.facts.len(), doc.audit.len())
}
