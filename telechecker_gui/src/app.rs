use crate::comms::CommChannelsApp;
use crate::UserData;

pub struct Telegather {
    pub config: Config,
    pub error_message: Option<String>,
    pub info_message: Option<String>,
    pub user_phone: Option<String>,
    pub cache: Cache,
    pub connection_status: ConnectionStatus,
    pub comm_channels: CommChannelsApp,
    pub user_data: Option<Vec<UserData>>,
}

pub struct Config {
    pub hide_phone: bool,
    pub hide_code: bool,
    pub hide_pass: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hide_phone: false,
            hide_code: false,
            hide_pass: true,
        }
    }
}

#[derive(Default, Clone, Copy, Eq, PartialEq, strum::IntoStaticStr)]
pub enum ConnectionStatus {
    #[default]
    RequiresApiInfo,
    NotConnected,
    AwaitingPhoneNumber,
    AwaitingUserCode,
    AwaitingPassword,
    Authorized,
}

pub struct Cache {
    pub user_phone: String,
    pub user_code: String,
    pub user_pwd: String,
    pub phones_input: String,
    pub api_id: String,
    pub api_hash: String,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            user_phone: String::new(),
            user_code: String::new(),
            user_pwd: String::new(),
            phones_input: String::new(),
            api_id: String::new(),
            api_hash: String::new(),
        }
    }
}

impl Telegather {
    pub fn new(comm_channels: CommChannelsApp) -> Self {
        Self {
            config: Default::default(),
            user_phone: None,
            error_message: None,
            info_message: None,
            cache: Cache::new(),
            connection_status: Default::default(),
            comm_channels,
            user_data: None,
        }
    }
}
