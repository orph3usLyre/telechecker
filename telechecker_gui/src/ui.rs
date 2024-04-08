use egui::{Color32, Label, Layout, TextEdit};
use std::{fs::File, io::Write};
use tracing::{error, info, warn};

use crate::{
    validate, ConnectionStatus, Telegather, APP_VERSION, DEFAULT_OUTPUT_FILE,
    USER_PHONE_INPUT_ERROR,
};

const CONFIRM_BUTTON_TEXT: &str = "confirm";
const GREEN_CHECK_EMOJI: &str = "✅";
// const EDIT_EMOJI: &str = "📝";

impl Telegather {
    pub fn top_ui(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("Top Menu").show(ctx, |ui| {
            ui.columns(3, |columns| {
                columns[0].vertical(|ui| {
                    egui::global_dark_light_mode_switch(ui);
                });

                columns[1].vertical_centered(|_| {});

                columns[2].vertical(|ui| {
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(Color32::DARK_GRAY, APP_VERSION);
                    });
                });
            });
        });
    }

    pub fn central_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.connection_status {
                ConnectionStatus::RequiresApiInfo => {
                    ui.label("Telechecker requires API info:");
                    ui.horizontal(|ui| {
                        ui.label("API id:      ");
                        ui.add(TextEdit::singleline(&mut self.cache.api_id)
                            .desired_width(ui.available_width() / 2.)
                            .hint_text("<API_ID>")
                            .password(true)
                        );
                    });
                    ui.add_space(5.);
                    ui.horizontal(|ui| {
                        ui.end_row();
                        ui.label("API hash: ");
                        ui.add(TextEdit::singleline(&mut self.cache.api_hash)
                            .desired_width(ui.available_width() / 1.5)
                            .hint_text("<API_HASH>")
                            .password(true)
                        );
                    });
                    ui.vertical_centered(|ui| {
                        if !self.cache.api_id.is_empty() && !self.cache.api_hash.is_empty() {
                            if ui.small_button("Save to .env").clicked() {
                                if let Ok(api_id) = self.cache.api_id.parse::<i32>() {
                                if let Ok(mut env) = File::options()
                                    .write(true)
                                    .truncate(true)
                                    .create(true)
                                    .open(".env") {
                                    let buf = format!("API_ID='{}'\nAPI_HASH='{}'", api_id, self.cache.api_hash);
                                    match env.write_all(buf.as_bytes()) {
                                        Ok(()) => self.info_message = Some(".env saved".into()),
                                        Err(e) => self.error_message = Some(format!("Unable to save .env. Error: {e}")),
                                    }

                                }
                                }
                            }
                            if ui.small_button(GREEN_CHECK_EMOJI).clicked() {

                                if let Ok(api_id) = self.cache.api_id.parse::<i32>() {
                                    if let Some(sender) = self.comm_channels.api_info_tx.take() {
                                        if let Err(e) = sender.send((api_id, self.cache.api_hash.clone())) {
                                            let error = format!("Unable to send user password. Is the user already authorized?: {e:?}");
                                            warn!(error);
                                            self.error_message = Some(error);
                                        } else {
                                            self.info_message = Some("API info sent".into());
                                        }
                                        self.cache.user_pwd.clear();
                                    }

                                } else {
                                    self.info_message = Some(format!("Invalid API_ID: '{}'", self.cache.api_id));
                                }
                            }
                        }
                    });
                }
                ConnectionStatus::NotConnected | ConnectionStatus::AwaitingPhoneNumber=> {
                    // Handle phone input
                    if self.user_phone.is_none() {
                        ui.vertical(|ui| {
                            ui.label("Input your telegram-associated phone number:");
                        });
                        ui.horizontal(|ui| {
                            ui.add(
                                TextEdit::singleline(&mut self.cache.user_phone)
                                    .desired_width(ui.available_width() / 2.)
                                    .hint_text("i.e. +1223456789")
                                    .password(self.config.hide_phone),
                            );
                            if ui.small_button(GREEN_CHECK_EMOJI).clicked() {
                                match validate(&self.cache.user_phone) {
                                    Err(_) => {
                                        self.error_message =
                                            Some(USER_PHONE_INPUT_ERROR.to_string())
                                    }
                                    Ok(()) => self.user_phone = Some(self.cache.user_phone.clone()),
                                }
                            }
                        });
                    }
                    if let Some(p) = self.user_phone.as_ref() {
                        if let Some(sender) = self.comm_channels.user_phone_tx.take() {
                            if let Err(e) = sender.send(p.clone()) {
                                warn!("Unable to send user phone. Is the user already authorized?: {e:?}");
                            } else {
                                self.info_message = Some("Phone number sent, awaiting response...".to_string());
                            }
                        }
                    }
                }

                ConnectionStatus::AwaitingUserCode => {
                    ui.label("Enter the code when you receive the text:");
                    ui.horizontal(|ui| {
                        ui.add(
                            TextEdit::singleline(&mut self.cache.user_code)
                                .password(self.config.hide_code)
                                .desired_width(ui.available_width() / 3.),
                        );
                        if ui.small_button(GREEN_CHECK_EMOJI).clicked() {
                            if let Some(sender) = self.comm_channels.user_code_tx.take() {
                                if let Err(e) = sender.send(self.cache.user_code.clone()) {
                                    warn!("Unable to send user code. Is the user already authorized?: {e:?}");
                                } else {
                                    self.info_message = Some("Code sent, awaiting response...".to_string());
                                }
                                self.cache.user_code.clear();
                            }
                        }
                    });
                }
                ConnectionStatus::AwaitingPassword => {
                    ui.label("Password required:");
                    ui.horizontal(|ui| {
                        ui.add(
                            TextEdit::singleline(&mut self.cache.user_pwd)
                                .desired_width(ui.available_width() / 2.)
                                .password(self.config.hide_pass),
                        );
                        if ui.small_button(GREEN_CHECK_EMOJI).clicked() {
                            if let Some(sender) = self.comm_channels.pass_receive_tx.take() {
                                if let Err(e) = sender.send(self.cache.user_pwd.clone()) {
                                    warn!("Unable to send user password. Is the user already authorized?: {e:?}");
                                } else {
                                    self.info_message = Some("Password sent, awaiting response...".to_string());
                                }
                                self.cache.user_pwd.clear();
                            }
                        }
                    });
                }
                ConnectionStatus::Authorized => {
                    ui.vertical_centered(|ui| {
                        ui.label("Please input target phone numbers:");
                        ui.text_edit_multiline(&mut self.cache.phones_input);
                        if ui.button(CONFIRM_BUTTON_TEXT).clicked() {
                            // validate
                            let phonenumbers = self
                                .cache
                                .phones_input
                                .lines()
                                .filter_map(|n| {
                                    if validate(n).is_ok() {
                                        Some(n.to_string())
                                    } else {
                                        warn!("Phone number '{n}' is invalid");
                                        None
                                    }
                                })
                                .collect();
                            if let Err(e) = self.comm_channels
                                .input_phones_tx
                                .blocking_send(phonenumbers) {
                                error!("Encountered error sending input phone numbers: {e:?}");
                            }
                        }
                    });
                }
            }

            if let Ok(user_data) = self.comm_channels.user_data_rx.try_recv() {
                self.user_data = Some(user_data);
            }

            if let Some(ref mut user_data)  = self.user_data  {
                if !user_data.is_empty() {
                    ui.separator();
                    ui.add_space(5.);
                        match serde_json::to_string_pretty(user_data) {
                            Ok(out) => {
                                info!("Writing output to '{DEFAULT_OUTPUT_FILE}'");
                                if let Ok(mut file) = File::options()
                                    .write(true)
                                    .create(true)
                                    .truncate(true)
                                    .open(DEFAULT_OUTPUT_FILE)
                                {
                                    if file.write_all(out.as_bytes()).is_err() {
                                        let fail = format!("Unable to write to file '{DEFAULT_OUTPUT_FILE}'");
                                        warn!(fail);
                                        self.info_message = Some(fail);
                                        user_data.clear();
                                    } else {
                                        let success = format!("Results saved as '{DEFAULT_OUTPUT_FILE}'");
                                        info!(success);
                                        self.info_message = Some(success);
                                        user_data.clear();
                                    }
                                }
                            }
                            Err(_) => error!("Failed to serialize UserData"),
                        }
                }
            }

            if let Some(ref info_message) = self.info_message {
                ui.add_space(10.);
                ui.separator();
                ui.label(egui::RichText::new("Info: ").underline().strong());
                ui.add(Label::new(egui::RichText::new(info_message)).wrap(true));
                ui.add_space(5.);
                    if ui.small_button("clear").clicked() {
                        self.info_message = None;
                    }
            }
            if let Some(ref error_message) = self.error_message {
                ui.add_space(10.);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Error: ")
                            .underline()
                            .strong()
                            .color(Color32::DARK_RED),
                    );
                    ui.add(Label::new(egui::RichText::new(error_message)).wrap(true));
                });
                ui.add_space(5.);
                    if ui.small_button("clear").clicked() {
                        self.error_message = None;
                    }
            }
        });
    }

    pub fn bottom_ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom panel").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label("Status: ");
                let as_str = Into::<&'static str>::into(self.connection_status);
                match self.connection_status {
                    ConnectionStatus::RequiresApiInfo => {
                        ui.colored_label(Color32::GRAY, as_str);
                    }
                    ConnectionStatus::NotConnected => {
                        ui.colored_label(Color32::DARK_GRAY, as_str);
                    }
                    ConnectionStatus::AwaitingPhoneNumber
                    | ConnectionStatus::AwaitingUserCode
                    | ConnectionStatus::AwaitingPassword => {
                        ui.colored_label(Color32::YELLOW, as_str);
                    }

                    ConnectionStatus::Authorized => {
                        ui.colored_label(Color32::GREEN, as_str);
                    }
                }
            });
        });
    }
}
