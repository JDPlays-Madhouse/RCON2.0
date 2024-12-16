use std::fmt::Display;

use anyhow::bail;
use config::ValueKind;
use serde::{Deserialize, Serialize};
use twitch_types::SubscriptionTier;

#[allow(dead_code)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Hash)]
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
        tier: String,
        user_name: String,
    },
    Unknown,
    Stop,
    Pause,
    Continue,
    /// TODO: add implementation for updating triggers and commands for runner.
    Update,
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
            Stop => Stop,
            Pause => Pause,
            Continue => Continue,
            Update => Update,
            Disconnected => Disconnected,
            Subscription { .. } => Subscription {
                tier: Default::default(),
                user_name: Default::default(),
            },
        }
    }
}

pub fn normalise_tier(
    twitch_tier: Option<SubscriptionTier>,
    _youtube_tier: Option<String>,
) -> String {
    if twitch_tier.is_some() {
        match twitch_tier.unwrap() {
            SubscriptionTier::Tier1 => "Tier1".to_string(),
            SubscriptionTier::Tier2 => "Tier2".to_string(),
            SubscriptionTier::Tier3 => "Tier3".to_string(),
            SubscriptionTier::Prime => "Prime".to_string(),
            SubscriptionTier::Other(t) => t,
        }
    } else {
        String::new()
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum CustomRewardVariant {
    #[default]
    New,
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

#[derive(Debug, Default, PartialEq, Eq, Clone, Hash)]
pub struct CustomRewardEvent {
    /// Occurrence ID
    pub event_id: String,
    /// Reward ID
    pub id: String,
    /// Reward Title
    pub title: String,
    pub user_name: String,
    pub status: CustomRewardVariant,
}
