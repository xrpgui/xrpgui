use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use argon2::{Argon2, PasswordHasher};
use serde::Serialize;

#[derive(Serialize)]
struct AccountData<'a> {
    account_name: &'a str,
    address: &'a str,
    secret_numbers: &'a str,
}

fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password_with_salt(password.as_bytes(), salt)
        .map_err(|e| e.to_string())?;
    let hash_bytes = hash.hash.as_ref().ok_or("no hash")?.as_bytes();
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash_bytes[..32]);
    Ok(key)
}

pub fn save_account(
    account_name: &str,
    address: &str,
    secret_numbers: &str,
    password: &str,
) -> Result<(), String> {
    if account_name.is_empty() {
        return Err("Account name is empty".to_string());
    }

    let data = AccountData {
        account_name,
        address,
        secret_numbers,
    };
    let plaintext = serde_json::to_string(&data).map_err(|e| e.to_string())?;

    let salt = [0u8; 16];
    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    let nonce_bytes = [0u8; 12];
    let nonce = Nonce::try_from(nonce_bytes).map_err(|e| e.to_string())?;
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| e.to_string())?;

    let dir = data_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let path = dir.join(account_name);
    std::fs::write(&path, ciphertext).map_err(|e| e.to_string())?;

    println!("Saved encrypted account to {}", path.display());
    Ok(())
}

fn data_dir() -> Result<std::path::PathBuf, String> {
    Ok(dirs::data_dir()
        .ok_or("cannot determine data dir")?
        .join("xrpgui"))
}

pub fn list_accounts() -> Result<Vec<String>, String> {
    let dir = data_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_file() {
            if let Some(name) = entry.file_name().to_str() {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}
