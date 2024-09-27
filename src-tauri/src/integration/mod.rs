pub enum Api {
    Twitch,
    YouTube,
}

pub enum Connection {
    Stream,
    Fetch(String),
}

pub struct Integration {
    pub api: Api,
    pub connection: Connection,
}
