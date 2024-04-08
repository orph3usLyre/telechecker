#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use anyhow::{Context, Result};
use eframe::egui;
use std::{env, thread};
use telechecker_lib::{user::UserData, validate, UserRetriever, SESSION_FILE};
use tokio::sync::oneshot::Receiver;
use tracing::{debug, info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BASE_APP_SIZE: [f32; 2] = [320.0, 440.0];
pub const MIN_APP_SIZE: [f32; 2] = [220.0, 340.0];

const API_ID_ENV: &str = "API_ID";
const API_HASH_ENV: &str = "API_HASH";
const DEFAULT_OUTPUT_FILE: &str = "results.json";
const CHANNEL_BUFFER_SIZE: usize = 5;
const USER_PHONE_INPUT_ERROR: &str = "Unable to validate phone number. Check that your phone number is correctly formatted as an international phone number (i.e. +1-(404)-123-4567)";

#[cfg(not(debug_assertions))]
const LOG_FILE: &str = concat!(env!("CARGO_PKG_NAME"), ".log");

mod app;
mod client;
mod comms;
mod ui;

use app::{ConnectionStatus, Telegather};
use client::client_handler;
use comms::create_comms_channels;

fn main() -> Result<()> {
    // Sets the directory of the current process to the directory of the executable,
    // so that the `.env` will always be read from where the program is
    #[cfg(not(debug_assertions))]
    std::env::set_current_dir(
        std::env::current_exe()
            .context("couldn't get current_exe")?
            .parent()
            .context("couldn't find parent to exe")?,
    )
    .context("couldn't set current dir")?;

    // remove colors from tracing_subscriber output if windows
    #[cfg(windows)]
    let colors = false;
    #[cfg(not(windows))]
    let colors = true;

    // set up tracing
    let default_env_filter = EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(|_| {
        format!(
            "telegather_lib=trace,{}={}",
            env!("CARGO_PKG_NAME").replace('-', "_"),
            "trace"
        )
    }));

    // we write to the console in debug mode, otherwise we write to LOG_FILE
    #[cfg(debug_assertions)]
    tracing_subscriber::registry()
        .with(fmt::layer().with_ansi(colors).without_time())
        .with(default_env_filter)
        .init();

    #[cfg(not(debug_assertions))]
    let log_file = std::fs::File::options()
        .truncate(true)
        .read(true)
        .write(true)
        .create(true)
        .open(LOG_FILE)?;

    #[cfg(not(debug_assertions))]
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_ansi(colors)
                .without_time()
                .with_writer(std::sync::Arc::new(log_file)),
        )
        .with(default_env_filter)
        .init();

    info!("App: {APP_NAME}: Version: {APP_VERSION}");

    // dotenv
    match dotenvy::dotenv().context("Load .env") {
        Ok(_) => debug!(".env file loaded"),
        Err(_) => warn!("Unable to find .env file"),
    }
    debug!("Reading api_id and api_hash from .env");

    // try get api_id and api_hash from env
    let (api_id, api_hash) = (
        env::var(API_ID_ENV)
            .map(|id| id.parse::<i32>().ok())
            .ok()
            .unwrap_or(None),
        env::var(API_HASH_ENV).ok(),
    );

    // comms to allow app thread to communicate with async thread
    let (comms_app, comms_client, client_exit_error_tx) = create_comms_channels();

    // spawn the async runtime thread
    let t = thread::spawn(move || {
        match client_handler(api_id, api_hash, SESSION_FILE, comms_client) {
            Ok(_) => debug!("Client handler successfully exited"),
            Err(e) => {
                warn!("Client handler exited with an error: {e:?}");
                if client_exit_error_tx.send(e).is_err() {
                    debug!("App already exited");
                };
            }
        };
    });

    // eframe options (egui)
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(BASE_APP_SIZE)
            .with_min_inner_size(MIN_APP_SIZE),
        ..Default::default()
    };

    match eframe::run_native(
        "telegatherer",
        options,
        Box::new(|_| Box::new(Telegather::new(comms_app))),
    ) {
        Ok(_) => (),
        Err(e) => anyhow::bail!("Encountered eframe error: {e:?}"),
    }

    let _ = t.join();
    Ok(())
}

impl eframe::App for Telegather {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let connection_status = self.comm_channels.connection_status_rx.borrow_and_update();
        if connection_status.has_changed() {
            self.connection_status = *connection_status;
        }
        drop(connection_status);

        if let Ok(e) = self.comm_channels.client_exit_error_rx.try_recv() {
            self.error_message = Some(format!(
                "Client exited with an error: {e:?}\nPlease restart the app and try again"
            ));
        }

        self.top_ui(ctx);
        self.central_ui(ctx);
        self.bottom_ui(ctx);
    }
}
