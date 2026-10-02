//! Pure domain types for the native port. No UI, host services, or clocks.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ServiceState {
    #[default]
    Idle,
    Parsing,
    NeedInfo,
    AwaitingConfirm,
    Executing,
    Following,
    Partial,
    Replan,
    Unstable,
    Completed,
    PostSale,
    ParseFailed,
}

impl ServiceState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Parsing => "Parsing",
            Self::NeedInfo => "NeedInfo",
            Self::AwaitingConfirm => "AwaitingConfirm",
            Self::Executing => "Executing",
            Self::Following => "Following",
            Self::Partial => "Partial",
            Self::Replan => "Replan",
            Self::Unstable => "Unstable",
            Self::Completed => "Completed",
            Self::PostSale => "PostSale",
            Self::ParseFailed => "ParseFailed",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Fact {
    pub field: String,
    pub value: String,
    pub source_quote: String,
    pub source_id: String,
    pub confidence: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParsedNotice {
    pub facts: Vec<Fact>,
    pub missing: Vec<String>,
    pub errors: Vec<String>,
}

impl ParsedNotice {
    pub fn fact(&self, field: &str) -> Option<&Fact> {
        self.facts.iter().find(|fact| fact.field == field)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    pub title: String,
    pub detail: String,
    pub source_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TimelineStep {
    pub title: String,
    pub status: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuditEntry {
    pub action: String,
    pub result: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrchestratorDocument {
    pub state: ServiceState,
    pub notice: String,
    pub parsed: ParsedNotice,
    pub plans: Vec<Plan>,
    pub selected_plan: usize,
    pub replan_count: u8,
    pub timeline: Vec<TimelineStep>,
    pub audit: Vec<AuditEntry>,
    pub message: String,
}

impl Default for OrchestratorDocument {
    fn default() -> Self {
        Self {
            state: ServiceState::Idle,
            notice: String::new(),
            parsed: ParsedNotice::default(),
            plans: Vec::new(),
            selected_plan: 0,
            replan_count: 0,
            timeline: vec![
                TimelineStep { title: "配送".into(), status: "pending".into() },
                TimelineStep { title: "安装预约".into(), status: "pending".into() },
                TimelineStep { title: "售后复查".into(), status: "pending".into() },
            ],
            audit: Vec::new(),
            message: "粘贴一条配送/安装通知开始".into(),
        }
    }
}
