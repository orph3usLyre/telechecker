#![forbid(unsafe_code)]
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    rustdoc::broken_intra_doc_links,
    missing_docs
)]

//! TODO: Docs
//!
//!
//!
//!
//!
//!
//!

use crate::user::UserData;
use anyhow::{Context, Result};
use grammers_client::{Client, Config, SignInError};
use grammers_session::Session;
use grammers_tl_types as tl;
use grammers_tl_types::{
    enums::{contacts::ImportedContacts, InputContact},
    types::InputPhoneContact,
};
use std::future::Future;
use tl::enums::{InputUser, User};
use tracing::{debug, trace, warn};

/// The name of the session file created by [`grammers_session::Session`]
pub const SESSION_FILE: &str = "telechecker.session";

/// This module provides local types that are equivalent to types found in [`grammers_tl_types`] to
/// allow them to be serializable
pub mod user;

#[tracing::instrument]
/// Function that validates a phone number using the [`phonenumber`] crate
///
/// # Errors
///
/// Returns [`Err`] if the [`phonenumber`] is unable to parse the provided number
pub fn validate(user_phone: &str) -> Result<()> {
    phonenumber::parse(None, user_phone).context("Validate phone number")?;
    debug!("Phone number is valid");
    Ok(())
}

/// Function that returns an [`InputContact`] given a `client_id` and `phone` number
///
/// # Errors
///
/// Returns [`Err`] if the phone number is incorrectly formatted or if the `client_id`
/// is larger than an [`i64`]
fn get_input_content((client_id, phone): (usize, String)) -> Result<InputContact> {
    phonenumber::parse(None, &phone).context("Validate phone number")?;
    let client_id = i64::try_from(client_id).context("Convert usize to i64")?;
    Ok(InputContact::InputPhoneContact(InputPhoneContact {
        client_id,
        phone,
        first_name: String::new(),
        last_name: String::new(),
    }))
}

/// Function that maps an [Iterator] of [User] (foreign type) into an [Iterator] of [`UserData`] (local)
fn into_user_data(users: impl Iterator<Item = User>) -> impl Iterator<Item = UserData> {
    users.into_iter().filter_map(|u| match u {
        tl::enums::User::Empty(_) => None,
        tl::enums::User::User(u) => Some(UserData::from(u)),
    })
}

/// This is the fundemental type this crate provides.
///
/// Provides an abstraction over a [`Client`] with methods to retrieve [`UserData`] from the
/// telegram API
///
pub struct UserRetriever<'a> {
    client: Client,
    sign_out: bool,
    session_file: &'a str,
}

impl<'a> UserRetriever<'a> {
    /// Returns a new [`UserRetriever`]
    ///
    /// # Errors
    ///
    /// Returns an [`Err`] if the underlying [`grammers_client::Client`] fails to build
    ///
    pub async fn new(api_id: i32, api_hash: String, session_file: &'a str) -> Result<Self> {
        let client = Client::connect(Config {
            session: Session::load_file_or_create(session_file).context("Load/Create session")?,
            api_id,
            api_hash,
            params: grammers_client::InitParams::default(),
        })
        .await
        .context("build Client")?;
        Ok(Self {
            client,
            sign_out: true,
            session_file,
        })
    }

