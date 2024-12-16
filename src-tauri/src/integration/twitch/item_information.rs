use std::fmt::Display;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::info;
use twitch_api::helix::points::get_custom_reward::CustomReward;
use twitch_types::RewardId;

use crate::command::Trigger;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomChannelPointRewardInfo {
    pub id: RewardId,
    pub title: String,
    pub is_enabled: bool,
    pub is_paused: bool,
    pub is_in_stock: bool,
    pub cost: usize,
    pub is_user_input_required: bool,
}

impl std::hash::Hash for CustomChannelPointRewardInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for CustomChannelPointRewardInfo {}

impl PartialEq for CustomChannelPointRewardInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl CustomChannelPointRewardInfo {
    pub fn is_available(&self) -> bool {
        self.is_enabled && !self.is_paused && self.is_in_stock
    }

    pub fn from_list(reward_list: Vec<CustomReward>) -> Vec<Self> {
        reward_list.into_iter().map(Self::from).collect_vec()
    }
    pub fn from_test_list(reward_list: Vec<CloneCustomReward>) -> Vec<Self> {
        reward_list.into_iter().map(Self::from).collect_vec()
    }

    pub fn display_for_log(reward_list: Vec<Self>) {
        info!("Channel Point Rewards: ");
        for reward in reward_list {
            info!("{}", reward);
        }
    }

    /// reward_list: A list of Channel Point Rewards.
    /// display: Whether to log the, to console.
    pub fn list_valid_trigger(reward_list: Vec<Self>, display: bool) -> Vec<Trigger> {
        if display {
            info!("Channel Point Rewards Triggers: ");
        }
        let triggers = reward_list.into_iter().map(Trigger::from).collect_vec();
        if display {
            for trigger in &triggers {
                info!("     {:?}", trigger);
            }
        }
        triggers
    }
}

impl From<CustomReward> for CustomChannelPointRewardInfo {
    fn from(reward: CustomReward) -> Self {
        let CustomReward {
            id,
            title,
            cost,
            is_enabled,
            is_user_input_required,
            is_paused,
            is_in_stock,
            ..
        } = reward;
        Self {
            id,
            title,
            is_enabled,
            is_paused,
            is_in_stock,
            cost,
            is_user_input_required,
        }
    }
}

impl From<CustomChannelPointRewardInfo> for Trigger {
    fn from(reward: CustomChannelPointRewardInfo) -> Self {
        let CustomChannelPointRewardInfo { title, id, .. } = reward;
        Trigger::ChannelPointRewardRedeemed {
            title,
            id: id.to_string(),
            variant: Default::default(),
        }
    }
}

impl Display for CustomChannelPointRewardInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CustomReward( title: '{}', id: '{}', enabled: {}, paused: {}, in_stock: {} )",
            self.title, self.id, self.is_enabled, self.is_paused, self.is_in_stock
        )
    }
}

impl From<CloneCustomReward> for CustomChannelPointRewardInfo {
    fn from(reward: CloneCustomReward) -> Self {
        let CloneCustomReward {
            id,
            title,
            cost,
            is_enabled,
            is_user_input_required,
            is_paused,
            is_in_stock,
            ..
        } = reward;
        Self {
            id,
            title,
            is_enabled,
            is_paused,
            is_in_stock,
            cost,
            is_user_input_required,
        }
    }
}

