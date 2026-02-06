use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Status {
    Current,
    Drifted,
    Missing,
    Invalid,
}

impl Status {
    pub fn is_problem(&self) -> bool {
        matches!(self, Status::Drifted | Status::Missing)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Current => write!(f, "CURRENT"),
            Status::Drifted => write!(f, "DRIFTED"),
            Status::Missing => write!(f, "MISSING"),
            Status::Invalid => write!(f, "INVALID"),
        }
    }
}
