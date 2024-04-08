use super::FromGrammersData;
use grammers_tl_types::enums::UserStatus as UserStatusGramm;
use grammers_tl_types::types::UserStatusOffline as UserStatusOfflineGramm;
use grammers_tl_types::types::UserStatusOnline as UserStatusOnlineGramm;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum UserStatus {
    Empty,
    Online(UserStatusOnline),
    Offline(UserStatusOffline),
    Recently,
    LastWeek,
    LastMonth,
}

#[derive(Serialize, Deserialize)]
pub struct UserStatusOnline {
    pub expires: i32,
}
#[derive(Serialize, Deserialize)]
pub struct UserStatusOffline {
    pub was_online: i32,
}

impl FromGrammersData for UserStatus {
    type GrammersType = UserStatusGramm;

    fn from_grammers(grammers_data: Self::GrammersType) -> Self {
        match grammers_data {
            UserStatusGramm::Empty => Self::Empty,
            UserStatusGramm::Online(UserStatusOnlineGramm { expires }) => {
                Self::Online(UserStatusOnline { expires })
            }
            UserStatusGramm::Offline(UserStatusOfflineGramm { was_online }) => {
                Self::Offline(UserStatusOffline { was_online })
            }
            UserStatusGramm::Recently => Self::Recently,
            UserStatusGramm::LastWeek => Self::LastWeek,
            UserStatusGramm::LastMonth => Self::LastMonth,
        }
    }
}
