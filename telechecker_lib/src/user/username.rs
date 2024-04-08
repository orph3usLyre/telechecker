use super::FromGrammersData;
use grammers_tl_types::enums::Username as UsernameGramm;
use serde::Serialize;

#[derive(Serialize)]
pub struct Username {
    pub editable: bool,
    pub active: bool,
    pub username: String,
}

impl FromGrammersData for Username {
    type GrammersType = UsernameGramm;

    fn from_grammers(grammers_data: Self::GrammersType) -> Self {
        match grammers_data {
            UsernameGramm::Username(un) => Self {
                editable: un.editable,
                active: un.active,
                username: un.username,
            },
        }
    }
}
