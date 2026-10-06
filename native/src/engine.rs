//! Deterministic state transitions for the native implementation.

use crate::{model::*, parser::parse_notice};

fn required_missing(facts: &[Fact]) -> Vec<String> {
    ["order_id", "delivery_date", "installation_time"]
        .iter()
        .filter(|field| !facts.iter().any(|fact| fact.field == **field))
        .map(|field| (*field).into())
        .collect()
}

fn apply_facts(doc: &mut OrchestratorDocument) {
    if let Some(fact) = doc.parsed.fact("service_type") {
        doc.service_type = fact.value.clone();
        doc.checklist = match fact.value.as_str() {
            "家具配送组装" => vec![
                ChecklistItem {
                    title: "确认电梯和入户通道".into(),
                    status: "pending".into(),
                },
                ChecklistItem {
                    title: "清空组装区域并准备工具".into(),
                    status: "pending".into(),
                },
                ChecklistItem {
                    title: "组装后检查稳固性".into(),
                    status: "pending".into(),
                },
            ],
            "家政预约" => vec![
                ChecklistItem {
                    title: "确认服务地址和联系人".into(),
                    status: "pending".into(),
                },
                ChecklistItem {
                    title: "列出本次服务范围".into(),
                    status: "pending".into(),
                },
                ChecklistItem {
                    title: "完成服务验收".into(),
                    status: "pending".into(),
                },
            ],
            "家电维修" => vec![
                ChecklistItem {
                    title: "准备故障现象和设备型号".into(),
                    status: "pending".into(),
                },
                ChecklistItem {
                    title: "确认上门时间和入户条件".into(),
                    status: "pending".into(),
                },
                ChecklistItem {
                    title: "维修后通电并记录结果".into(),
                    status: "pending".into(),
                },
            ],
            _ => doc.checklist.clone(),
        };
    }
    if let Some(fact) = doc.parsed.fact("warranty_expires") {
        doc.warranty.expires = fact.value.clone();
        doc.warranty.status = "registered".into();
    }
}

pub fn parse_and_plan(doc: &mut OrchestratorDocument) {
    doc.state = ServiceState::Parsing;
    doc.parsed = parse_notice(&doc.notice, "native_notice");
    apply_facts(doc);
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
    let delivery = doc
        .parsed
        .fact("delivery_date")
        .map(|f| f.value.as_str())
        .unwrap_or("待定");
    let install = doc
        .parsed
        .fact("installation_time")
        .map(|f| f.value.as_str())
        .unwrap_or("待定");
    doc.plans = vec![
        Plan {
            title: "方案 A · 保留安装时段".into(),
            detail: format!("配送：{delivery}；安装：{install}"),
            source_ids: doc
                .parsed
                .facts
                .iter()
                .map(|f| f.source_id.clone())
                .collect(),
        },
        Plan {
            title: "方案 B · 顺延并登记复查".into(),
            detail: "更新本地服务时间线，加入一次复查提醒".into(),
            source_ids: doc
                .parsed
                .facts
                .iter()
                .map(|f| f.source_id.clone())
                .collect(),
        },
    ];
    doc.state = ServiceState::AwaitingConfirm;
    doc.message = "方案已生成，请选择方案并确认执行".into();
}

/// Merge a later notification into the current service case. New facts replace
/// older values by field, while facts that are absent in the update remain
/// available. This keeps one service case across multiple delivery events.
pub fn merge_notice(doc: &mut OrchestratorDocument, notice: &str) {
    doc.state = ServiceState::Parsing;
    let incoming = parse_notice(notice, "native_update");
    if incoming.facts.is_empty() && !incoming.errors.is_empty() {
        doc.parsed.errors = incoming.errors;
        doc.state = ServiceState::ParseFailed;
        doc.message = "新通知中没有可合并事实".into();
        return;
    }
    for fact in incoming.facts {
        doc.parsed.facts.retain(|old| old.field != fact.field);
        doc.parsed.facts.push(fact);
    }
    doc.parsed.errors.clear();
    doc.parsed.missing = required_missing(&doc.parsed.facts);
    doc.notice = notice.into();
    apply_facts(doc);
    if !doc.parsed.missing.is_empty() {
        doc.state = ServiceState::NeedInfo;
        doc.message = format!("案件仍缺少：{}", doc.parsed.missing.join("、"));
        return;
    }
    doc.plans.clear();
    doc.selected_plan = 0;
    doc.replan_count = doc.replan_count.saturating_add(1);
    doc.plans = vec![
        Plan {
            title: "方案 A · 接受最新安排".into(),
            detail: "将本地服务时间线更新为新通知中的安排".into(),
            source_ids: doc
                .parsed
                .facts
                .iter()
                .map(|f| f.source_id.clone())
                .collect(),
        },
        Plan {
            title: "方案 B · 保留原安排并登记复查".into(),
            detail: "暂不覆盖原时间，提醒用户再次确认服务方".into(),
            source_ids: doc
                .parsed
                .facts
                .iter()
                .map(|f| f.source_id.clone())
                .collect(),
        },
    ];
    doc.state = ServiceState::AwaitingConfirm;
    doc.message = "新通知已合并，请确认更新方案".into();
}

