mod encrypt;
mod mnemonic;
mod qr;
mod secret_numbers;

use encrypt::{decrypt_account, list_accounts, save_account};
use mnemonic::generate as generate_mnemonic;
use qr::render_qr_texture;
use secret_numbers::generate as generate_secret_numbers;
use secret_numbers::validate as validate_secret_numbers;

fn xrp_amount(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => {
            let drops: f64 = s.parse().unwrap_or(0.0);
            format!("{:.6} XRP", drops / 1_000_000.0)
        }
        serde_json::Value::Object(obj) => obj
            .get("currency")
            .and_then(|c| c.as_str())
            .unwrap_or("?")
            .to_string(),
        _ => "?".to_string(),
    }
}

fn unix_to_iso(secs: i64) -> String {
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let (h, rem2) = (rem / 3600, rem % 3600);
    let (m, s) = (rem2 / 60, rem2 % 60);
    format!("{}d {:02}:{:02}:{:02}", days, h, m, s)
}

struct Transaction {
    network: String,
    date: String,
    direction: String,
    amount: String,
    tx_type: String,
    from: String,
    to: String,
    ledger: String,
    hash: String,
}

fn tx_from_json(network: &str, address: &str, tx_json: &serde_json::Value, ledger: u32) -> Transaction {
    let hash = tx_json
        .get("hash")
        .and_then(|h| h.as_str())
        .unwrap_or("")
        .to_string();
    let date = tx_json
        .get("date")
        .and_then(|d| d.as_i64())
        .map(unix_to_iso)
        .unwrap_or_default();
    let ttype = tx_json
        .get("TransactionType")
        .and_then(|x| x.as_str())
        .unwrap_or("?")
        .to_string();
    let from = tx_json
        .get("Account")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let to = tx_json
        .get("Destination")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let amount = tx_json
        .get("Amount")
        .or_else(|| tx_json.get("DeliverMax"))
        .map(xrp_amount)
        .unwrap_or_default();

    let direction = if to == address {
        "receive".to_string()
    } else if from == address {
        "send".to_string()
    } else {
        "other".to_string()
    };

    Transaction {
        network: network.to_string(),
        date,
        direction,
        amount,
        tx_type: ttype,
        from,
        to,
        ledger: ledger.to_string(),
        hash,
    }
}

