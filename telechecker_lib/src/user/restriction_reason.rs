use super::FromGrammersData;
use grammers_tl_types::enums::RestrictionReason as RestrictionReasonGramm;
use serde::Serialize;

#[derive(Serialize)]
pub struct RestrictionReason {
    pub platform: String,
    pub reason: String,
    pub text: String,
}

impl FromGrammersData for RestrictionReason {
    type GrammersType = RestrictionReasonGramm;

    fn from_grammers(grammers_data: Self::GrammersType) -> Self {
        match grammers_data {
            RestrictionReasonGramm::Reason(rr) => Self {
                platform: rr.platform,
                reason: rr.reason,
                text: rr.text,
            },
        }
    }
}
