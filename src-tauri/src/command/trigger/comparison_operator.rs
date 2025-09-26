use std::cmp::Ordering;

use config::ValueKind;
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Eq, PartialOrd, Ord)]
pub enum ComparisonOperator {
    #[serde(rename = "<")]
    /// Less than '<'
    Lt,
    #[serde(rename = "<=")]
    /// Less than or equal to '<='
    Le,
    #[serde(rename = "==")]
    /// Equal to `==`
    Eq,
    #[serde(rename = ">")]
    /// Greater than `>`
    Gt,
    #[serde(rename = ">=")]
    /// Greater than or equal to '>='
    Ge,
    #[serde(rename = "!=")]
    /// Not equal to `!=`
    Ne,
    #[default]
    /// Any value
    Any,
}

impl ComparisonOperator {
    pub fn compare<O>(&self, left: &O, right: &O) -> bool
    where
        O: Ord,
    {
        let result: Ordering = left.cmp(right);
        use ComparisonOperator::*;
        match self {
            Lt => result.is_lt(),
            Le => result.is_le(),
            Eq => result.is_eq(),
            Gt => result.is_gt(),
            Ge => result.is_ge(),
            Ne => result.is_ne(),
            Any => true,
        }
    }
}

impl From<config::Value> for ComparisonOperator {
    /// If an invalid operator is used, it defaults to [ComparisonOperator::Any].
    fn from(value: config::Value) -> Self {
        use ComparisonOperator::*;
        match value.to_string().to_lowercase().as_str() {
            "<" => Lt,
            "<=" => Le,
            "==" | "=" => Eq,
            ">=" => Ge,
            ">" => Gt,
            "!=" => Ne,
            "*" | "any" => Any,
            c => {
                warn!(
                    "Recieved an invalid Comparison operator: {}. Defaulting to Any",
                    c
                );
                Any
            }
        }
    }
}

impl From<ComparisonOperator> for ValueKind {
    fn from(value: ComparisonOperator) -> Self {
        use ComparisonOperator::*;
        use ValueKind::String as VString;
        match value {
            Lt => VString(String::from("<")),
            Le => VString(String::from("<=")),
            Eq => VString(String::from("==")),
            Gt => VString(String::from(">")),
            Ge => VString(String::from(">=")),
            Ne => VString(String::from("!=")),
            Any => VString(String::from("Any")),
        }
    }
}