/// Clone of the [CustomReward]
pub struct CloneCustomReward {
    /// ID of the channel the reward is for
    pub broadcaster_id: twitch_types::UserId,
    /// Login of the channel the reward is for
    pub broadcaster_login: twitch_types::UserName,
    /// Display name of the channel the reward is for
    pub broadcaster_name: twitch_types::DisplayName,
    /// ID of the reward
    pub id: twitch_types::RewardId,
    /// The title of the reward
    pub title: String,
    /// The prompt for the viewer when they are redeeming the reward
    pub prompt: String,
    /// The cost of the reward
    pub cost: usize,
    /// Set of custom images of 1x, 2x and 4x sizes for the reward { url_1x: string, url_2x: string, url_4x: string }, can be null if no images have been uploaded
    pub image: Option<twitch_types::Image>,
    /// Set of default images of 1x, 2x and 4x sizes for the reward { url_1x: string, url_2x: string, url_4x: string }
    pub default_image: Option<twitch_types::Image>,
    /// Custom background color for the reward. Format: Hex with # prefix. Example: #00E5CB.
    pub background_color: String,
    /// Is the reward currently enabled, if false the reward won’t show up to viewers
    pub is_enabled: bool,
    /// Does the user need to enter information when redeeming the reward
    pub is_user_input_required: bool,
    /// Whether a maximum per stream is enabled and what the maximum is. { is_enabled: bool, max_per_stream: int }
    pub max_per_stream_setting: twitch_types::Max,
    /// Whether a maximum per user per stream is enabled and what the maximum is. { is_enabled: bool, max_per_user_per_stream: int }
    pub max_per_user_per_stream_setting: twitch_types::Max,
    /// Whether a cooldown is enabled and what the cooldown is. { is_enabled: bool, global_cooldown_seconds: int }
    pub global_cooldown_setting: CloneGlobalCooldown,
    /// Is the reward currently paused, if true viewers can’t redeem
    pub is_paused: bool,
    /// Is the reward currently in stock, if false viewers can’t redeem
    pub is_in_stock: bool,
    /// Should redemptions be set to FULFILLED status immediately when redeemed and skip the request queue instead of the normal UNFULFILLED status.
    pub should_redemptions_skip_request_queue: bool,
    /// The number of redemptions redeemed during the current live stream. Counts against the max_per_stream_setting limit. Null if the broadcasters stream isn’t live or max_per_stream_setting isn’t enabled.
    pub redemptions_redeemed_current_stream: Option<usize>,
    /// Timestamp of the cooldown expiration. Null if the reward isn’t on cooldown.
    pub cooldown_expires_at: Option<twitch_types::Timestamp>,
}

#[allow(dead_code)]
pub struct CloneGlobalCooldown {
    pub is_enabled: bool,
    pub global_cooldown_seconds: u32,
}

