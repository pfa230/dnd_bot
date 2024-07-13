use teloxide::{
    adaptors::{throttle::Limits, CacheMe, Throttle},
    requests::RequesterExt,
    types::{Me, StickerSet},
};

pub type Bot = CacheMe<Throttle<teloxide::Bot>>;

pub fn create_bot() -> Bot {
    teloxide::Bot::from_env()
        .throttle(Limits::default())
        .cache_me()
}

#[derive(Clone)]
pub struct Context {
    pub stickers: StickerSet,
    pub me: Me,
}