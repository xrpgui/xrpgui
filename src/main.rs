mod encrypt;
mod mnemonic;
mod secret_numbers;

use encrypt::{decrypt_account, list_accounts, save_account};
use mnemonic::generate as generate_mnemonic;
use secret_numbers::generate as generate_secret_numbers;
use secret_numbers::validate as validate_secret_numbers;

fn main() -> eframe::Result<()> {
    let mut app = App::default();
    if let Ok(files) = list_accounts() {
        if !files.is_empty() {
            app.screen = Screen::DecryptAccount;
            app.account_files = files;
            app.selected_account = app.account_files[0].clone();
        }
    }
    eframe::run_native(
        "xrpgui",
        eframe::NativeOptions::default(),
        Box::new(move |_cc| Ok(Box::new(app))),
    )
}

enum Screen {
    Welcome,
    CreateAccountFromSecretNumbers,
    CreateAccountFromMnemonic,
    ConfirmSecretNumbers,
    EncryptAccount,
    DecryptAccount,
    Overview,
    // ConfirmMnemonic,
    // ImportAccountFromSecretNumbers,
    // ImportAccountFromMnemonic,
    // Settings,
}

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Overview,
    Receive,
    Send,
    Settings,
}

struct App {
    screen: Screen,
    mnemonic: String,
    secret_numbers: String,
    confirmed_secret_numbers: String,
    account_name: String,
    password: String,
    confirm_password: String,
    decrypt_password: String,
    account_files: Vec<String>,
    selected_account: String,
    tab: Tab,
    address: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::Welcome,
            mnemonic: String::new(),
            secret_numbers: String::new(),
            confirmed_secret_numbers: String::new(),
            account_name: String::new(),
            password: String::new(),
            confirm_password: String::new(),
            decrypt_password: String::new(),
            account_files: Vec::new(),
            tab: Tab::Overview,
            selected_account: String::new(),
            address: String::new(),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match self.screen {
            Screen::Welcome => self.welcome_screen(ui),
            Screen::CreateAccountFromSecretNumbers => {
                self.create_account_from_secret_numbers_screen(ui)
            }
            Screen::CreateAccountFromMnemonic => self.create_account_from_mnemonic_screen(ui),
            Screen::ConfirmSecretNumbers => self.confirm_secret_numbers_screen(ui),
            Screen::EncryptAccount => self.encrypt_account_screen(ui),
            Screen::DecryptAccount => self.decrypt_account_screen(ui),
            Screen::Overview => self.overview_screen(ui),
        }
    }
}

