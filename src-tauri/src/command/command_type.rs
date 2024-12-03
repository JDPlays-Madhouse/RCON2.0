use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(tag = "type", content = "data")]
pub enum CommandType {
    ChannelPoints(String),
    Chat,
}
