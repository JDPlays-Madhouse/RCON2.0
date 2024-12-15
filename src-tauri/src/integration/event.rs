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

#[derive(Debug, Default, PartialEq, Eq, Clone, Hash)]
pub struct CustomRewardEvent {
    pub id: String,
    pub title: String,
    pub is_enabled: bool,
    pub is_paused: bool,
    pub is_in_stock: bool,
}

impl CustomRewardEvent {
    pub fn is_available(&self) -> bool {
        self.is_enabled && !self.is_paused && self.is_in_stock
    }
}
