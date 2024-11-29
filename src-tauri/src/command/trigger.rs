use serde::{Deserialize, Serialize};

use crate::integration::IntegrationEvent;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Eq, PartialOrd, Ord)]
#[serde(tag = "trigger", content = "data")]
pub enum Trigger {
    /// Trigger for chat messages not using regular expression.
    Chat {
        /// continuous text, case insensitive. not regex.
        pattern: String,
    },
    /// Trigger for chat messages using regular expression. Not Implemented.
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
    pub fn is_match(&self, event: &IntegrationEvent) -> bool {
        match self {
            Trigger::Chat { pattern } => {
                if let IntegrationEvent::Chat { msg, .. } = event {
                    msg.as_str().contains(pattern)
                } else {
                    false
                }
            }
            Trigger::ChannelPointRewardRedemed { id, .. } => {
                if let IntegrationEvent::ChannelPoint { id: event_id, .. } = event {
                    event_id == id
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// A default implementation of each enum used for being a key.
    pub fn trigger_type(&self) -> Self {
        use Trigger::*;
        match self {
            Chat { .. } => Chat {
                pattern: Default::default(),
            },
            ChatRegex { .. } => ChatRegex {
                pattern: Default::default(),
            },
            ChannelPointRewardRedemed { .. } => ChannelPointRewardRedemed {
                name: Default::default(),
                id: Default::default(),
            },
        }
    }

    /// Linking [Trigger] with [IntegrationEvent].
    pub fn event_type(&self) -> IntegrationEvent {
        match self {
            Trigger::Chat { .. } => IntegrationEvent::Chat {
                msg: Default::default(),
                author: Default::default(),
            },
            Trigger::ChatRegex { .. } => IntegrationEvent::Chat {
                msg: Default::default(),
                author: Default::default(),
            },
            Trigger::ChannelPointRewardRedemed { .. } => IntegrationEvent::ChannelPoint {
                id: Default::default(),
                redeemer: Default::default(),
            },
        }
    }

    /// Returns `true` if the trigger is [`Chat`].
    ///
    /// [`Chat`]: Trigger::Chat
    #[must_use]
    pub fn is_chat(&self) -> bool {
        matches!(self, Self::Chat { .. })
    }

    /// Returns `true` if the trigger is [`ChatRegex`].
    ///
    /// [`ChatRegex`]: Trigger::ChatRegex
    #[must_use]
    pub fn is_chat_regex(&self) -> bool {
        matches!(self, Self::ChatRegex { .. })
    }

    /// Returns `true` if the trigger is [`ChannelPointRewardRedemed`].
    ///
    /// [`ChannelPointRewardRedemed`]: Trigger::ChannelPointRewardRedemed
    #[must_use]
    pub fn is_channel_point_reward_redemed(&self) -> bool {
        matches!(self, Self::ChannelPointRewardRedemed { .. })
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//
//     #[test]
//     fn test_trigger_equality() {
//         assert_eq!(
//             Trigger::Chat {
//                 pattern: "test".into()
//             },
//             Trigger::Chat {
//                 pattern: "test".into()
//             }
//         )
//     }
// }