fn fetch_transactions(address: &str) -> Vec<Transaction> {
    use xrpl::clients::{XRPLSyncClient, json_rpc::JsonRpcClient};
    use xrpl::models::requests::account_tx::AccountTx;

    let url = "https://s.altnet.rippletest.net:51234";
    let network = "Testnet";
    let client = match JsonRpcClient::connect(url.parse().expect("valid url")) {
        c => c,
    };

    let req = AccountTx::new(
        None,
        address.into(),
        None,
        None,
        Some(false),
        None,
        None,
        None,
        Some(20),
        None,
    );

    let mut rows = Vec::new();
    match client.request(req.into()) {
        Ok(resp) => {
            use xrpl::models::results::account_tx::AccountTxVersionMap;
            let map: AccountTxVersionMap = match resp.try_into() {
                Ok(m) => m,
                Err(e) => {
                    println!("Error parsing account_tx: {:?}", e);
                    return rows;
                }
            };
            let account_tx = match &map {
                AccountTxVersionMap::Default(t) => {
                    let mut v = Vec::new();
                    for tx in &t.base.transactions {
                        let ledger = tx
                            .base
                            .ledger_index
                            .unwrap_or_default();
                        let mut tx_json = tx.tx_json.clone().unwrap_or_default();
                        tx_json["hash"] = serde_json::Value::String(tx.hash.to_string());
                        if tx_json.get("date").is_none() {
                            tx_json["date"] = serde_json::Value::Null;
                        }
                        v.push(tx_from_json(network, address, &tx_json, ledger));
                    }
                    v
                }
                AccountTxVersionMap::V1(t) => {
                    let mut v = Vec::new();
                    for tx in &t.base.transactions {
                        let ledger = tx
                            .base
                            .ledger_index
                            .unwrap_or_default();
                        let tx_json = tx.tx.clone().unwrap_or_default();
                        v.push(tx_from_json(network, address, &tx_json, ledger));
                    }
                    v
                }
            };
            println!("Got {} transactions for {}", account_tx.len(), address);
            rows = account_tx;
        }
        Err(e) => println!("Error requesting account_tx: {:?}", e),
    }
    rows
}

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
    TransactionDetails,
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
    transactions: Vec<Transaction>,
    tx_loaded: bool,
    selected_tx: Option<usize>,
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
            transactions: Vec::new(),
            tx_loaded: false,
            selected_tx: None,
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
            Screen::TransactionDetails => self.transaction_details_screen(ui),
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
            Screen::TransactionDetails => {}
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

                ui.separator();
                ui.label("Transactions:");
                if !self.tx_loaded {
                    self.transactions = fetch_transactions(&self.address);
                    self.tx_loaded = true;
                }
                if self.transactions.is_empty() {
                    ui.label("No transactions.");
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for (i, tx) in self.transactions.iter().enumerate() {
                                let color = if tx.direction == "receive" {
                                    egui::Color32::GREEN
                                } else if tx.direction == "send" {
                                    egui::Color32::RED
                                } else {
                                    egui::Color32::GRAY
                                };
                                let frame = egui::Frame::group(ui.style())
                                    .inner_margin(egui::Margin::same(8));
                                frame
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(&tx.date)
                                                    .strong(),
                                            );
                                            ui.label(
                                                egui::RichText::new(&tx.direction)
                                                    .color(color)
                                                    .strong(),
                                            );
                                            ui.label(egui::RichText::new(&tx.amount).strong());
                                        });
                                    })
                                    .response
                                    .interact(egui::Sense::click())
                                    .clicked();
                                if ui.interact(
                                    ui.max_rect(),
                                    ui.id().with(i),
                                    egui::Sense::click(),
                                )
                                .clicked()
                                {
                                    self.selected_tx = Some(i);
                                    self.screen = Screen::TransactionDetails;
                                }
                            }
                        });
                }
            }
            Tab::Receive => {
                ui.label("Your address:");
                ui.add(
                    egui::Label::new(egui::RichText::new(&self.address).monospace())
                        .selectable(true),
                );

                ui.add_space(12.0);
                if let Some(texture) = render_qr_texture(ui.ctx(), &self.address, 200) {
                    ui.add(
                        egui::Image::new(&texture)
                            .fit_to_exact_size(egui::Vec2::splat(200.0)),
                    );
                }
            }
            Tab::Send => {
                ui.label("Send");
            }
            Tab::Settings => {
                ui.label("Settings");
            }
        }
    }

    fn transaction_details_screen(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                self.screen = Screen::Overview;
            }
        });

        if let Some(idx) = self.selected_tx {
            if let Some(tx) = self.transactions.get(idx) {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("tx_details")
                        .num_columns(2)
                        .spacing([20.0, 8.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Network");
                            ui.label(&tx.network);
                            ui.end_row();

                            ui.label("Date");
                            ui.label(&tx.date);
                            ui.end_row();

                            ui.label("Direction");
                            ui.label(&tx.direction);
                            ui.end_row();

                            ui.label("Type");
                            ui.label(&tx.tx_type);
                            ui.end_row();

                            ui.label("From");
                            ui.add(egui::Label::new(&tx.from).selectable(true));
                            ui.end_row();

                            ui.label("To");
                            ui.add(egui::Label::new(&tx.to).selectable(true));
                            ui.end_row();

                            ui.label("Amount");
                            ui.label(&tx.amount);
                            ui.end_row();

                            ui.label("Ledger");
                            ui.label(&tx.ledger);
                            ui.end_row();

                            ui.label("Hash");
                            ui.add(egui::Label::new(&tx.hash).selectable(true));
                            ui.end_row();
                        });
                });
            }
        }
    }
}
