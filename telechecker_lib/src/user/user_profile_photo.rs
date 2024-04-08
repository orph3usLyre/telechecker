use super::FromGrammersData;
use grammers_tl_types::enums::UserProfilePhoto as UserProfilePhotoGramm;
use serde::Serialize;

#[derive(Serialize)]
pub struct UserProfilePhoto {
    pub has_video: bool,
    pub personal: bool,
    pub photo_id: i64,
    pub stripped_thumb: Option<Vec<u8>>,
    pub dc_id: i32,
}

impl FromGrammersData for Option<UserProfilePhoto> {
    type GrammersType = UserProfilePhotoGramm;

    fn from_grammers(grammers_data: Self::GrammersType) -> Self {
        match grammers_data {
            UserProfilePhotoGramm::Empty => None,
            UserProfilePhotoGramm::Photo(user_profile_photo) => Some(UserProfilePhoto {
                has_video: user_profile_photo.has_video,
                personal: user_profile_photo.personal,
                photo_id: user_profile_photo.photo_id,
                stripped_thumb: user_profile_photo.stripped_thumb,
                dc_id: user_profile_photo.dc_id,
            }),
        }
    }
}