pub fn jd_channel_points() -> Vec<CloneCustomReward> {
    use twitch_types::points::Max::*;

    vec![
    CloneCustomReward { 
        broadcaster_id: "148141490".into(), 
        broadcaster_login: "jdplays".into(),
        broadcaster_name: "JDPlays".into(),
        id: "06fdb9c4-1a57-4dbd-93fc-d6f9bb43c54e".into(),
        title: "FIREWALL".into(),
        prompt: "FIRE FIRE FIRE".into(),
        cost: 5000,
        image: None,
        default_image: None,
        background_color: "#EA1D1D".into(),
        is_enabled: false,
        is_user_input_required: true,
        max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 },
        max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 },
        global_cooldown_setting: CloneGlobalCooldown{ is_enabled: false, global_cooldown_seconds: 0},
        is_paused: false,
        is_in_stock: true,
        should_redemptions_skip_request_queue: true,
        redemptions_redeemed_current_stream: None,
        cooldown_expires_at: None },
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "09fa7cb0-45a5-4788-82b8-1de36d7228ae".into(), title: "Toasted Marshmellows ".into(), prompt: "Everyone needs a campfire to see in the dark".into(), cost: 500, image: None, default_image: None, background_color: "#EB0400".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 60 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "170b115b-6eb5-4b62-b83d-bc08f2b142b8".into(), title: "BattleShips".into(), prompt: "Have you ever played battleships before? Type in your X, Y coords for a Arty strike\nPlease mark the X and a number, then the Y and a number eg X 1, y 1".into(), cost: 25000, image: None, default_image: None, background_color: "#9147FF".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 7200 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "22a60c3d-769e-4274-9b1c-a654ff3cc3d1".into(), title: "Treeeeeeeesss".into(), prompt: "A Forrest around u shall grow".into(), cost: 2500, image: None, default_image: None, background_color: "#005F0A".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 1260 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "3017801e-3ace-4c62-b322-aa76f08b6e37".into(), title: "STOP RIGHT THERE".into(), prompt: "JD Needs to Stop".into(), cost: 8500, image: None, default_image: None, background_color: "#00B2FF".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 3900 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "32f8dac0-b479-4136-8b4b-7227ea49b9af".into(), title: "JD Was Here!".into(), prompt: "Leaves a scorch mark on the map to show where JD has been".into(), cost: 2000, image: None, default_image: None, background_color: "#5C16C5".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 1500 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "38a933d2-8017-4e86-a275-0fa4a366f8d4".into(), title: "Inventory Shuffle".into(), prompt: "like 52 card pickup, its going to end up in a mess".into(), cost: 5000, image: None, default_image: None, background_color: "#00F593".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 4100 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "3a48eb4f-5ac8-46e7-aeb4-08db7c912ed4".into(), title: "FIRST".into(), prompt: "This is your proof you were here first, which means either you are the most dedicated, or the most bored. Either way you where here first!".into(), cost: 1, image: None, default_image: None, background_color: "#F80000".into(), is_enabled: true, is_user_input_required: false, max_per_stream_setting: MaxPerStream { is_enabled: true, max_per_stream: 1 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: false, global_cooldown_seconds: 0 }, is_paused: false, is_in_stock: false, should_redemptions_skip_request_queue: false, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "3cd34a22-c15a-484d-83bf-eddc2ef9291b".into(), title: "💣 One Shot 💥".into(), prompt: "This will leave a lasting impression on JD and the server".into(), cost: 50000, image: None, default_image: None, background_color: "#000000".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 5160 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "3e1f22bc-66fb-462e-939e-d4057f3d715d".into(), title: "LeakyFlamer".into(), prompt: "Sometimes your guns have technical difficulties ".into(), cost: 750, image: None, default_image: None, background_color: "#DB0000".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 1380 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "4b46a792-89e6-443a-ae6d-31c11bee8536".into(), title: "UFOs".into(), prompt: "Space Biters Attack!".into(), cost: 1999, image: None, default_image: None, background_color: "#FAB3FF".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 1080 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "5082667b-c85a-49e8-9260-0ed73936addd".into(), title: "Pants on Fire".into(), prompt: "liar liar pants on fire".into(), cost: 5000, image: None, default_image: None, background_color: "#FF0000".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 300 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "58d06f9d-ba2a-4df7-8c74-d99f66de99ca".into(), title: "Where did I park my car?".into(), prompt: "Who gave the spider a rocket launcher?".into(), cost: 7500, image: None, default_image: None, background_color: "#D1B3FF".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 2100 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "5bf18721-3ba6-44b6-83b7-7c95efdc9e18".into(), title: "It runs down your leg".into(), prompt: "Rather than a whole inventory poop, just a slow gentle release over time.".into(), cost: 10000, image: None, default_image: None, background_color: "#804600".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 1380 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "5ca58e65-e14e-457b-b1ba-9b961e01791d".into(), title: "!combat".into(), prompt: "to help with Combat".into(), cost: 750, image: None, default_image: None, background_color: "#BDA8FF".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: false, global_cooldown_seconds: 0 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "5dd1d118-7dd2-4145-89bf-e906164d99ba".into(), title: "Can i be your Pet? ".into(), prompt: "Gives JD a Pet".into(), cost: 1000, image: None, default_image: None, background_color: "#451093".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 60 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "65e161f9-e3e3-479a-896a-2470d2606611".into(), title: "God Clearing His Throat".into(), prompt: "cough cough cough".into(), cost: 5500, image: None, default_image: None, background_color: "#008C07".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 3780 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "6970482e-904e-4202-a4f8-2dd0b1b03861".into(), title: "Portal".into(), prompt: "Just a portal to BITERS (within 1K tiles)".into(), cost: 4000, image: None, default_image: None, background_color: "#BD0078".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 1500 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "71ef7616-906e-4d1f-84b3-04ed9202559f".into(), title: "Lob a Nade at his....".into(), prompt: "Run Faster JD".into(), cost: 200, image: None, default_image: None, background_color: "#FA1E1E".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 600 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "88a47fbe-0f41-42f0-8fdb-ec1d1f9c552f".into(), title: "Mini Nukes".into(), prompt: "Do you need a description? Its a Mini nuke delivered directly to JDs Head!".into(), cost: 10000, image: None, default_image: None, background_color: "#421561".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 1380 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "892c1684-7730-430d-a12f-e25bdc458b3a".into(), title: "You only NEED 1 HP".into(), prompt: "JD has said it many times anymore then 1 HP is a buffer!".into(), cost: 12000, image: None, default_image: None, background_color: "#1345AA".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 2700 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "8d9dcabc-2b9c-4e91-873d-6ab2b8b3f6a8".into(), title: "Blame me!".into(), prompt: "Your Name in the game. There is 99.997% chance you will be blamed for everything, and never be praised during your stay in the game! Also comes with a high chance of a sudden and untimely death.".into(), cost: 1000, image: None, default_image: None, background_color: "#000000".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: true, max_per_user_per_stream: 1 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 3600 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: false, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "9074bdec-483c-4f30-9764-2400e496bd12".into(), title: "Let there be light! ".into(), prompt: "It's Dark Chat cant see".into(), cost: 300, image: None, default_image: None, background_color: "#FF0000".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 180 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "9c713aa7-cf45-437d-9f7c-dc44b9136be8".into(), title: "Group Gastro".into(), prompt: "Gastro tends to spread from player to player".into(), cost: 15000, image: None, default_image: None, background_color: "#000000".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 3780 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "a90d0965-272a-41c7-a715-10a59a153b5e".into(), title: "Who Farted?".into(), prompt: "Well who did it?".into(), cost: 2501, image: None, default_image: None, background_color: "#E69900".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: true, max_per_user_per_stream: 2 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 1320 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "a9206e00-ab0d-44be-bab7-4068dc26c3a0".into(), title: "Shuffle it ALL".into(), prompt: "Inventory Shuffle EVERYTHING".into(), cost: 15000, image: None, default_image: None, background_color: "#E69900".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: true, max_per_user_per_stream: 2 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 12100 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "b526a2f1-cc63-4c2a-ba59-7d7f602e9ba1".into(), title: "AIR Support".into(), prompt: "Call in air support for JD".into(), cost: 1500, image: None, default_image: None, background_color: "#EB0400".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: false, global_cooldown_seconds: 0 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "b77bb703-b32d-46a9-887b-98f300e0d36d".into(), title: "Decon".into(), prompt: "Decon it decon it all!".into(), cost: 3000, image: None, default_image: None, background_color: "#FF0000".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 1080 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "ba67de35-aacf-4a9e-88d7-04041f44a1e8".into(), title: "Run Faster!".into(), prompt: "Accuracy isn't needed if you throw more!".into(), cost: 2000, image: None, default_image: None, background_color: "#781EFA".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 480 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "c82628fb-808d-405d-9067-314c29365c08".into(), title: "MASS Decon".into(), prompt: "Mass decon for EVERYONE!".into(), cost: 30000, image: None, default_image: None, background_color: "#931010".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 2880 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "c95f6365-f240-41ab-8319-f0d56570dcc5".into(), title: "Call for HELP!!!!!".into(), prompt: "HELP JD he is in trouble!".into(), cost: 2000, image: None, default_image: None, background_color: "#FFD37A".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 60 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "cae87f07-4a32-438b-add2-55943cab6545".into(), title: "Home".into(), prompt: "This might save JD, this might hinder JD either way Go Home JD go Home".into(), cost: 6000, image: None, default_image: None, background_color: "#151761".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 180 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "d041c611-eca9-4f6b-ac88-0317ab83d70a".into(), title: "!badfox".into(), prompt: "This Mod has been bad they should help JD out more often".into(), cost: 5000, image: None, default_image: None, background_color: "#8205B3".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 1080 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "d1e4bb1a-396d-44dd-8f4e-fc6ec396d0fa".into(), title: "boom stick".into(), prompt: "See this? This is my Boomstick!".into(), cost: 700, image: None, default_image: None, background_color: "#0034A0".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 720 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "da97e97c-5940-4b78-a6c3-55728708d745".into(), title: "JD Bullet Time".into(), prompt: "Slow Down MR JD".into(), cost: 4500, image: None, default_image: None, background_color: "#451093".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 2100 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "dcd7f0df-dcc7-4cf4-918e-c9b8f5f46227".into(), title: "FIRE Pits".into(), prompt: "When its dark this should help".into(), cost: 2000, image: None, default_image: None, background_color: "#9147FF".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 180 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "e0489a01-69f9-43be-a4ae-4dd73f1149c0".into(), title: "Power Rangers Assemble".into(), prompt: "Bring the mod team over to help!".into(), cost: 7500, image: None, default_image: None, background_color: "#000000".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: false, global_cooldown_seconds: 0 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "e23f6585-bcfb-45a9-8f35-8b3e7c01497c".into(), title: "Rise from the dead".into(), prompt: "Did you know JD Moonlights as a necromancer?".into(), cost: 1000, image: None, default_image: None, background_color: "#FFD37A".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 240 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "e438f413-de1d-447d-bc51-da17a940f8db".into(), title: "Carpet Bomb".into(), prompt: "Artillery Bombardment Fixes everything".into(), cost: 10000, image: None, default_image: None, background_color: "#BDA8FF".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 2820 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "e81276dd-1a97-4f79-a7c8-bd56aee9ea50".into(), title: "The Devils Special Reward".into(), prompt: "A very special reward can be redeemed once per stream, use at your own discretion ".into(), cost: 666, image: None, default_image: None, background_color: "#FF6666".into(), is_enabled: true, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: true, max_per_stream: 1 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: false, global_cooldown_seconds: 0 }, is_paused: false, is_in_stock: false, should_redemptions_skip_request_queue: false, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "ec3aaa03-adbd-49f2-9779-6b52093fc296".into(), title: "Fortress".into(), prompt: "Protect JD the leader of the Mad House in a Fortress (or imprison him so he can't run away!)".into(), cost: 1250, image: None, default_image: None, background_color: "#0013A3".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 300 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "f1f6f6a9-780d-4355-bb42-930656c150d4".into(), title: "JD can drive anything".into(), prompt: "JD can drive anything if its in range that is".into(), cost: 5000, image: None, default_image: None, background_color: "#EB0400".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 1500 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "f5537157-8885-4e87-aef1-9c28e4eeb882".into(), title: "Hi Im new here".into(), prompt: "Use your JD tears to say Hi!".into(), cost: 200, image: None, default_image: None, background_color: "#FFBF00".into(), is_enabled: true, is_user_input_required: false, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: true, max_per_user_per_stream: 1 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: false, global_cooldown_seconds: 0 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: false, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "f9c71f9a-94fc-4d9c-acc6-d48028f4f836".into(), title: "Multiplayer Snake".into(), prompt: "Its Snake times, Remember don't cross the path!".into(), cost: 3000, image: None, default_image: None, background_color: "#2A6138".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 1200 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }, 
    CloneCustomReward { broadcaster_id: "148141490".into(), broadcaster_login: "jdplays".into(), broadcaster_name: "JDPlays".into(), id: "ffb332b7-34e0-4ce5-b32c-056472de3529".into(), title: "poop".into(), prompt: "Well oops".into(), cost: 2000, image: None, default_image: None, background_color: "#C76500".into(), is_enabled: false, is_user_input_required: true, max_per_stream_setting: MaxPerStream { is_enabled: false, max_per_stream: 0 }, max_per_user_per_stream_setting: MaxPerUserPerStream { is_enabled: false, max_per_user_per_stream: 0 }, global_cooldown_setting: CloneGlobalCooldown { is_enabled: true, global_cooldown_seconds: 2100 }, is_paused: false, is_in_stock: true, should_redemptions_skip_request_queue: true, redemptions_redeemed_current_stream: None, cooldown_expires_at: None }]
}
