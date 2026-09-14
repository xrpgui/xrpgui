use xrpl::constants::CryptoAlgorithm;
use xrpl::core::addresscodec::utils::SEED_LENGTH;
use xrpl::core::keypairs::derive_classic_address;
use xrpl::core::keypairs::derive_keypair;
use xrpl::core::keypairs::generate_seed;

pub struct Generated {
    pub secret_numbers: String,
    pub address: String,
}

pub fn validate(input: &str, original: &str) -> bool {
    let normalized_input: String = input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let normalized_original: String = original
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    normalized_input == normalized_original
}

pub fn generate() -> Result<Generated, String> {
    loop {
        match try_generate() {
            Ok(generated) => return Ok(generated),
            Err(_) => continue,
        }
    }
}

fn try_generate() -> Result<Generated, String> {
    let seed = generate_seed(None, Some(CryptoAlgorithm::SECP256K1)).map_err(|e| e.to_string())?;
    let entropy = seed_to_entropy(&seed)?;
    let (public, _private) = derive_keypair(&seed, false).map_err(|e| e.to_string())?;
    let address = derive_classic_address(&public).map_err(|e| e.to_string())?;
    Ok(Generated {
        secret_numbers: entropy_to_secret(&entropy),
        address,
    })
}

fn seed_to_entropy(seed: &str) -> Result<[u8; SEED_LENGTH], String> {
    let (entropy, _algo) =
        xrpl::core::addresscodec::decode_seed(seed).map_err(|e| e.to_string())?;
    Ok(entropy)
}

fn calculate_checksum(position: usize, value: u32) -> u32 {
    (value * (position as u32 * 2 + 1)) % 9
}

fn entropy_to_secret(entropy: &[u8; SEED_LENGTH]) -> String {
    let mut chunks = Vec::with_capacity(8);
    for i in 0..8 {
        let no = ((entropy[i * 2] as u32) << 8) | entropy[i * 2 + 1] as u32;
        let fill = "0".repeat(5 - no.to_string().len());
        let checksum = calculate_checksum(i, no);
        chunks.push(format!("{}{}{}", fill, no, checksum));
    }
    chunks.join(" ")
}
