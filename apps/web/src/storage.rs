// SPDX-License-Identifier: MPL-2.0

//! Browser persistence boundary.

#[cfg(any(test, target_arch = "wasm32"))]
const ENCODING_PREFIX: &str = "hex-v1:";

#[cfg(any(test, target_arch = "wasm32"))]
fn encode_bytes(data: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut encoded = String::with_capacity(ENCODING_PREFIX.len() + data.len() * 2);
    encoded.push_str(ENCODING_PREFIX);
    for byte in data {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(any(test, target_arch = "wasm32"))]
fn decode_bytes(encoded: &str) -> Result<Vec<u8>, &'static str> {
    let payload = encoded
        .strip_prefix(ENCODING_PREFIX)
        .ok_or("encoding prefix is missing")?;
    if payload.len() % 2 != 0 {
        return Err("hex payload has an odd length");
    }

    payload
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = decode_nibble(pair[0]).ok_or("hex payload contains an invalid digit")?;
            let low = decode_nibble(pair[1]).ok_or("hex payload contains an invalid digit")?;
            Ok((high << 4) | low)
        })
        .collect()
}

#[cfg(any(test, target_arch = "wasm32"))]
const fn decode_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use raster_storage::{ByteStorage, StorageError, StorageKey};
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{DomException, Storage};

    use super::{decode_bytes, encode_bytes};

    const STORAGE_NAMESPACE: &str = "raster-nights.persistence.v1";

    /// Local browser persistence using same-origin `localStorage`.
    #[derive(Clone)]
    pub struct BrowserByteStorage {
        storage: Storage,
    }

    impl std::fmt::Debug for BrowserByteStorage {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter
                .debug_struct("BrowserByteStorage")
                .finish_non_exhaustive()
        }
    }

    impl BrowserByteStorage {
        pub fn new() -> Result<Self, StorageError> {
            let window = web_sys::window()
                .ok_or_else(|| StorageError::new("browser window is unavailable"))?;
            let storage = window
                .local_storage()
                .map_err(|error| storage_error("open local storage", &error))?
                .ok_or_else(|| StorageError::new("browser local storage is unavailable"))?;
            Ok(Self { storage })
        }

        fn next_corrupt_key(&self, key: StorageKey) -> Result<String, StorageError> {
            let entries = self
                .storage
                .length()
                .map_err(|error| storage_error("inspect local storage", &error))?;

            // At most `entries` keys are occupied, so one of these candidates
            // must be available even when the namespace contains sparse backups.
            for sequence in 1..=entries.saturating_add(1) {
                let candidate = format!(
                    "{STORAGE_NAMESPACE}.corrupt.{}.{sequence:08}",
                    key_name(key)
                );
                let occupied = self
                    .storage
                    .get_item(&candidate)
                    .map_err(|error| storage_error("inspect corrupt-data backup", &error))?
                    .is_some();
                if !occupied {
                    return Ok(candidate);
                }
            }

            Err(StorageError::new(
                "could not allocate a corrupt-data backup key",
            ))
        }
    }

    impl ByteStorage for BrowserByteStorage {
        fn read(&self, key: StorageKey) -> Result<Option<Vec<u8>>, StorageError> {
            let Some(encoded) = self
                .storage
                .get_item(&storage_key(key))
                .map_err(|error| storage_error("read local storage", &error))?
            else {
                return Ok(None);
            };

            // Values not written by this adapter are deliberately passed to the
            // repository as raw bytes. Its codec will reject and preserve them.
            Ok(Some(
                decode_bytes(&encoded).unwrap_or_else(|_| encoded.into_bytes()),
            ))
        }

        fn write(&mut self, key: StorageKey, data: &[u8]) -> Result<(), StorageError> {
            self.storage
                .set_item(&storage_key(key), &encode_bytes(data))
                .map_err(|error| storage_error("write local storage", &error))
        }

        fn remove(&mut self, key: StorageKey) -> Result<(), StorageError> {
            self.storage
                .remove_item(&storage_key(key))
                .map_err(|error| storage_error("remove local storage", &error))
        }

        fn preserve_corrupt(&mut self, key: StorageKey, data: &[u8]) -> Result<(), StorageError> {
            let backup_key = self.next_corrupt_key(key)?;
            self.storage
                .set_item(&backup_key, &encode_bytes(data))
                .map_err(|error| storage_error("preserve corrupt local data", &error))?;

            self.storage
                .remove_item(&storage_key(key))
                .map_err(|error| {
                    storage_error("remove corrupt local data after preserving it", &error)
                })
        }
    }

    fn storage_key(key: StorageKey) -> String {
        format!("{STORAGE_NAMESPACE}.{}", key_name(key))
    }

    const fn key_name(key: StorageKey) -> &'static str {
        match key {
            StorageKey::Settings => "settings",
            StorageKey::Scores => "scores",
            StorageKey::SystemState => "system-state",
        }
    }

    fn storage_error(operation: &str, error: &JsValue) -> StorageError {
        let detail = error.dyn_ref::<DomException>().map_or_else(
            || "browser rejected the operation".to_owned(),
            |exception| match exception.name().as_str() {
                "QuotaExceededError" => "local storage quota is exhausted".to_owned(),
                "SecurityError" => {
                    "local storage access is blocked by browser security policy".to_owned()
                }
                name => format!("{name}: {}", exception.message()),
            },
        );
        StorageError::new(format!("{operation}: {detail}"))
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::BrowserByteStorage;

#[cfg(test)]
mod tests {
    use super::{decode_bytes, encode_bytes};

    #[test]
    fn byte_encoding_is_deterministic_and_reversible() {
        let data = [0x00, 0x01, 0x7f, 0x80, 0xfe, 0xff];
        let encoded = encode_bytes(&data);

        assert_eq!(encoded, "hex-v1:00017f80feff");
        assert_eq!(decode_bytes(&encoded), Ok(data.to_vec()));
    }

    #[test]
    fn byte_encoding_handles_empty_data() {
        assert_eq!(encode_bytes(&[]), "hex-v1:");
        assert_eq!(decode_bytes("hex-v1:"), Ok(Vec::new()));
    }

    #[test]
    fn byte_decoding_rejects_malformed_values() {
        assert_eq!(decode_bytes("00"), Err("encoding prefix is missing"));
        assert_eq!(
            decode_bytes("hex-v1:0"),
            Err("hex payload has an odd length")
        );
        assert_eq!(
            decode_bytes("hex-v1:0x"),
            Err("hex payload contains an invalid digit")
        );
    }
}
