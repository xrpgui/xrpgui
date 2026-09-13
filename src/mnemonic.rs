use bip32::{DerivationPath, XPrv};
use bip39::Mnemonic;
use xrpl::core::addresscodec::encode_classic_address;
use xrpl::core::keypairs::utils::get_account_id;

pub struct Generated {
    pub mnemonic: String,
    pub address: String,
}

pub fn generate() -> Result<Generated, String> {
    let mnemonic = Mnemonic::generate(12).map_err(|e| e.to_string())?;
    let phrase = mnemonic.words().collect::<Vec<_>>().join(" ");

    let seed = mnemonic.to_seed("");
    let path: DerivationPath = "m/44'/144'/0'/0/0"
        .parse()
        .map_err(|e: bip32::Error| e.to_string())?;
    let xprv = XPrv::derive_from_path(&seed, &path).map_err(|e| e.to_string())?;
    let xpub = xprv.public_key();
    let public_key = xpub.to_bytes();
    let account_id = get_account_id(&public_key);
    let address = encode_classic_address(&account_id).map_err(|e| e.to_string())?;

    Ok(Generated {
        mnemonic: phrase,
        address,
    })
}
