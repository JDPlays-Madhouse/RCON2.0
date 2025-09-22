use anyhow::bail;
use config::{Map, Value, ValueKind};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::integration::{CustomRewardEvent, CustomRewardVariant, IntegrationEvent};

mod server_trigger;
pub use server_trigger::GameServerTrigger;

mod subscription;
pub use subscription::SubscriptionTier;
mod comparison_operator;
pub use comparison_operator::ComparisonOperator;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Eq, PartialOrd, Ord)]
#[serde(tag = "trigger", content = "data")]
pub enum Trigger {
    /// Trigger for chat messages not using regular expression.
    Chat {
        /// Continuous text, not regex.
        pattern: String,
        case_sensitive: bool,
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
    Subscription {
        tier: SubscriptionTier,
        comparison_operator: ComparisonOperator,
    },
}

impl PartialEq<IntegrationEvent> for Trigger {
    fn eq(&self, event: &IntegrationEvent) -> bool {
        match self {
            Trigger::Chat {
                pattern,
                case_sensitive,
            } => {
                if let IntegrationEvent::Chat { msg, .. } = event {
                    let mut msg_match = msg.clone();
                    let mut pattern_match = pattern.clone();
                    if !case_sensitive {
                        msg_match = msg_match.to_lowercase();
                        pattern_match = pattern_match.to_lowercase();
                    }
                    msg_match.as_str().contains(&pattern_match)
                } else {
                    false
                }
            }
            Trigger::ChannelPointRewardRedeemed { id, variant, .. } => {
                if let IntegrationEvent::ChannelPoint(reward_event) = event {
                    &reward_event.id == id && &reward_event.variant == variant
                } else {
                    false
                }
            }
            Trigger::ChatRegex { .. } => {
                error!("Chat Regex as a trigger not implemented yet.");
                false
            }
            Trigger::Subscription {
                tier,
                comparison_operator,
            } => {
                let trigger_tier = tier.clone();
                if let IntegrationEvent::Subscription { tier, .. } = event {
                    let event_tier = tier.clone();

                    let result = dbg!(event_tier.cmp(&trigger_tier));
                    use ComparisonOperator::*;
                    match comparison_operator {
                        Lt => result.is_lt(),
                        Le => result.is_le(),
                        Eq => result.is_eq(),
                        Gt => result.is_gt(),
                        Ge => result.is_ge(),
                        Ne => result.is_ne(),
                        Any => true,
                    }
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
                case_sensitive: true,
            },
            ChatRegex { .. } => ChatRegex {
                pattern: Default::default(),
            },
            ChannelPointRewardRedeemed { .. } => ChannelPointRewardRedeemed {
                title: Default::default(),
                id: Default::default(),
                variant: Default::default(),
            },
            Subscription { .. } => Subscription {
                tier: SubscriptionTier::default(),
                comparison_operator: ComparisonOperator::default(),
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
            Trigger::ChannelPointRewardRedeemed { .. } => {
                IntegrationEvent::ChannelPoint(CustomRewardEvent::default())
            }
            Trigger::Subscription { .. } => IntegrationEvent::Subscription {
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
            "chat" => match trigger_table.get("pattern") {
                Some(p) => match p.clone().into_string() {
                    Ok(pattern) => {
                        let case_sensitive = match trigger_table.get("case_sensitive") {
                            Some(c) => match c.clone().into_bool() {
                                Ok(b) => b,
                                Err(e) => {
                                    error!(
                                        "Invalid case_sensitve property for chat trigger: {:?}",
                                        e
                                    );
                                    warn!("Setting value to true");
                                    true
                                }
                            },
                            None => {
                                error!("Missing case_sensitve property for chat trigger");
                                warn!("Setting value to true");
                                true
                            }
                        };

                        Ok(Self::Chat {
                            pattern,
                            case_sensitive,
                        })
                    }
                    Err(e) => bail!(e),
                },
                None => bail!(
                    "A trigger_type of '{}' needs the properties: {:?}",
                    trigger_type,
                    vec!["pattern"]
                ),
            },
            "subscription" => {
                let tier = match trigger_table.get("tier") {
                    Some(t) => t.clone().into(),
                    None => {
                        warn!(
                        "A trigger_type of '{}' needs the properties: {:?}. Defaulting to \"{:?}\"",
                        trigger_type,
                        vec!["tier", "comparison_operator"],
                        SubscriptionTier::default()
                    );

                        SubscriptionTier::default()
                    }
                };
                let comparison_operator = match trigger_table.get("comparison_operator") {
                    Some(t) => t.clone().into(),
                    None => {
                        warn!(
                        "A trigger_type of '{}' needs the properties: {:?} Defaulting to \"{:?}\"",
                        trigger_type,
                        vec!["tier", "comparison_operator"],
                        ComparisonOperator::default()

                    );
                        ComparisonOperator::default()
                    }
                };
                Ok(Self::Subscription {
                    tier,
                    comparison_operator,
                })
            }
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
            Trigger::Chat {
                pattern,
                case_sensitive,
            } => {
                map.insert(
                    "trigger_type".to_string(),
                    ValueKind::from(stringify!(Chat)),
                );
                map.insert("pattern".to_string(), ValueKind::from(pattern));
                map.insert(
                    "case_sensitive".to_string(),
                    ValueKind::from(case_sensitive),
                );
            }
            Trigger::ChatRegex { pattern } => {
                map.insert(
                    "trigger_type".to_string(),
                    ValueKind::from(stringify!(ChatRegex)),
                );
                map.insert("pattern".to_string(), ValueKind::from(pattern));
            }
            Trigger::ChannelPointRewardRedeemed { title, id, variant } => {
                map.insert(
                    "trigger_type".to_string(),
                    ValueKind::from(stringify!(ChannelPointRewardRedeemed)),
                );
                map.insert("title".to_string(), ValueKind::from(title));
                map.insert("id".to_string(), ValueKind::from(id));
                map.insert("variant".to_string(), ValueKind::from(variant));
            }
            Trigger::Subscription {
                tier,
                comparison_operator,
            } => {
                map.insert(
                    "trigger_type".to_string(),
                    ValueKind::from(stringify!(Subscription)),
                );
                map.insert("tier".to_string(), ValueKind::from(tier));
                map.insert(
                    "comparison_operator".to_string(),
                    ValueKind::from(comparison_operator),
                );
            }
        }
        Self::new(None, ValueKind::from(map))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("test", false, IntegrationEvent::Chat { msg: "test".to_string(), author: String::new() })]
    #[case("test", true, IntegrationEvent::Chat { msg: "test".to_string(), author: String::new() })]
    #[case("Test", false, IntegrationEvent::Chat { msg: "test".to_string(), author: String::new() })]
    #[case("test", false, IntegrationEvent::Chat { msg: "testa".to_string(), author: String::new() })]
    #[case("test", true, IntegrationEvent::Chat { msg: "testa".to_string(), author: String::new() })]
    #[case("Test", false, IntegrationEvent::Chat { msg: "testa".to_string(), author: String::new() })]
    fn chat_triggered(
        #[case] pattern: &str,
        #[case] case_sensitive: bool,
        #[case] event: IntegrationEvent,
    ) {
        assert_eq!(
            Trigger::Chat {
                pattern: pattern.to_string(),
                case_sensitive
            },
            event
        )
    }

    #[rstest]
    #[case("testa", false, IntegrationEvent::Chat { msg: "test".to_string(), author: String::new() })]
    #[case("testa", true, IntegrationEvent::Chat { msg: "test".to_string(), author: String::new() })]
    #[case("Test", true, IntegrationEvent::Chat { msg: "test".to_string(), author: String::new() })]
    fn chat_not_triggered(
        #[case] pattern: &str,
        #[case] case_sensitive: bool,
        #[case] event: IntegrationEvent,
    ) {
        assert_ne!(
            Trigger::Chat {
                pattern: pattern.to_string(),
                case_sensitive
            },
            event
        )
    }

    #[rstest]
    #[case("Testing", "1", CustomRewardVariant::New , IntegrationEvent::ChannelPoint(CustomRewardEvent {event_id:"1".to_string(),id:"1".to_string(),title:"Testing".to_string(),user_name:"Testing_User".to_string(),variant:CustomRewardVariant::New, message: "test message".to_string() }))]
    #[case("Testing", "1", CustomRewardVariant::Update , IntegrationEvent::ChannelPoint(CustomRewardEvent {event_id:"1".to_string(),id:"1".to_string(),title:"Testing".to_string(),user_name:"Testing_User".to_string(),variant:CustomRewardVariant::Update, message: "test message".to_string() }))]
    #[case(
        "ting", 
        "1", 
        CustomRewardVariant::New, 
        IntegrationEvent::ChannelPoint(CustomRewardEvent {event_id:"1".to_string(),id:"1".to_string(),title:"Testing".to_string(),user_name:"Testing_User".to_string(),variant:CustomRewardVariant::New, message: "test message".to_string() })
    )]
    #[case("Testing", "2", CustomRewardVariant::Update , IntegrationEvent::ChannelPoint(CustomRewardEvent {event_id:"1".to_string(),id:"2".to_string(),title:"Testing".to_string(),user_name:"Testing_User".to_string(),variant:CustomRewardVariant::Update, message: "test message".to_string() }))]
    fn channel_point_rewards_triggered(
        #[case] title: &str,
        #[case] id: &str,
        #[case] variant: CustomRewardVariant,
        #[case] event: IntegrationEvent,
    ) {
        assert_eq!(
            Trigger::ChannelPointRewardRedeemed {
                title: title.to_string(),
                id: id.to_string(),
                variant
            },
            event
        )
    }

    #[rstest]
    #[case(
        "Testing", 
        "1", 
        CustomRewardVariant::Update, 
        IntegrationEvent::ChannelPoint(CustomRewardEvent {event_id:"1".to_string(),id:"1".to_string(),title:"Testing".to_string(),user_name:"Testing_User".to_string(),variant:CustomRewardVariant::New, message: "test message".to_string() })
    )]
    #[case(
        "Testing", 
        "2", 
        CustomRewardVariant::New, 
        IntegrationEvent::ChannelPoint(CustomRewardEvent {event_id:"1".to_string(),id:"1".to_string(),title:"Testing".to_string(),user_name:"Testing_User".to_string(),variant:CustomRewardVariant::New, message: "test message".to_string() })
    )]
    fn channel_point_rewards_not_triggered(
        #[case] title: &str,
        #[case] id: &str,
        #[case] variant: CustomRewardVariant,
        #[case] event: IntegrationEvent,
    ) {
        assert_ne!(
            Trigger::ChannelPointRewardRedeemed {
                title: title.to_string(),
                id: id.to_string(),
                variant
            },
            event
        )
    }

    #[rstest]
    #[case(
        SubscriptionTier::Tier1,
        ComparisonOperator::Any, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier2, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier2,
        ComparisonOperator::Any, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier1, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier3,
        ComparisonOperator::Any, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier1, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Prime,
        ComparisonOperator::Any, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Prime, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Other(String::from("Test_Tier")),
        ComparisonOperator::Any, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Other(String::from("Test_Tier1")), user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Other(String::from("Test_Tier")),
        ComparisonOperator::Any, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Other(String::from("Test_Tier")), user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier1,
        ComparisonOperator::Eq, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier1, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier2,
        ComparisonOperator::Eq, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier2, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier3,
        ComparisonOperator::Eq, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier3, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Prime,
        ComparisonOperator::Eq, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Prime, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Other(String::from("Test_Tier")),
        ComparisonOperator::Eq, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Other(String::from("Test_Tier")), user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier2,
        ComparisonOperator::Lt, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier1, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier2,
        ComparisonOperator::Le, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier1, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier2,
        ComparisonOperator::Le, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier2, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier3,
        ComparisonOperator::Ne, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier1, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Prime,
        ComparisonOperator::Gt, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier2, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Other(String::from("Test_Tier")),
        ComparisonOperator::Ge, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier3, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Other(String::from("Test_Tier")),
        ComparisonOperator::Ge, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Other(String::from("Test_Tier")), user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Other(String::from("Test_Tier")),
        ComparisonOperator::Any, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Other(String::from("Test_Tier21")), user_name: String::new()}
    )]
    fn channel_subscription_triggered(
        #[case] tier: SubscriptionTier,
        #[case] comparison_operator: ComparisonOperator,
        #[case] event: IntegrationEvent,
    ) {
        assert_eq!(
            Trigger::Subscription {
                tier,
                comparison_operator
            },
            event
        )
    }

    #[rstest]
    #[case(
        SubscriptionTier::Tier2,
        ComparisonOperator::Eq, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier1, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier3,
        ComparisonOperator::Eq, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier2, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier3,
        ComparisonOperator::Eq, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Prime, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Other(String::from("Test_Tier")),
        ComparisonOperator::Eq, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Other(String::from("Test_Tier1")), user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Prime,
        ComparisonOperator::Eq, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Other(String::from("Test_Tier1")), user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier2,
        ComparisonOperator::Lt, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier3, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier2,
        ComparisonOperator::Le, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier3, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Prime,
        ComparisonOperator::Le, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier2, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier3,
        ComparisonOperator::Ne, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier3, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier2,
        ComparisonOperator::Gt, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Tier2, user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier2,
        ComparisonOperator::Gt, 
        IntegrationEvent::Subscription { tier: SubscriptionTier::Other(String::from("Test")), user_name: String::new()}
    )]
    #[case(
        SubscriptionTier::Tier2,
        ComparisonOperator::Gt, 
        IntegrationEvent::Chat { msg: String::from("Test"), author: String::from("Test") }
    )]
    fn channel_subscription_not_triggered(
        #[case] tier: SubscriptionTier,
        #[case] comparison_operator: ComparisonOperator,
        #[case] event: IntegrationEvent,
    ) {
        assert_ne!(
            Trigger::Subscription {
                tier,
                comparison_operator
            },
            event
        )
    }
}
