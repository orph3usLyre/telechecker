use crate::warn;
use crate::Receiver;
use crate::UserRetriever;
use crate::{comms::CommChannelsClient, info};
use crate::{debug, ConnectionStatus};
use anyhow::Result;

#[tokio::main]
#[tracing::instrument(skip_all)]
pub async fn client_handler(
    api_id2: Option<i32>,
    api_hash2: Option<String>,
    session_file: &str,
    comms_channel: CommChannelsClient,
) -> Result<()> {
    let CommChannelsClient {
        api_info_rx,
        user_phone_rx,
        code_receiver_rx,
        pass_recieve_rx,
        mut input_phones_rx,
        user_data_tx,
        connection_status_tx,
    } = comms_channel;

    let tx = connection_status_tx.clone();

    let (api_id, api_hash) = match (api_id2, api_hash2) {
        (Some(id), Some(hash)) => (id, hash),
        _ => {
            connection_status_tx.send(ConnectionStatus::RequiresApiInfo)?;
            match api_info_rx.await {
                Ok(api_info) => api_info,
                Err(_) => anyhow::bail!("Unable to retrieve api info"),
            }
        }
    };

    connection_status_tx.send(ConnectionStatus::NotConnected)?;

    let mut user_retriever = UserRetriever::new(api_id, api_hash, session_file).await?;
    debug!("UserRetriever built");

    let user_phone_input_handler = || async move {
        tx.send(ConnectionStatus::AwaitingPhoneNumber)?;
        phone_input_handler(user_phone_rx).await
    };
    let tx = connection_status_tx.clone();
    let code_input_handler = || async move {
        tx.send(ConnectionStatus::AwaitingUserCode)?;
        code_input_handler(code_receiver_rx).await
    };
    let tx = connection_status_tx.clone();
    let password_input_handler = |_hint: String| async move {
        tx.send(ConnectionStatus::AwaitingPassword)?;
        password_input_handler(pass_recieve_rx).await
    };
    user_retriever
        .sign_in(
            user_phone_input_handler,
            code_input_handler,
            password_input_handler,
        )
        .await?;
    connection_status_tx.send(ConnectionStatus::Authorized)?;
    debug!("Signed in. Waiting for phone numbers");
    while let Some(phone_numbers) = input_phones_rx.recv().await {
        let users: Vec<_> = user_retriever.get_users(phone_numbers).await?.collect();
        debug!("Received numbers, sending");
        debug!("Returned users length: {}", users.len());
        if user_data_tx.send(users).await.is_err() {
            debug!("Cannot sent any more users");
            break;
        };
    }

    Ok(())
}

async fn phone_input_handler(user_phone_rx: Receiver<String>) -> Result<String> {
    let Ok(phone) = user_phone_rx.await else {
        anyhow::bail!("Unable to retrieve user phone");
    };
    info!("phone handler: {phone}");
    Ok(phone)
}

async fn code_input_handler(code_receiver_rx: Receiver<String>) -> Result<String> {
    let Ok(code) = code_receiver_rx.await else {
        anyhow::bail!("Unable to retrieve user code");
    };
    Ok(code)
}

async fn password_input_handler(pass_recieve_rx: Receiver<String>) -> Result<String> {
    let out = match pass_recieve_rx.await {
        Ok(pwd) => pwd,
        Err(_) => anyhow::bail!("Unable to retrieve user password"),
    };
    Ok(out)
}
