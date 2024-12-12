use twitch_types::RewardId;

#[allow(dead_code)]
struct CustomRewardEvent {
    id: RewardId,
    title: String,
    is_enabled: bool,
    is_paused: bool,
    is_in_stock: bool,
}

#[allow(dead_code)]
impl CustomRewardEvent {
    pub fn is_available(&self) -> bool {
        self.is_enabled && !self.is_paused && self.is_in_stock
    }
}
