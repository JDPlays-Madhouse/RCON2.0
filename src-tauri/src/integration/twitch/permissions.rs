use std::vec;

use itertools::Itertools;
use tracing::warn;
use twitch_api::eventsub::EventType::{self};
use twitch_oauth2::Scope::{self, *};

/// Get the permissions required for a given eventsub event type.
/// As of 21 Sep 2025.
pub fn get_eventsub_eventtype_scopes(event_type: EventType) -> Option<Vec<Scope>> {
    match event_type {
        EventType::ChannelAdBreakBegin => Some(vec![ChannelReadAds]),
        EventType::ChannelChatClear => Some(vec![UserReadChat]),
        EventType::ChannelChatClearUserMessages => Some(vec![UserReadChat]),
        EventType::ChannelChatMessage => Some(vec![UserReadChat, UserBot]),
        EventType::ChannelChatMessageDelete => Some(vec![UserReadChat]),
        EventType::ChannelChatNotification => Some(vec![UserReadChat]),
        EventType::ChannelCharityCampaignDonate => Some(vec![ChannelReadCharity]),
        EventType::ChannelCharityCampaignProgress => Some(vec![ChannelReadCharity]),
        EventType::ChannelCharityCampaignStart => Some(vec![ChannelReadCharity]),
        EventType::ChannelCharityCampaignStop => Some(vec![ChannelReadCharity]),
        EventType::ChannelUpdate => None,
        EventType::ChannelFollow => Some(vec![ModeratorReadFollowers]),
        EventType::ChannelSubscribe => Some(vec![ChannelReadSubscriptions]),
        EventType::ChannelCheer => Some(vec![BitsRead]),
        EventType::ChannelBan => Some(vec![Scope::ChannelModerate]),
        EventType::ChannelUnban => Some(vec![Scope::ChannelModerate]),
        EventType::ChannelPointsCustomRewardAdd => Some(vec![ChannelReadRedemptions]),
        EventType::ChannelPointsCustomRewardUpdate => Some(vec![ChannelReadRedemptions]),
        EventType::ChannelPointsCustomRewardRemove => Some(vec![ChannelReadRedemptions]),
        EventType::ChannelPointsCustomRewardRedemptionAdd => Some(vec![ChannelReadRedemptions]),
        EventType::ChannelPointsCustomRewardRedemptionUpdate => Some(vec![ChannelReadRedemptions]),
        EventType::ChannelPollBegin => Some(vec![ChannelReadPolls]),
        EventType::ChannelPollProgress => Some(vec![ChannelReadPolls]),
        EventType::ChannelPollEnd => Some(vec![ChannelReadPolls]),
        EventType::ChannelPredictionBegin => Some(vec![ChannelReadPredictions]),
        EventType::ChannelPredictionProgress => Some(vec![ChannelReadPredictions]),
        EventType::ChannelPredictionLock => Some(vec![ChannelReadPredictions]),
        EventType::ChannelPredictionEnd => Some(vec![ChannelReadPredictions]),
        EventType::ChannelShoutoutCreate => Some(vec![ModeratorReadShoutouts]),
        EventType::ChannelShoutoutReceive => Some(vec![ModeratorReadShoutouts]),
        EventType::ChannelRaid => None,
        EventType::ChannelSubscriptionEnd => Some(vec![ChannelReadSubscriptions]),
        EventType::ChannelSubscriptionGift => Some(vec![ChannelReadSubscriptions]),
        EventType::ChannelSubscriptionMessage => Some(vec![ChannelReadSubscriptions]),
        EventType::ChannelShieldModeBegin => Some(vec![ModeratorReadShieldMode]),
        EventType::ChannelShieldModeEnd => Some(vec![ModeratorReadShieldMode]),
        EventType::ChannelGoalBegin => Some(vec![ChannelReadGoals]),
        EventType::ChannelGoalProgress => Some(vec![ChannelReadGoals]),
        EventType::ChannelGoalEnd => Some(vec![ChannelReadGoals]),
        EventType::ChannelHypeTrainBegin => Some(vec![ChannelReadHypeTrain]),
        EventType::ChannelHypeTrainProgress => Some(vec![ChannelReadHypeTrain]),
        EventType::ChannelHypeTrainEnd => Some(vec![ChannelReadHypeTrain]),
        EventType::ConduitShardDisabled => None,
        EventType::StreamOnline => None,
        EventType::StreamOffline => None,
        EventType::UserUpdate => Some(vec![UserReadEmail]),
        EventType::UserAuthorizationRevoke => None,
        EventType::UserAuthorizationGrant => None,
        EventType::AutomodMessageHold => Some(vec![ModeratorManageAutoMod]),
        EventType::AutomodMessageUpdate => Some(vec![ModeratorManageAutoMod]),
        EventType::AutomodSettingsUpdate => Some(vec![ModeratorReadAutomodSettings]),
        EventType::AutomodTermsUpdate => Some(vec![ModeratorManageAutoMod]),
        EventType::ChannelBitsUse => Some(vec![BitsRead]),
        EventType::ChannelChatUserMessageHold => Some(vec![UserReadChat, UserBot]),
        EventType::ChannelChatUserMessageUpdate => Some(vec![UserReadChat, UserBot]),
        EventType::ChannelChatSettingsUpdate => Some(vec![UserReadChat, UserBot, ChannelBot]),
        EventType::ChannelUnbanRequestCreate => Some(vec![
            ModeratorReadUnbanRequests,
            ModeratorManageUnbanRequests,
        ]),
        EventType::ChannelUnbanRequestResolve => Some(vec![
            ModeratorReadUnbanRequests,
            ModeratorManageUnbanRequests,
        ]),
        EventType::ChannelPointsAutomaticRewardRedemptionAdd => {
            Some(vec![ChannelReadRedemptions, ChannelManageRedemptions])
        }
        EventType::ChannelSharedChatBegin => None,
        EventType::ChannelSharedChatEnd => None,
        EventType::ChannelSharedChatUpdate => None,
        EventType::ChannelSuspiciousUserMessage => Some(vec![ModeratorReadSuspiciousUsers]),
        EventType::ChannelSuspiciousUserUpdate => Some(vec![ModeratorReadSuspiciousUsers]),
        EventType::ChannelGuestStarSessionBegin => Some(vec![
            ChannelReadGuestStar,
            ChannelManageGuestStar,
            ModeratorReadGuestStar,
            ModeratorManageGuestStar,
        ]),
        EventType::ChannelGuestStarSessionEnd => Some(vec![
            ChannelReadGuestStar,
            ChannelManageGuestStar,
            ModeratorReadGuestStar,
            ModeratorManageGuestStar,
        ]),
        EventType::ChannelGuestStarSettingsUpdate => Some(vec![
            ChannelReadGuestStar,
            ChannelManageGuestStar,
            ModeratorReadGuestStar,
            ModeratorManageGuestStar,
        ]),
        EventType::ChannelGuestStarGuestUpdate => Some(vec![
            ChannelReadGuestStar,
            ChannelManageGuestStar,
            ModeratorReadGuestStar,
            ModeratorManageGuestStar,
        ]),
        EventType::ChannelModerate => Some(vec![
            ModeratorReadBlockedTerms,
            ModeratorManageBlockedTerms,
            ModeratorReadChatSettings,
            ModeratorManageChatSettings,
            ModeratorReadUnbanRequests,
            ModeratorManageUnbanRequests,
            ModeratorReadBannedUsers,
            ModeratorManageBannedUsers,
            ModeratorReadChatMessages,
            ModeratorManageChatMessages,
            ModeratorReadWarnings,
            ModeratorManageWarnings,
            ModeratorReadModerators,
            ModeratorReadVips,
        ]),
        EventType::ChannelModeratorAdd => Some(vec![ModerationRead]),
        EventType::ChannelModeratorRemove => Some(vec![ModerationRead]),
        EventType::ChannelVipAdd => Some(vec![ChannelReadVips, ChannelManageVips]),
        EventType::ChannelWarningAcknowledge => {
            Some(vec![ModeratorReadWarnings, ModeratorManageWarnings])
        }
        EventType::ChannelWarningSend => Some(vec![ModeratorReadWarnings, ModeratorManageWarnings]),
        EventType::ChannelVipRemove => Some(vec![ChannelReadVips, ChannelManageVips]),
        EventType::UserWhisperMessage => Some(vec![UserReadWhispers, UserManageWhispers]),
        event_type => {
            unknown_event_type(event_type);
            None
        }
    }
}

fn unknown_event_type(event_type: EventType) {
    warn!("Unknown event type: {event_type}");
    warn!("Please visit https://github.com/JDPlays-Madhouse/RCON2.0/issues/new?title=New+Twitch+EventType+{event_type}&labels=enhancement&assignees=ozy_viking&body=Add+%60EventType::{event_type:?}%60+to+%60integration::twitch::permissions::get_eventsub_eventtype_scopes%60+match+statement.+It+is+currently+returning+%60None%60.");
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

#[cfg(test)]
mod tests {
    use tracing::subscriber;

    use super::*;

    #[test]
    fn check_url_creation_works() {
        use tracing_subscriber::fmt;

        let _default = subscriber::set_default(fmt().with_test_writer().finish());

        unknown_event_type(EventType::ChannelBitsUse)
    }
}
