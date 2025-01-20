use blake2::{
    digest::{Update, VariableOutput},
    Blake2bVar,
};
use casper_types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    BlockGlobalAddr, CLType, CLValue, Key, PublicKey, URef, ED25519_TAG, SECP256K1_TAG,
};
use itertools::Itertools;

use crate::checksummed_hex;

pub(crate) const DIGEST_LENGTH: usize = 32;

/// Compute the blake2b hash over some byte data
pub(crate) fn blake2b<T: AsRef<[u8]>>(data: T) -> [u8; DIGEST_LENGTH] {
    let mut result = [0; DIGEST_LENGTH];
    // NOTE: Assumed safe as `BLAKE2B_DIGEST_LENGTH` is a valid value for a hasher
    let mut hasher = Blake2bVar::new(DIGEST_LENGTH).expect("should create hasher");

    hasher.update(data.as_ref());

    // NOTE: This should never fail, because result is exactly DIGEST_LENGTH long
    hasher.finalize_variable(&mut result).ok();

    result
}

/// Turn JSON representation into a string.
fn serde_value_to_str(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(value) => format!("{}", value),
        serde_json::Value::Number(num) => format!("{}", num),
        serde_json::Value::String(string) => drop_key_type_prefix(string.clone()),
        serde_json::Value::Array(arr) => {
            format!("[{}]", arr.iter().map(serde_value_to_str).join(", "))
        }
        serde_json::Value::Object(map) => map.values().map(serde_value_to_str).join(":"),
    }
}

/// Drop type prefix (if we know how).
fn drop_key_type_prefix(cl_in: String) -> String {
    let parsed_key = Key::from_formatted_str(&cl_in);
    match parsed_key {
        Ok(key) => {
            let prefix = match key {
                Key::Account(_) => "account-hash-",
                Key::Hash(_) => "hash-",
                Key::Transfer(_) => "transfer-",
                Key::DeployInfo(_) => "deploy-",
                Key::EraInfo(_) => "era-",
                Key::Balance(_) => "balance-",
                Key::Bid(_) => "bid-",
                Key::Withdraw(_) => "withdraw-",
                Key::URef(_) => {
                    // format: uref-XXXX-YYY
                    return cl_in
                        .chars()
                        .skip("uref-".len())
                        .take_while(|c| *c != '-')
                        .collect();
                }
                Key::Dictionary(_) => "dictionary-",
                Key::SystemEntityRegistry => "system-entity-registry-",
                Key::Unbond(_) => "ubond-",
                Key::ChainspecRegistry => "chainspec-registry",
                Key::ChecksumRegistry => "checksum-registry",
                Key::EraSummary => "era-summary-",
                Key::BidAddr(_) => "bid-addr-",
                Key::SmartContract(_) => "package-",
                Key::BlockGlobal(addr) => match addr {
                    BlockGlobalAddr::BlockTime => "block-time-",
                    BlockGlobalAddr::MessageCount => "block-message-count-",
                },
                Key::BalanceHold(_) => "balance-hold-",
                Key::State(_) => "state-",
                Key::AddressableEntity(_)
                | Key::ByteCode(_)
                | Key::Message(_)
                | Key::NamedKey(_)
                | Key::EntryPoint(_) => "",
            };

            let stripped_prefix = cl_in.chars().skip(prefix.len()).collect();
            debug_assert_eq!(
                Key::from_formatted_str(&format!("{prefix}{stripped_prefix}")).unwrap(),
                key
            );
            stripped_prefix
        }
        Err(_) => {
            // No idea how to handle that. Return raw.
            cl_in
        }
    }
}

