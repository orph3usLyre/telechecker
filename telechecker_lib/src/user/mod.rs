// Documentaiton should be equivalent to the types provided by [`grammers_tl_types`]
#![allow(missing_docs)]
// We allow name repetitions to match the types from [`grammers_tl_types`]
#![allow(clippy::module_name_repetitions)]

use grammers_tl_types::{enums::InputUser, types::User};
use serde::Serialize;

mod emoji_status;
mod restriction_reason;
mod user_profile_photo;
mod user_status;
mod username;

pub use self::{
    emoji_status::*, restriction_reason::*, user_profile_photo::*, user_status::*, username::*,
};

impl From<User> for UserData {
    fn from(value: User) -> Self {
        Self::from_grammers(value)
    }
}

impl From<&UserData> for grammers_tl_types::types::InputUser {
    fn from(value: &UserData) -> Self {
        Self {
            user_id: value.id,
            // TODO: what should be done if no access hash is provided?
            access_hash: value.access_hash.unwrap_or(0),
        }
    }
}

impl From<&UserData> for InputUser {
    fn from(value: &UserData) -> Self {
        Self::User(value.into())
    }
}

/// Local equivalent struct of [User]
#[derive(Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct UserData {
    pub is_self: bool,
    pub contact: bool,
    pub mutual_contact: bool,
    pub deleted: bool,
    pub bot: bool,
    pub bot_chat_history: bool,
    pub bot_nochats: bool,
    pub verified: bool,
    pub restricted: bool,
    pub min: bool,
    pub bot_inline_geo: bool,
    pub support: bool,
    pub scam: bool,
    pub apply_min_photo: bool,
    pub fake: bool,
    pub bot_attach_menu: bool,
    pub premium: bool,
    pub attach_menu_enabled: bool,
    pub bot_can_edit: bool,
    pub close_friend: bool,
    pub stories_hidden: bool,
    pub stories_unavailable: bool,
    pub id: i64,
    pub access_hash: Option<i64>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub phone: Option<String>,
    pub photo: Option<UserProfilePhoto>,
    pub status: Option<UserStatus>,
    pub bot_info_version: Option<i32>,
    pub restriction_reason: Option<Vec<RestrictionReason>>,
    pub bot_inline_placeholder: Option<String>,
    pub lang_code: Option<String>,
    pub emoji_status: Option<EmojiStatus>,
    pub usernames: Option<Vec<Username>>,
    pub stories_max_id: Option<i32>,
    pub color: Option<i32>,
    pub background_emoji_id: Option<i64>,
}

/// Helper trait used to convert types from [`grammers_tl_types`] into local types
trait FromGrammersData {
    type GrammersType;

    fn from_grammers(grammers_data: Self::GrammersType) -> Self;
}

impl FromGrammersData for UserData {
    type GrammersType = grammers_tl_types::types::User;

    fn from_grammers(grammers_data: Self::GrammersType) -> Self {
        Self {
            is_self: grammers_data.is_self,
            contact: grammers_data.contact,
            mutual_contact: grammers_data.mutual_contact,
            deleted: grammers_data.deleted,
            bot: grammers_data.bot,
            bot_chat_history: grammers_data.bot_chat_history,
            bot_nochats: grammers_data.bot_nochats,
            verified: grammers_data.verified,
            restricted: grammers_data.restricted,
            min: grammers_data.min,
            bot_inline_geo: grammers_data.bot_inline_geo,
            support: grammers_data.support,
            scam: grammers_data.scam,
            apply_min_photo: grammers_data.apply_min_photo,
            fake: grammers_data.fake,
            bot_attach_menu: grammers_data.bot_attach_menu,
            premium: grammers_data.premium,
            attach_menu_enabled: grammers_data.attach_menu_enabled,
            bot_can_edit: grammers_data.bot_can_edit,
            close_friend: grammers_data.close_friend,
            stories_hidden: grammers_data.stories_hidden,
            stories_unavailable: grammers_data.stories_unavailable,
            id: grammers_data.id,
            access_hash: grammers_data.access_hash,
            first_name: grammers_data.first_name,
            last_name: grammers_data.last_name,
            username: grammers_data.username,
            phone: grammers_data.phone,
            photo: grammers_data
                .photo
                .map_or_else(|| None, Option::<UserProfilePhoto>::from_grammers),
            status: grammers_data
                .status
                .map_or_else(|| None, |status| Some(UserStatus::from_grammers(status))),
            bot_info_version: grammers_data.bot_info_version,
            restriction_reason: grammers_data.restriction_reason.map(|rr| {
                rr.into_iter()
                    .map(RestrictionReason::from_grammers)
                    .collect()
            }),
            bot_inline_placeholder: grammers_data.bot_inline_placeholder,
            lang_code: grammers_data.lang_code,
            emoji_status: grammers_data
                .emoji_status
                .map_or_else(|| None, Option::<EmojiStatus>::from_grammers),
            usernames: grammers_data.usernames.map_or_else(
                || None,
                |uns| Some(uns.into_iter().map(Username::from_grammers).collect()),
            ),
            stories_max_id: grammers_data.stories_max_id,
            color: grammers_data.color,
            background_emoji_id: grammers_data.background_emoji_id,
        }
    }
}