pub fn confirm(doc: &mut OrchestratorDocument) {
    match doc.state {
        ServiceState::AwaitingConfirm => {
            doc.state = ServiceState::Executing;
            doc.audit.push(AuditEntry {
                action: "reschedule_installation".into(),
                result: "success".into(),
            });
            doc.audit.push(AuditEntry {
                action: "create_reminder".into(),
                result: "success".into(),
            });
            for step in doc.timeline.iter_mut().take(2) {
                step.status = "done".into();
            }
            for item in doc.checklist.iter_mut().take(2) {
                item.status = "done".into();
            }
            doc.state = ServiceState::Following;
            doc.message = "已更新本地服务时间线并登记复查".into();
        }
        ServiceState::Partial => retry(doc),
        _ => doc.audit.push(AuditEntry {
            action: "confirm".into(),
            result: "duplicate_or_invalid_intercepted".into(),
        }),
    }
}

/// Select the next generated plan before the user confirms execution.
/// Selection is local and auditable; it never applies a plan by itself.
pub fn select_next_plan(doc: &mut OrchestratorDocument) {
    if doc.plans.is_empty() {
        doc.audit.push(AuditEntry {
            action: "select_plan".into(),
            result: "no_plan_available".into(),
        });
        return;
    }
    doc.selected_plan = (doc.selected_plan + 1) % doc.plans.len();
    doc.audit.push(AuditEntry {
        action: "select_plan".into(),
        result: format!("index={}", doc.selected_plan),
    });
    doc.message = format!("已切换到{}，等待确认", doc.plans[doc.selected_plan].title);
}

pub fn mark_accepted(doc: &mut OrchestratorDocument) {
    if !matches!(
        doc.state,
        ServiceState::Following | ServiceState::Completed | ServiceState::PostSale
    ) {
        doc.audit.push(AuditEntry {
            action: "mark_accepted".into(),
            result: "invalid_state_intercepted".into(),
        });
        return;
    }
    if doc
        .timeline
        .iter()
        .any(|step| step.title == "用户验收" && step.status == "done")
    {
        doc.audit.push(AuditEntry {
            action: "mark_accepted".into(),
            result: "duplicate_or_invalid_intercepted".into(),
        });
        return;
    }
    if let Some(step) = doc
        .timeline
        .iter_mut()
        .find(|step| step.title == "用户验收")
    {
        step.status = "done".into();
    }
    if let Some(item) = doc.checklist.last_mut() {
        item.status = "done".into();
    }
    doc.audit.push(AuditEntry {
        action: "mark_accepted".into(),
        result: "success".into(),
    });
    doc.state = ServiceState::PostSale;
    doc.message = "服务已验收，进入售后复查".into();
}

pub fn set_recheck(doc: &mut OrchestratorDocument, when: Option<&str>) {
    doc.follow_up = when
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("安装完成后复查")
        .into();
    doc.audit.push(AuditEntry {
        action: "set_recheck".into(),
        result: doc.follow_up.clone(),
    });
    doc.message = format!("已登记售后复查：{}", doc.follow_up);
    if doc.state == ServiceState::Completed {
        doc.state = ServiceState::PostSale;
    }
}

pub fn record_issue(doc: &mut OrchestratorDocument, issue: &str) {
    let issue = if issue.trim().is_empty() {
        "用户报告服务问题"
    } else {
        issue
    };
    doc.service_issue = issue.into();
    doc.audit.push(AuditEntry {
        action: "record_service_issue".into(),
        result: issue.into(),
    });
    doc.state = ServiceState::PostSale;
    doc.message = format!("已记录售后问题：{}", issue);
}

pub fn retry(doc: &mut OrchestratorDocument) {
    if doc.state != ServiceState::Partial {
        return;
    }
    doc.audit.push(AuditEntry {
        action: "create_reminder".into(),
        result: "retry_success".into(),
    });
    doc.state = ServiceState::Following;
    doc.message = "已补齐未完成的复查提醒".into();
}

pub fn state_summary(doc: &OrchestratorDocument) -> String {
    format!(
        "state={} service={} replan={} facts={} checklist={}/{} warranty={} audit={}",
        doc.state.label(),
        doc.service_type,
        doc.replan_count,
        doc.parsed.facts.len(),
        doc.checklist
            .iter()
            .filter(|item| item.status == "done")
            .count(),
        doc.checklist.len(),
        doc.warranty.expires,
        doc.audit.len()
    )
}

/// A compact, deterministic risk label for the dashboard and assistant bus.
/// It is derived only from the local case state; no model judgment is involved.
pub fn risk_summary(doc: &OrchestratorDocument) -> String {
    if doc.state == ServiceState::ParseFailed {
        return "解析失败 · 请修正通知".into();
    }
    if !doc.parsed.missing.is_empty() {
        return format!("待补充 · {} 项信息", doc.parsed.missing.len());
    }
    if !doc.service_issue.is_empty() {
        return "售后问题 · 待回访".into();
    }
    if doc.state == ServiceState::Partial {
        return "部分成功 · 待重试".into();
    }
    if doc.replan_count > 0 {
        return "有变更 · 需要确认".into();
    }
    match doc.state {
        ServiceState::Following | ServiceState::Completed | ServiceState::PostSale => {
            "运行正常 · 已核验".into()
        }
        ServiceState::AwaitingConfirm => "待确认 · 尚未执行".into(),
        _ => "低风险 · 等待通知".into(),
    }
}

pub fn completion_percent(doc: &OrchestratorDocument) -> u8 {
    let total = doc.timeline.len().max(1);
    let done = doc
        .timeline
        .iter()
        .filter(|step| step.status == "done")
        .count();
    ((done * 100) / total).min(100) as u8
}
