use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use surf::StatusCode;
use uuid::Uuid;
use std::fs::File;
use std::io::Read;
use sha2::{Digest, Sha256};
use base64::{encode};

pub fn generate_hex_string(length: usize) -> String {
    let rng = thread_rng();
    let random_bytes: Vec<u8> = rng
        .sample_iter(&Alphanumeric)
        .take(length / 2)
        .collect();

    hex::encode(random_bytes)
}

pub fn format_uuid(uuid_in: String) -> tide::Result<String>{
    let uuid = Uuid::parse_str(&uuid_in)
        .map_err(|_| tide::Error::from_str(StatusCode::InternalServerError, "Failed to parse UUID"))?;
    let uuid_with_dashes = uuid.as_hyphenated().to_string();
    Ok(uuid_with_dashes)
}

pub fn calculate_file_sha256(file_path: &str) -> Result<String, std::io::Error> {
    // Read the file content
    let mut file = File::open(file_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    // Convert the content to base64
    let base64_content = encode(&content);

    // Calculate the SHA-256 hash of the base64 string
    let hash = Sha256::digest(base64_content.as_bytes());

    // Convert the hash to a hexadecimal string
    let hex_hash = hash.iter().map(|byte| format!("{:02x}", byte)).collect::<String>();

    Ok(hex_hash)
}