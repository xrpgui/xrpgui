mod mnemonic;
mod secret_numbers;

use mnemonic::generate as generate_mnemonic;
use secret_numbers::generate as generate_secret_numbers;
use secret_numbers::validate as validate_secret_numbers;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "xrpgui",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}

enum Screen {
    Welcome,
    CreateAccountFromSecretNumbers,
    CreateAccountFromMnemonic,
    ConfirmSecretNumbers,
    EncryptAccount,
    // ConfirmMnemonic,
    // ImportAccountFromSecretNumbers,
    // ImportAccountFromMnemonic,
    // EncryptAccount,
    // DecryptAccount,
    // Overview,
    // Send,
    // Receive,
    // Settings,
}

struct App {
    screen: Screen,
    mnemonic: String,
    secret_numbers: String,
    confirmed_secret_numbers: String,
    account_name: String,
    password: String,
    confirm_password: String,
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
            if ui.button("Next").clicked() {}
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
}