    /// This method signs the requesting user in.
    ///
    /// In order to use this method, three closures must be provided:
    /// 1. `phone_input_handler`: A function to retrieve the requesting user's phone number from some
    ///    form of input
    /// 2. `code_input_handler`: A function to retrieve the requesting user's verification code from some
    ///    form of input (this happens after providing the user's phone)
    /// 3. `password_input_handler`: In the event that the user also has a password set up, this
    ///    function must retrieve the password from some form of input
    ///
    /// This method will also attempt to save the current session into the session file specified
    /// by [`UserRetriever::session_file`]
    ///
    /// # Errors
    ///
    /// Errors on
    /// 1) any failure from [`grammers_client::Client`]
    /// 2) if the handler functions return an [`Err`]
    pub async fn sign_in<PH, CH, PWH, R1, R2, R3>(
        &mut self,
        phone_input_handler: PH,
        code_input_handler: CH,
        password_input_handler: PWH,
    ) -> Result<()>
    where
        // Thre three handler functions
        PH: FnOnce() -> R1 + Send + 'static,
        CH: FnOnce() -> R2 + Send + 'static,
        PWH: FnOnce(String) -> R3 + Send + 'static,
        // Outputs of the three handler functions
        R1: Future<Output = Result<String>> + Send + 'static,
        R2: Future<Output = Result<String>> + Send + 'static,
        R3: Future<Output = Result<String>> + Send + 'static,
    {
        if self.client.is_authorized().await? {
            debug!("User is already authorized");
        } else {
            debug!("Awaiting input: user phone number");
            let phone = phone_input_handler().await?;
            validate(&phone)?;
            let token = self.client.request_login_code(&phone).await?;
            debug!("Awaiting input: user code");
            let code = code_input_handler().await?;
            let signed_in = self.client.sign_in(&token, &code).await;
            match signed_in {
                Err(SignInError::PasswordRequired(password_token)) => {
                    let hint = password_token.hint().unwrap_or("None").to_string();
                    debug!("Awaiting input: user password");
                    let password = password_input_handler(hint).await?;

                    self.client
                        .check_password(password_token, password.trim())
                        .await?;
                }
                Err(e) => anyhow::bail!("Unable to sign in: {e:?}"),
                Ok(_) => {}
            };
            debug!("Signed in!");
            self.try_save_session();
            debug!("Session saved");
        }
        Ok(())
    }

    /// Attempts to save the session to [`UserRetriever::session_file`], failing silently if unable
    pub fn try_save_session(&mut self) {
        match self.client.session().save_to_file(self.session_file) {
            Ok(()) => {
                self.sign_out = false;
            }
            Err(e) => {
                warn!(
                    "NOTE: failed to save the session to {}: {}",
                    self.session_file, e
                );
            }
        }
    }

    /// Requests to import the provided [`InputContact`]s into the requesting user's contacts
    ///
    /// # Note
    ///
    /// This method is the core of the [`UserRetriever`], our strategy of retrieving [`UserData`] requires we add the
    /// prvoided phone numbers to the requesting user's contacts
    async fn import_contacts(&self, contacts: Vec<InputContact>) -> Result<Vec<User>> {
        let ImportedContacts::Contacts(tl::types::contacts::ImportedContacts { users, .. }) = self
            .client
            .invoke(&tl::functions::contacts::ImportContacts { contacts })
            .await?;
        Ok(users)
    }

    /// A validated [`UserRetriever`] (see [`UserRetriever::sign_in`]) can call this method
    /// on an [Iterator] of phone numbers.
    ///
    /// # Errors
    ///
    /// Will return [Err] if the [`UserRetriever`] is not authorized or if the provided phone
    /// numbers are not correctly formated
    pub async fn get_users(&self, numbers: Vec<String>) -> Result<impl Iterator<Item = UserData>> {
        let Ok(contacts): Result<Vec<InputContact>> = numbers
            .into_iter()
            .enumerate()
            .map(get_input_content)
            .collect()
        else {
            anyhow::bail!("Unable to create contacts request from provided phone numbers.");
        };

        let users = self.import_contacts(contacts).await?;
        Ok(into_user_data(users.into_iter()))
    }

    /// Requests to delete the provided [`UserData`]s from the users contacts
    ///
    /// # Note
    ///
    /// This method is provided because our strategy of retrieving [`UserData`] requires we add the
    /// prvoided phone numbers to the requesting user's contacts
    ///
    /// # Errors
    ///
    /// Returns an [`Err`] if the underlying call to
    /// [`grammers_tl_types::functions::contacts::DeleteContacts`] fails
    pub async fn delete_contacts(&self, users: &[UserData]) -> Result<()> {
        debug!("Removing contacts");
        let delete_users: Vec<InputUser> = users.iter().map(InputUser::from).collect();

        let u = self
            .client
            .invoke(&tl::functions::contacts::DeleteContacts { id: delete_users })
            .await?;
        trace!("Updates: {u:?}");
        Ok(())
    }

    /// Consumes self and returns the internal [`grammers_client::Client`]
    #[must_use]
    pub fn into_inner(self) -> Client {
        self.client
    }
}
