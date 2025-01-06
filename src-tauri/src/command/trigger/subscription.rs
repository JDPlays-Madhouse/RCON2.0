use config::{Value, ValueKind};
use serde::{Deserialize, Serialize};

/// Subscription tiers, only Twitch at the moment.
#[derive(Default, Clone, Debug, PartialEq, Eq, Deserialize, PartialOrd, Ord, Hash)]
#[serde(field_identifier)]
#[repr(i8)]
pub enum SubscriptionTier {
    #[default]
    Tier1 = 1,
    Tier2 = 2,
    Tier3 = 3,
    Prime = 0,
    /// Other is not orderable so it will be considered the lowest.
    Other(String) = -1,
}

impl PartialEq<twitch_types::SubscriptionTier> for SubscriptionTier {
    fn eq(&self, other: &twitch_types::SubscriptionTier) -> bool {
        use twitch_types::SubscriptionTier as TwitchTier;
        use SubscriptionTier::*;
        match self {
            Tier1 => &TwitchTier::Tier1 == other,
            Tier2 => &TwitchTier::Tier2 == other,
            Tier3 => &TwitchTier::Tier3 == other,
            Prime => &TwitchTier::Prime == other,
            Other(tier) => {
                if let TwitchTier::Other(twitch_tier) = other {
                    tier == twitch_tier
                } else {
                    false
                }
            }
        }
    }
}

impl Serialize for SubscriptionTier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            SubscriptionTier::Tier1 => "Tier1",
            SubscriptionTier::Tier2 => "Tier2",
            SubscriptionTier::Tier3 => "Tier3",
            SubscriptionTier::Prime => "Prime",
            SubscriptionTier::Other(o) => o,
        })
    }
}

impl From<Value> for SubscriptionTier {
    fn from(value: Value) -> Self {
        use SubscriptionTier::*;
        match value.to_string().to_lowercase().as_str() {
            "tier1" | "tier 1" => Tier1,
            "tier2" | "tier 2" => Tier2,
            "tier3" | "tier 3" => Tier3,
            "prime" => Prime,
            t => Other(t.to_string()),
        }
    }
}

impl From<SubscriptionTier> for ValueKind {
    fn from(value: SubscriptionTier) -> Self {
        use SubscriptionTier::*;
        use ValueKind::String as VString;
        match value {
            Tier1 => VString(String::from("Tier1")),
            Tier2 => VString(String::from("Tier2")),
            Tier3 => VString(String::from("Tier3")),
            Prime => VString(String::from("Prime")),
            Other(t) => VString(t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use twitch_types::SubscriptionTier as TwitchTier;
    use SubscriptionTier::*;

    #[rstest]
    #[case(Tier1, TwitchTier::Tier1)]
    fn subscription_eq(#[case] sub_tier: SubscriptionTier, #[case] twitch_sub_tier: TwitchTier) {
        assert_eq!(sub_tier, twitch_sub_tier);
    }
}
