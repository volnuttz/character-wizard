//! Versioned, bounded character share-code encoding.

use crate::character_wizard_domain::Character;

const PREFIX: &str = "cw1:";
const MAX_CODE_BYTES: usize = 256 * 1024 + PREFIX.len();
const MAX_JSON_BYTES: usize = 192 * 1024;
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encode a character as a versioned, URL-safe, unpadded share code.
///
/// # Errors
///
/// Returns an error if serialization fails or exceeds the supported size.
pub fn encode(character: &Character) -> Result<String, String> {
    let source = serde_json::to_vec(character).map_err(|error| error.to_string())?;
    if source.len() > MAX_JSON_BYTES {
        return Err(format!(
            "character JSON exceeds the {} KiB share limit",
            MAX_JSON_BYTES / 1024
        ));
    }
    let mut output = String::with_capacity(PREFIX.len() + source.len().div_ceil(3) * 4);
    output.push_str(PREFIX);
    for chunk in source.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(char::from(ALPHABET[usize::from(first >> 2)]));
        output.push(char::from(
            ALPHABET[usize::from(((first & 0x03) << 4) | (second >> 4))],
        ));
        if chunk.len() > 1 {
            output.push(char::from(
                ALPHABET[usize::from(((second & 0x0f) << 2) | (third >> 6))],
            ));
        }
        if chunk.len() > 2 {
            output.push(char::from(ALPHABET[usize::from(third & 0x3f)]));
        }
    }
    Ok(output)
}

/// Decode and structurally validate an untrusted character share code.
///
/// Pack-dependent mechanics must still be resolved by the caller.
///
/// # Errors
///
/// Returns an error for unsupported versions, oversized or malformed data, or
/// an invalid canonical character record.
pub fn decode(code: &str) -> Result<Character, String> {
    let code = code.trim();
    if code.len() > MAX_CODE_BYTES {
        return Err(format!(
            "share code exceeds the {} KiB input limit",
            (MAX_CODE_BYTES - PREFIX.len()) / 1024
        ));
    }
    let payload = code.strip_prefix(PREFIX).ok_or_else(|| {
        if code.starts_with("cw") {
            "unsupported character share-code version".to_owned()
        } else {
            "character share code must start with cw1:".to_owned()
        }
    })?;
    if payload.is_empty() {
        return Err("character share code has an empty payload".to_owned());
    }
    let source = decode_base64url(payload)?;
    if source.len() > MAX_JSON_BYTES {
        return Err(format!(
            "decoded character exceeds the {} KiB share limit",
            MAX_JSON_BYTES / 1024
        ));
    }
    let source = std::str::from_utf8(&source)
        .map_err(|_| "character share payload is not UTF-8 JSON".to_owned())?;
    Character::from_json(source).map_err(|error| format!("invalid shared character: {error}"))
}

fn decode_base64url(payload: &str) -> Result<Vec<u8>, String> {
    if payload.len() % 4 == 1 {
        return Err("character share payload has an invalid length".to_owned());
    }
    let values = payload
        .bytes()
        .map(|byte| {
            ALPHABET
                .iter()
                .position(|value| *value == byte)
                .and_then(|value| u8::try_from(value).ok())
                .ok_or_else(|| "character share payload is not valid base64url".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut output = Vec::with_capacity(payload.len() / 4 * 3 + 2);
    for chunk in values.chunks(4) {
        output.push((chunk[0] << 2) | (chunk[1] >> 4));
        if chunk.len() > 2 {
            output.push((chunk[1] << 4) | (chunk[2] >> 2));
        }
        if chunk.len() > 3 {
            output.push((chunk[2] << 6) | chunk[3]);
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{MAX_CODE_BYTES, decode, encode};
    use crate::character_wizard_domain::Character;

    #[test]
    fn share_code_round_trips_the_canonical_character() {
        let character = Character::from_json(include_str!("../fixtures/complete-character.json"))
            .expect("character fixture");
        let code = encode(&character).expect("encode character");
        assert!(code.starts_with("cw1:"));
        assert!(!code.contains('='));
        assert_eq!(decode(&code), Ok(character));
    }

    #[test]
    fn rejects_untrusted_versions_alphabets_lengths_and_sizes() {
        assert_eq!(
            decode("cw2:AAAA").expect_err("unsupported version"),
            "unsupported character share-code version"
        );
        assert_eq!(
            decode("cw1:AA=A").expect_err("padding is forbidden"),
            "character share payload is not valid base64url"
        );
        assert_eq!(
            decode("cw1:A").expect_err("invalid payload length"),
            "character share payload has an invalid length"
        );
        let oversized = format!("cw1:{}", "A".repeat(MAX_CODE_BYTES));
        assert!(
            decode(&oversized)
                .expect_err("oversized")
                .contains("input limit")
        );
    }
}
