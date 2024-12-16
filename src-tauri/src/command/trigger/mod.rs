use anyhow::bail;
use config::{Map, Value, ValueKind};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::integration::{CustomRewardEvent, CustomRewardVariant, IntegrationEvent};

mod server_trigger;
pub use server_trigger::GameServerTrigger;

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
    ChannelPointRewardRedeemed {
        title: String,
        /// Twitch id for this channel points reward redeem.
        id: String,
        variant: CustomRewardVariant,
    },
    Subscription,
}
impl PartialEq<IntegrationEvent> for Trigger {
    fn eq(&self, event: &IntegrationEvent) -> bool {
        match self {
            Trigger::Chat { pattern } => {
                if let IntegrationEvent::Chat { msg, .. } = event {
                    msg.as_str().contains(pattern)
                } else {
                    false
                }
            }
            Trigger::ChannelPointRewardRedeemed { id, .. } => {
                if let IntegrationEvent::ChannelPoint(reward_event) = event {
                    &reward_event.id == id
                } else {
                    false
                }
            }
            Trigger::ChatRegex { .. } => {
                error!("Chat Regex as a trigger not implemented yet.");
                false
            }
            Trigger::Subscription => {
                if let IntegrationEvent::Subscription { .. } = event {
                    // TODO: Implement tiers for triggers.
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl Trigger {
    pub fn is_match(&self, event: &IntegrationEvent) -> bool {
        self == event
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
            ChannelPointRewardRedeemed { .. } => ChannelPointRewardRedeemed {
                title: Default::default(),
                id: Default::default(),
                variant: Default::default(),
            },
            Subscription => Subscription,
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
            Trigger::ChannelPointRewardRedeemed { .. } => {
                IntegrationEvent::ChannelPoint(CustomRewardEvent::default())
            }
            Trigger::Subscription => IntegrationEvent::Subscription {
                tier: Default::default(),
                user_name: Default::default(),
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

    /// Returns `true` if the trigger is [`ChannelPointRewardRedeemed`].
    ///
    /// [`ChannelPointRewardRedeemed`]: Trigger::ChannelPointRewardRedeemed
    #[must_use]
    pub fn is_channel_point_reward_redeemed(&self) -> bool {
        matches!(self, Self::ChannelPointRewardRedeemed { .. })
    }
}

impl TryFrom<Value> for Trigger {
    type Error = anyhow::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let trigger_table = match value.into_table() {
            Ok(t) => t,
            Err(e) => bail!(e),
        };
        let trigger_type = match trigger_table.get("trigger_type") {
            Some(trigger_type) => match trigger_type.clone().into_string() {
                Ok(t) => t,
                Err(e) => bail!(e),
            },
            None => bail!("Missing 'trigger_type' property."),
        };

        match trigger_type.to_lowercase().as_str() {
            "chat" => match trigger_table.get("pattern") {
                Some(p) => match p.clone().into_string() {
                    Ok(pattern) => Ok(Self::Chat { pattern }),
                    Err(e) => bail!(e),
                },
                None => bail!(
                    "A trigger_type of '{}' needs the properties: {:?}",
                    trigger_type,
                    vec!["pattern"]
                ),
            },
            "channelpointrewardredeemed" => {
                let id = match trigger_table.get("id") {
                    Some(id) => match id.clone().into_string() {
                        Ok(i) => i,
                        Err(e) => {
                            bail!(e)
                        }
                    },
                    None => bail!(
                        "A trigger_type of '{}' needs the properties: {:?}",
                        trigger_type,
                        vec!["id", "title", "variant"]
                    ),
                };
                let title = match trigger_table.get("title") {
                    Some(name) => match name.clone().into_string() {
                        Ok(n) => n,
                        Err(e) => {
                            bail!(e)
                        }
                    },
                    None => bail!(
                        "A trigger_type of '{}' needs the properties: {:?}",
                        trigger_type,
                        vec!["id", "title", "variant"]
                    ),
                };
                let variant = match trigger_table.get("variant") {
                    Some(name) => match name.clone().into_string() {
                        Ok(n) => match CustomRewardVariant::try_from(n) {
                            Ok(v) => v,
                            Err(e) => {
                                error!("{}", e);
                                warn!("Using new variant for {} trigger", title);
                                CustomRewardVariant::New
                            }
                        },
                        Err(e) => {
                            bail!(e)
                        }
                    },
                    None => bail!(
                        "A trigger_type of '{}' needs the properties: {:?}",
                        trigger_type,
                        vec!["id", "title", "variant"]
                    ),
                };
                Ok(Self::ChannelPointRewardRedeemed { title, id, variant })
            }
            "subscription" => Ok(Self::Subscription),
            trig => {
                error!("Trigger type has not been implemented: {}", trig);
                bail!("Trigger type has not been implemented: {}", trig)
            }
        }
    }
}

impl From<Trigger> for Value {
    fn from(trigger: Trigger) -> Self {
        let mut map = Map::new();
        match trigger {
            Trigger::Chat { pattern } => {
                map.insert(
                    "trigger_type".to_string(),
                    ValueKind::from(stringify!(Chat)),
                );
                map.insert("pattern".to_string(), ValueKind::from(pattern));
            }
            Trigger::ChatRegex { pattern } => {
                map.insert(
                    "trigger_type".to_string(),
                    ValueKind::from(stringify!(ChatRegex)),
                );
                map.insert("pattern".to_string(), ValueKind::from(pattern));
            }
            Trigger::ChannelPointRewardRedeemed {
                title: name,
                id,
                variant,
            } => {
                map.insert(
                    "trigger_type".to_string(),
                    ValueKind::from(stringify!(ChannelPointRewardRedeemed)),
                );
                map.insert("name".to_string(), ValueKind::from(name));
                map.insert("id".to_string(), ValueKind::from(id));
                map.insert("variant".to_string(), ValueKind::from(variant));
            }
            Trigger::Subscription => {
                map.insert(
                    "trigger_type".to_string(),
                    ValueKind::from(stringify!(Subscription)),
                );
            }
        }
        Self::new(None, ValueKind::from(map))
    }
}
