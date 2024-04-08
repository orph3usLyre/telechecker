use super::FromGrammersData;
use grammers_tl_types::enums::EmojiStatus as EmojiStatusGramm;
use serde::Serialize;

#[derive(Serialize)]
pub enum EmojiStatus {
    Status(Status),
    Until(EmojiStatusUntil),
}

#[derive(Serialize)]
pub struct Status {
    pub document_id: i64,
}

#[derive(Serialize)]
pub struct EmojiStatusUntil {
    pub document_id: i64,
    pub until: i32,
}

impl FromGrammersData for Option<EmojiStatus> {
    type GrammersType = EmojiStatusGramm;

    fn from_grammers(grammers_data: Self::GrammersType) -> Self {
        match grammers_data {
            EmojiStatusGramm::Empty => None,
            EmojiStatusGramm::Status(s) => Some(EmojiStatus::Status(Status {
                document_id: s.document_id,
            })),
            EmojiStatusGramm::Until(u) => Some(EmojiStatus::Until(EmojiStatusUntil {
                document_id: u.document_id,
                until: u.until,
            })),
        }
    }
}
