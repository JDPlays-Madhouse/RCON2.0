use itertools::Itertools;
use tracing::warn;
use twitch_api::eventsub::EventType::{self, *};
use twitch_oauth2::Scope::{self, *};

/// Get the permissions required for a given eventsub event type.
/// As of 06 Nov 2024.
pub fn get_eventsub_eventtype_scopes(event_type: EventType) -> Option<Vec<Scope>> {
    match event_type {
        ChannelAdBreakBegin => Some(vec![ChannelReadAds]),
        ChannelChatClear => Some(vec![UserReadChat]),
        ChannelChatClearUserMessages => Some(vec![UserReadChat]),
        ChannelChatMessage => Some(vec![UserReadChat, UserBot]),
        ChannelChatMessageDelete => Some(vec![UserReadChat]),
        ChannelChatNotification => Some(vec![UserReadChat]),
        ChannelCharityCampaignDonate => Some(vec![ChannelReadCharity]),
        ChannelCharityCampaignProgress => Some(vec![ChannelReadCharity]),
        ChannelCharityCampaignStart => Some(vec![ChannelReadCharity]),
        ChannelCharityCampaignStop => Some(vec![ChannelReadCharity]),
        ChannelUpdate => None,
        ChannelFollow => Some(vec![ModeratorReadFollowers]),
        ChannelSubscribe => Some(vec![ChannelReadSubscriptions]),
        ChannelCheer => Some(vec![BitsRead]),
        ChannelBan => Some(vec![ChannelModerate]),
        ChannelUnban => Some(vec![ChannelModerate]),
        ChannelPointsCustomRewardAdd => Some(vec![ChannelReadRedemptions]),
        ChannelPointsCustomRewardUpdate => Some(vec![ChannelReadRedemptions]),
        ChannelPointsCustomRewardRemove => Some(vec![ChannelReadRedemptions]),
        ChannelPointsCustomRewardRedemptionAdd => Some(vec![ChannelReadRedemptions]),
        ChannelPointsCustomRewardRedemptionUpdate => Some(vec![ChannelReadRedemptions]),
        ChannelPollBegin => Some(vec![ChannelReadPolls]),
        ChannelPollProgress => Some(vec![ChannelReadPolls]),
        ChannelPollEnd => Some(vec![ChannelReadPolls]),
        ChannelPredictionBegin => Some(vec![ChannelReadPredictions]),
        ChannelPredictionProgress => Some(vec![ChannelReadPredictions]),
        ChannelPredictionLock => Some(vec![ChannelReadPredictions]),
        ChannelPredictionEnd => Some(vec![ChannelReadPredictions]),
        ChannelShoutoutCreate => Some(vec![ModeratorReadShoutouts]),
        ChannelShoutoutReceive => Some(vec![ModeratorReadShoutouts]),
        ChannelRaid => None,
        ChannelSubscriptionEnd => Some(vec![ChannelReadSubscriptions]),
        ChannelSubscriptionGift => Some(vec![ChannelReadSubscriptions]),
        ChannelSubscriptionMessage => Some(vec![ChannelReadSubscriptions]),
        ChannelShieldModeBegin => Some(vec![ModeratorReadShieldMode]),
        ChannelShieldModeEnd => Some(vec![ModeratorReadShieldMode]),
        ChannelGoalBegin => Some(vec![ChannelReadGoals]),
        ChannelGoalProgress => Some(vec![ChannelReadGoals]),
        ChannelGoalEnd => Some(vec![ChannelReadGoals]),
        ChannelHypeTrainBegin => Some(vec![ChannelReadHypeTrain]),
        ChannelHypeTrainProgress => Some(vec![ChannelReadHypeTrain]),
        ChannelHypeTrainEnd => Some(vec![ChannelReadHypeTrain]),
        ConduitShardDisabled => None,
        StreamOnline => None,
        StreamOffline => None,
        UserUpdate => Some(vec![UserReadEmail]),
        UserAuthorizationRevoke => None,
        UserAuthorizationGrant => None,
        _ => {
            warn!("Event type not found: {:?}", event_type);
            None
        }
    }
}

/// Get the consolidated scopes required for a list of eventsub event types.
pub fn get_eventsub_consolidated_scopes(subscriptions: Vec<EventType>) -> Vec<Scope> {
    let mut scopes: Vec<Scope> = Vec::new();
    for subscription in subscriptions {
        if let Some(s) = get_eventsub_eventtype_scopes(subscription) {
            scopes.extend(s);
        }
    }
    scopes = scopes.into_iter().unique_by(|s| s.to_string()).collect();
    scopes
}
