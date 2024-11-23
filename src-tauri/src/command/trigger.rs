use serde::{Deserialize, Serialize};

use crate::integration::Event;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "trigger", content = "data")]
pub enum Trigger {
    /// trigger for chat messages not using regular expression.
    Chat {
        /// continuous text, case insensitive. not regex.
        pattern: String,
    },
    /// trigger for chat messages using regular expression. Not Implemented.
    ChatRegex {
        /// Regular expression.
        pattern: String,
    },
    ChannelPointRewardRedemed {
        name: String,
        /// Twitch id for this channel points reward redeem.
        id: String,
    },
}

impl Trigger {
    pub fn is_match(&self, event: Event) -> bool {
        match self {
            Trigger::Chat { pattern } => {
                if let Event::Chat { msg, .. } = event {
                    msg.as_str().contains(pattern)
                } else {
                    false
                }
            }
            Trigger::ChannelPointRewardRedemed { id, .. } => {
                if let Event::ChannelPoint { id: event_id, .. } = event {
                    event_id == *id
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