/// Extracts the `parsed` field from the `CLValue`
/// (which is a pair of type identifier and raw bytes).
/// It should be human-readable.
pub(crate) fn cl_value_to_string(cl_in: &CLValue) -> String {
    match cl_in.cl_type() {
        CLType::Key => {
            let account: Key = FromBytes::from_bytes(cl_in.inner_bytes())
                .expect("key account to be deserialized with FromBytes")
                .0;

            match account {
                Key::URef(uref) => checksummed_hex::encode(uref.addr()),
                Key::Hash(addr) => checksummed_hex::encode(addr),
                Key::Transfer(addr) => checksummed_hex::encode(addr.value()),
                Key::DeployInfo(deploy_hash) => checksummed_hex::encode(
                    deploy_hash.to_bytes().expect("DeployHash should serialize"),
                ),
                Key::Balance(uref_addr) => checksummed_hex::encode(uref_addr),
                Key::Dictionary(dict_addr) => checksummed_hex::encode(dict_addr),
                Key::Account(account_hash)
                | Key::Unbond(account_hash)
                | Key::Withdraw(account_hash)
                | Key::Bid(account_hash) => checksummed_hex::encode(account_hash),
                Key::EraInfo(_)
                | Key::SystemEntityRegistry
                | Key::ChainspecRegistry
                | Key::ChecksumRegistry
                | Key::EraSummary => parse_as_default_json(cl_in),
                Key::SmartContract(package) => checksummed_hex::encode(package),
                Key::BidAddr(bid_addr) => {
                    checksummed_hex::encode(bid_addr.to_bytes().expect("BidAddr should serialize"))
                }
                Key::AddressableEntity(entity_addr) => checksummed_hex::encode(
                    entity_addr.to_bytes().expect("Entity should serialize"),
                ),
                Key::ByteCode(byte_code_addr) => checksummed_hex::encode(
                    byte_code_addr
                        .to_bytes()
                        .expect("ByteCodeAddr should serialize"),
                ),
                Key::Message(message_addr) => checksummed_hex::encode(
                    message_addr.to_bytes().expect("Message should serialize"),
                ),
                Key::NamedKey(named_key_addr) => checksummed_hex::encode(
                    named_key_addr
                        .to_bytes()
                        .expect("NamedKeyAddr should serialize"),
                ),
                Key::BlockGlobal(block_global_addr) => checksummed_hex::encode(
                    block_global_addr
                        .to_bytes()
                        .expect("BlockGlobalAddr should serialize"),
                ),
                Key::BalanceHold(balance_hold_addr) => checksummed_hex::encode(
                    balance_hold_addr
                        .to_bytes()
                        .expect("BalanceHoldAddr should serialize"),
                ),
                Key::EntryPoint(entry_point_addr) => checksummed_hex::encode(
                    entry_point_addr
                        .to_bytes()
                        .expect("EntryPointAddr should serialize"),
                ),
                Key::State(entity_addr) => checksummed_hex::encode(
                    entity_addr.to_bytes().expect("EntityAddr should serialize"),
                ),
            }
        }
        CLType::URef => {
            let uref: URef = FromBytes::from_bytes(cl_in.inner_bytes())
                .expect("uref to be deserialized with FromBytes")
                .0;
            checksummed_hex::encode(uref.addr())
        }
        CLType::PublicKey => {
            let public_key: PublicKey = FromBytes::from_bytes(cl_in.inner_bytes())
                .expect("public key to be deserialized with FromBytes")
                .0;
            parse_public_key(&public_key)
        }
        CLType::ByteArray(length) => {
            let (bytes, _remainder) = cl_in.inner_bytes().split_at(*length as usize);

            checksummed_hex::encode(bytes)
        }
        _ => parse_as_default_json(cl_in),
    }
}

fn parse_as_default_json(input: &CLValue) -> String {
    match serde_json::to_value(input) {
        Ok(value) => {
            let parsed = value.get("parsed").unwrap();
            serde_value_to_str(parsed)
        }
        Err(err) => {
            eprintln!("error when parsing CLValue to CLValueJson#Object, {}", err);
            panic!("{:?}", err)
        }
    }
}

// `PublicKey`'s `String` representation includes a `PublicKey::<variant>` prefix.
// This method drops that prefix (and the closing ')') from the `String` representation for the Ledger.
pub(crate) fn parse_public_key(key: &PublicKey) -> String {
    let key_tag = match key {
        PublicKey::System => panic!("Did not expect system key"),
        PublicKey::Ed25519(_) => format!("0{}", ED25519_TAG),
        PublicKey::Secp256k1(_) => format!("0{}", SECP256K1_TAG),
        _ => panic!("Should not happen - all key variants are covered at the time of writing"),
    };

    let checksummed_key = checksummed_hex::encode(Into::<Vec<u8>>::into(key));
    format!("{}{}", key_tag, checksummed_key)
}

// `AccountHash`'s `String` representation includes an `account-hash-` prefix.
// This method drops that prefix from the `String` representation for the Ledger.
pub(crate) fn parse_account_hash(account_hash: &AccountHash) -> String {
    account_hash
        .to_formatted_string()
        .chars()
        .skip("account-hash-".len())
        .collect()
}
