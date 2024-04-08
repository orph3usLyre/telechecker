use crate::UserData;
use anyhow::Error;
use tokio::sync::{
    mpsc,
    mpsc::channel,
    oneshot::{self, Receiver, Sender},
    watch,
};

use crate::{ConnectionStatus, CHANNEL_BUFFER_SIZE};

pub fn create_comms_channels() -> (CommChannelsApp, CommChannelsClient, Sender<Error>) {
    // Sender is app
    let (api_info_tx, api_info_rx) = oneshot::channel();
    // Sender is app
    let (user_phone_tx, user_phone_rx) = oneshot::channel();
    // Sender is app
    let (input_phones_tx, input_phones_rx) = channel(CHANNEL_BUFFER_SIZE);
    // Sender is runtime
    let (user_data_tx, user_data_rx) = channel(CHANNEL_BUFFER_SIZE);
    // Sender is app
    let (code_receive_tx, code_receive_rx) = oneshot::channel();
    // Sender is app
    let (pass_receive_tx, pass_receive_rx) = oneshot::channel();
    let (connection_status_tx, connection_status_rx) = watch::channel(ConnectionStatus::default());

    let (client_exit_error_tx, client_exit_error_rx) = oneshot::channel();

    let comm_channels_app = CommChannelsApp::new(
        api_info_tx,
        user_phone_tx,
        code_receive_tx,
        input_phones_tx,
        user_data_rx,
        pass_receive_tx,
        connection_status_rx,
        client_exit_error_rx,
    );

    let comms_channels_client = CommChannelsClient::new(
        api_info_rx,
        user_phone_rx,
        code_receive_rx,
        input_phones_rx,
        user_data_tx,
        pass_receive_rx,
        connection_status_tx,
    );
    (
        comm_channels_app,
        comms_channels_client,
        client_exit_error_tx,
    )
}

pub struct CommChannelsApp {
    pub api_info_tx: Option<Sender<(i32, String)>>,
    pub user_phone_tx: Option<Sender<String>>,
    pub user_code_tx: Option<Sender<String>>,
    pub input_phones_tx: mpsc::Sender<Vec<String>>,
    pub user_data_rx: mpsc::Receiver<Vec<UserData>>,
    pub pass_receive_tx: Option<Sender<String>>,
    pub connection_status_rx: watch::Receiver<ConnectionStatus>,
    pub client_exit_error_rx: Receiver<Error>,
}

impl CommChannelsApp {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        api_info_tx: Sender<(i32, String)>,
        user_phone_tx: Sender<String>,
        user_code_tx: Sender<String>,
        input_phones_tx: mpsc::Sender<Vec<String>>,
        user_data_rx: mpsc::Receiver<Vec<UserData>>,
        pass_receive_tx: Sender<String>,
        connection_status_rx: watch::Receiver<ConnectionStatus>,
        client_exit_error_rx: Receiver<Error>,
    ) -> Self {
        Self {
            api_info_tx: Some(api_info_tx),
            user_phone_tx: Some(user_phone_tx),
            user_code_tx: Some(user_code_tx),
            input_phones_tx,
            user_data_rx,
            pass_receive_tx: Some(pass_receive_tx),
            connection_status_rx,
            client_exit_error_rx,
        }
    }
}

pub struct CommChannelsClient {
    pub api_info_rx: Receiver<(i32, String)>,
    pub user_phone_rx: Receiver<String>,
    pub code_receiver_rx: Receiver<String>,
    pub pass_recieve_rx: Receiver<String>,
    pub input_phones_rx: mpsc::Receiver<Vec<String>>,
    pub user_data_tx: mpsc::Sender<Vec<UserData>>,
    pub connection_status_tx: watch::Sender<ConnectionStatus>,
}

impl CommChannelsClient {
    pub fn new(
        api_info_rx: Receiver<(i32, String)>,
        user_phone_rx: Receiver<String>,
        code_receiver_rx: Receiver<String>,
        input_phones_rx: mpsc::Receiver<Vec<String>>,
        user_data_tx: mpsc::Sender<Vec<UserData>>,
        pass_recieve_rx: Receiver<String>,
        connection_status_tx: watch::Sender<ConnectionStatus>,
    ) -> Self {
        Self {
            api_info_rx,
            user_phone_rx,
            code_receiver_rx,
            pass_recieve_rx,
            input_phones_rx,
            user_data_tx,
            connection_status_tx,
        }
    }
}
