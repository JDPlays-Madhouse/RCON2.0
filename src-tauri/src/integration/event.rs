use std::fmt::Display;

use anyhow::bail;
use config::ValueKind;
use serde::{Deserialize, Serialize};
use twitch_types::SubscriptionTier;

use crate::command::trigger;

#[allow(dead_code)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IntegrationEvent {
    #[default]
    Connected,
    Disconnected,
    Chat {
        msg: String,
        author: String,
    },
    ChannelPoint(CustomRewardEvent),
    Subscription {
        tier: trigger::SubscriptionTier,
        user_name: String,
    },
    GiftSub {
        tier: trigger::SubscriptionTier,
        count: u64,
        user_name: Option<String>,
    },
    Unknown,
    Stop,
    Pause,
    Continue,
    /// TODO: add implementation for updating triggers and commands for runner.
    Update,
    Server,
}

impl IntegrationEvent {
    /// Returns `true` if the integration event is [`Chat`].
    ///
    /// [`Chat`]: IntegrationEvent::Chat
    #[must_use]
    pub fn is_chat(&self) -> bool {
        matches!(self, Self::Chat { .. })
    }

    /// Returns `true` if the integration event is [`ChannelPoint`].
    ///
    /// [`ChannelPoint`]: IntegrationEvent::ChannelPoint
    #[must_use]
    pub fn is_channel_point(&self) -> bool {
        matches!(self, Self::ChannelPoint { .. })
    }

    pub fn message(&self) -> Option<&str> {
        match self {
            IntegrationEvent::ChannelPoint(custom_reward_event) => {
                Some(&custom_reward_event.message)
            }
            IntegrationEvent::Chat { msg, .. } => Some(&msg),
            IntegrationEvent::Connected
            | IntegrationEvent::Disconnected
            | IntegrationEvent::Server
            | IntegrationEvent::Subscription { .. }
            | IntegrationEvent::GiftSub { .. }
            | IntegrationEvent::Unknown
            | IntegrationEvent::Stop
            | IntegrationEvent::Pause
            | IntegrationEvent::Continue
            | IntegrationEvent::Update => None,
        }
    }

    pub fn username(&self) -> String {
        match self {
            IntegrationEvent::ChannelPoint(custom_reward_event) => {
                custom_reward_event.user_name.clone()
            }
            IntegrationEvent::Chat { author, .. } => author.clone(),
            IntegrationEvent::Subscription { user_name, .. } => user_name.clone(),
            IntegrationEvent::GiftSub { user_name, .. } => user_name.clone().unwrap_or_default(),
            IntegrationEvent::Connected
            | IntegrationEvent::Disconnected
            | IntegrationEvent::Server
            | IntegrationEvent::Unknown
            | IntegrationEvent::Stop
            | IntegrationEvent::Pause
            | IntegrationEvent::Continue
            | IntegrationEvent::Update => "<server>".to_string(),
        }
    }
    /// A default implementation of each enum which can be used for being a key.
    pub fn event_type(&self) -> Self {
        use IntegrationEvent::*;
        match self {
            Chat { .. } => Chat {
                msg: Default::default(),
                author: Default::default(),
            },
            Connected => Connected,
            ChannelPoint(..) => ChannelPoint(CustomRewardEvent::default()),
            Unknown => Unknown,
            Server => Server,
            Stop => Stop,
            Pause => Pause,
            Continue => Continue,
            Update => Update,
            Disconnected => Disconnected,
            Subscription { .. } => Subscription {
                tier: Default::default(),
                user_name: Default::default(),
            },
            GiftSub { .. } => GiftSub {
                tier: Default::default(),
                count: Default::default(),
                user_name: Default::default(),
            },
        }
    }
}

pub fn normalise_tier(
    twitch_tier: Option<SubscriptionTier>,
    _youtube_tier: Option<String>,
) -> trigger::SubscriptionTier {
    if twitch_tier.is_some() {
        match twitch_tier.unwrap() {
            SubscriptionTier::Tier1 => trigger::SubscriptionTier::Tier1,
            SubscriptionTier::Tier2 => trigger::SubscriptionTier::Tier2,
            SubscriptionTier::Tier3 => trigger::SubscriptionTier::Tier3,
            SubscriptionTier::Prime => trigger::SubscriptionTier::Prime,
            SubscriptionTier::Other(t) => trigger::SubscriptionTier::Other(t),
        }
    } else {
        trigger::SubscriptionTier::Other(String::new())
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum CustomRewardVariant {
    #[default]
    /// Needs subscription to "channel.channel_points_custom_reward_redemption.add"
    New,
    /// Needs subscription to "channel.channel_points_custom_reward_redemption.update"
    Update,
}

impl Display for CustomRewardVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "New"),
            Self::Update => write!(f, "Update"),
        }
    }
}

impl TryFrom<String> for CustomRewardVariant {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "new" => Ok(Self::New),
            "update" => Ok(Self::Update),
            _ => bail!("Invalid input for custom reward variant: {}", value),
        }
    }
}

impl From<CustomRewardVariant> for ValueKind {
    fn from(variant: CustomRewardVariant) -> Self {
        Self::String(variant.to_string())
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct CustomRewardEvent {
    pub variant: CustomRewardVariant,
    /// Occurrence ID
    pub event_id: String,
    /// Reward ID
    pub id: String,
    /// Reward Title
    pub title: String,
    pub user_name: String,
    pub message: String,
}
