//! Deterministic notification parser.
//!
//! Every emitted fact carries the exact matched text as `source_quote`. The
//! native port deliberately keeps the first slice small and auditable; richer
//! date normalization can be added without changing the state-machine API.

use crate::model::{Fact, ParsedNotice};

fn line_value(text: &str, labels: &[&str]) -> Option<(String, String)> {
    for raw in text.lines() {
        let line = raw.trim();
        for label in labels {
            if let Some(rest) = line.strip_prefix(label) {
                let rest = rest.trim_start_matches(['：', ':', ' ', '\t']);
                if !rest.is_empty() {
                    return Some((rest.trim().to_string(), line.to_string()));
                }
            }
        }
    }
    None
}

fn push_fact(facts: &mut Vec<Fact>, field: &str, value: String, quote: String, source_id: &str) {
    facts.push(Fact {
        field: field.into(),
        value,
        source_quote: quote,
        source_id: source_id.into(),
        confidence: "high".into(),
    });
}

pub fn parse_notice(text: &str, source_id: &str) -> ParsedNotice {
    if text.trim().is_empty() {
        return ParsedNotice {
            missing: vec!["notice_text".into()],
            errors: vec!["通知文本为空".into()],
            ..ParsedNotice::default()
        };
    }

    let mut facts = Vec::new();
    let mut errors = Vec::new();

    if let Some((value, quote)) = line_value(text, &["订单号", "订单编号"]) {
        push_fact(&mut facts, "order_id", value, quote, source_id);
    }
    if let Some((value, quote)) = line_value(text, &["配送预计", "配送日期", "送货日期", "到货日期"]) {
        push_fact(&mut facts, "delivery_date", value, quote, source_id);
    }
    if let Some((value, quote)) = line_value(text, &["安装预约", "预约安装", "安装时间", "上门安装"]) {
        push_fact(&mut facts, "installation_time", value, quote, source_id);
    }
    if let Some((value, quote)) = line_value(text, &["安装地址", "送货地址", "地址"]) {
        push_fact(&mut facts, "address", value, quote, source_id);
    }
    if let Some((value, quote)) = line_value(text, &["状态", "进度"]) {
        push_fact(&mut facts, "status", value, quote, source_id);
    }

    for phrase in ["延期", "延迟", "推迟"] {
        if text.contains(phrase) {
            push_fact(&mut facts, "delivery_status", "delayed".into(), phrase.into(), source_id);
            break;
        }
    }

    if facts.is_empty() {
        errors.push("通知中没有可识别事实".into());
    }

    let required = ["order_id", "delivery_date", "installation_time"];
    let missing = required
        .iter()
        .filter(|field| !facts.iter().any(|fact| fact.field == **field))
        .map(|field| (*field).into())
        .collect();

    ParsedNotice { facts, missing, errors }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delay_quote_is_the_exact_input_phrase() {
        for (text, quote) in [("配送延迟", "延迟"), ("到货推迟", "推迟"), ("配送延期", "延期")] {
            let parsed = parse_notice(text, "t");
            let fact = parsed.fact("delivery_status").expect("delay fact");
            assert_eq!(fact.source_quote, quote);
            assert!(text.contains(&fact.source_quote));
        }
    }

    #[test]
    fn required_fields_are_missing_instead_of_guessed() {
        let parsed = parse_notice("配送延迟", "t");
        assert!(parsed.fact("order_id").is_none());
        assert!(parsed.missing.iter().any(|field| field == "order_id"));
        assert!(parsed.missing.iter().any(|field| field == "delivery_date"));
        assert!(parsed.missing.iter().any(|field| field == "installation_time"));
    }
}