impl App {
    fn welcome_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            if ui.button("Create a new account").clicked() {
                self.switch_screen(Screen::CreateAccountFromSecretNumbers);
            }
            if ui.button("Import an existing account").clicked() {}
        });
    }

    fn switch_screen(&mut self, screen: Screen) {
        self.screen = screen;
        match self.screen {
            Screen::Welcome => {}
            Screen::CreateAccountFromSecretNumbers => match generate_secret_numbers() {
                Ok(g) => {
                    self.secret_numbers = g.secret_numbers;
                    self.address = g.address;
                }
                Err(e) => self.address = format!("Error: {}", e),
            },
            Screen::CreateAccountFromMnemonic => match generate_mnemonic() {
                Ok(g) => {
                    self.mnemonic = g.mnemonic;
                    self.address = g.address;
                }
                Err(e) => self.address = format!("Error: {}", e),
            },
            Screen::ConfirmSecretNumbers => {
                self.confirmed_secret_numbers = String::new();
            }
            Screen::EncryptAccount => {}
            Screen::DecryptAccount => {
                self.decrypt_password = String::new();
                if let Ok(files) = list_accounts() {
                    self.account_files = files;
                }
                if !self.account_files.is_empty() && self.selected_account.is_empty() {
                    self.selected_account = self.account_files[0].clone();
                }
            }
            Screen::Overview => {}
        }
    }

    fn create_account_from_secret_numbers_screen(&mut self, ui: &mut egui::Ui) {
        ui.label("Secret Numbers:");
        ui.add(egui::Label::new(&self.secret_numbers).selectable(true));

        ui.label("Address:");
        ui.add(egui::Label::new(&self.address).selectable(true));

        ui.horizontal(|ui| {
            if ui.link("Use Mnemonic").clicked() {
                self.switch_screen(Screen::CreateAccountFromMnemonic);
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                self.screen = Screen::Welcome;
            }
            if ui.button("Next").clicked() {
                self.switch_screen(Screen::ConfirmSecretNumbers);
            }
        });
    }

    fn confirm_secret_numbers_screen(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::TextEdit::multiline(&mut self.confirmed_secret_numbers)
                .hint_text("123456 123456 123456 123456 123456 123456 123456 123456"),
        );

        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                self.screen = Screen::CreateAccountFromSecretNumbers;
            }
            if ui.button("Next").clicked() {
                if validate_secret_numbers(&self.confirmed_secret_numbers, &self.secret_numbers) {
                    println!("ok");
                    self.switch_screen(Screen::EncryptAccount);
                } else {
                    println!("ng");
                }
            }
        });
    }

    fn encrypt_account_screen(&mut self, ui: &mut egui::Ui) {
        ui.label("Account Name:");
        ui.add(egui::TextEdit::singleline(&mut self.account_name));

        ui.label("Password:");
        ui.add(egui::TextEdit::singleline(&mut self.password).password(true));

        ui.label("Confirm Password:");
        ui.add(egui::TextEdit::singleline(&mut self.confirm_password).password(true));

        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                self.screen = Screen::ConfirmSecretNumbers;
            }
            if ui.button("Next").clicked() {
                if self.password != self.confirm_password {
                    println!("Passwords do not match");
                } else if self.account_name.trim().is_empty() {
                    println!("Account name is empty");
                } else {
                    match save_account(
                        &self.account_name,
                        &self.address,
                        &self.secret_numbers,
                        &self.password,
                    ) {
                        Ok(()) => {
                            println!("Account saved");
                            self.switch_screen(Screen::DecryptAccount);
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }
        });
    }

    fn create_account_from_mnemonic_screen(&mut self, ui: &mut egui::Ui) {
        ui.label("Mnemonic:");
        ui.add(egui::Label::new(&self.mnemonic).selectable(true));

        ui.label("Address:");
        ui.add(egui::Label::new(&self.address).selectable(true));

        ui.horizontal(|ui| {
            if ui.link("Use Secret Numbers").clicked() {
                self.switch_screen(Screen::CreateAccountFromSecretNumbers);
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                self.screen = Screen::Welcome;
            }
            if ui.button("Next").clicked() {}
        });
    }

    fn decrypt_account_screen(&mut self, ui: &mut egui::Ui) {
        ui.label("Account File:");
        egui::ComboBox::from_label("")
            .selected_text(&self.selected_account)
            .show_ui(ui, |ui| {
                for file in &self.account_files {
                    ui.selectable_value(&mut self.selected_account, file.clone(), file);
                }
            });

        ui.label("Password:");
        ui.add(
            egui::TextEdit::singleline(&mut self.decrypt_password)
                .password(true)
                .hint_text("Password"),
        );

        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                self.screen = Screen::EncryptAccount;
            }
            if ui.button("Next").clicked() {
                match decrypt_account(&self.selected_account, &self.decrypt_password) {
                    Ok((name, address)) => {
                        self.account_name = name;
                        self.address = address;
                        self.switch_screen(Screen::Overview);
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
        });
    }

    fn overview_screen(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tab, Tab::Overview, "Overview");
            ui.selectable_value(&mut self.tab, Tab::Receive, "Receive");
            ui.selectable_value(&mut self.tab, Tab::Send, "Send");
            ui.selectable_value(&mut self.tab, Tab::Settings, "Settings");
        });

        match self.tab {
            Tab::Overview => {
                ui.label("Account Name:");
                ui.add(egui::Label::new(&self.account_name).selectable(true));

                ui.label("Address:");
                ui.add(egui::Label::new(&self.address).selectable(true));
            }
            Tab::Receive => {
                ui.label("Receive");
            }
            Tab::Send => {
                ui.label("Send");
            }
            Tab::Settings => {
                ui.label("Settings");
            }
        }
    }
}
