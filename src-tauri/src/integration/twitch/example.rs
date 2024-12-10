use twitch_api::helix::points::CustomReward;
use twitch_types::RewardId;

struct CustomRewardEvent {
    id: RewardId,
    title: String,
    is_enabled: bool,
    is_paused: bool,
    is_in_stock: bool,
}

impl CustomRewardEvent {
    pub fn is_available(&self) -> bool {
        self.is_enabled && !self.is_paused && self.is_in_stock
    }
}
