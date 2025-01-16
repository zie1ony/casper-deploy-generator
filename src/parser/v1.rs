pub(crate) mod auction;

use std::collections::BTreeMap;

use crate::{
    ledger::Element,
    parser::{
        runtime_args::{parse_amount, parse_fee, parse_transfer_args},
        utils::timestamp_to_seconds_res,
    },
    utils::{parse_account_hash, parse_public_key},
};

use auction::{parse_activate_bid, parse_add_bid, parse_delegation, parse_redelegation, parse_undelegation};
use casper_types::{
    bytesrepr::Bytes,
    system::mint::{self, ARG_ID, ARG_SOURCE, ARG_TARGET, ARG_TO},
    CLValue, Digest, InitiatorAddr, PricingMode, RuntimeArgs, TransactionArgs,
    TransactionEntryPoint, TransactionInvocationTarget, TransactionScheduling, TransactionTarget,
    TransactionV1, TransactionV1Payload,
};
use hex::ToHex;

use super::runtime_args::{parse_bytesrepr_args, parse_runtime_args_v1};

pub(crate) const ARGS_MAP_KEY: u16 = 0;
pub(crate) const TARGET_MAP_KEY: u16 = 1;
pub(crate) const ENTRY_POINT_MAP_KEY: u16 = 2;
pub(crate) const SCHEDULING_MAP_KEY: u16 = 3;

#[derive(Clone)]
pub(crate) struct TransactionV1Meta {
    pub args: TransactionArgs,
    pub target: TransactionTarget,
    pub entry_point: TransactionEntryPoint,
    pub scheduling: TransactionScheduling,
}

impl TransactionV1Meta {
    pub fn new(
        args: TransactionArgs,
        target: TransactionTarget,
        entry_point: TransactionEntryPoint,
        scheduling: TransactionScheduling,
    ) -> Self {
        Self {
            args,
            target,
            entry_point,
            scheduling,
        }
    }

    pub fn deserialize_from(v1: &TransactionV1) -> TransactionV1Meta {
        let args = v1.deserialize_field(ARGS_MAP_KEY).unwrap();
        let target = v1.deserialize_field(TARGET_MAP_KEY).unwrap();
        let entry_point = v1.deserialize_field(ENTRY_POINT_MAP_KEY).unwrap();
        let scheduling = v1.deserialize_field(SCHEDULING_MAP_KEY).unwrap();

        Self {
            args,
            target,
            entry_point,
            scheduling,
        }
    }
}

pub(crate) fn parse_v1_payload(payload: &TransactionV1Payload) -> Vec<Element> {
    let mut elements = vec![];
    elements.push(Element::regular(
        "chain ID",
        payload.chain_name().to_string(),
    ));
    let initiator = match payload.initiator_addr() {
        InitiatorAddr::PublicKey(public_key) => parse_public_key(public_key),
        InitiatorAddr::AccountHash(account_hash) => parse_account_hash(account_hash),
    };
    elements.push(Element::regular("account", initiator));
    elements.push(Element::expert(
        "timestamp",
        timestamp_to_seconds_res(payload.timestamp()),
    ));
    elements.push(Element::expert("ttl", format!("{}", payload.ttl())));

    match payload.pricing_mode() {
        PricingMode::PaymentLimited { payment_amount, gas_price_tolerance, .. } => {
            let payment_type_label = Element::expert("payment", payment_amount.to_string());
            let max_gas_label = Element::expert("max gas", gas_price_tolerance.to_string());
            elements.push(payment_type_label);
            elements.push(max_gas_label);
        },
        PricingMode::Fixed { gas_price_tolerance, .. } => {
            let payment_type_label = Element::expert("payment", "fixed".into());
            let max_gas_label = Element::expert("max gas", gas_price_tolerance.to_string());
            elements.push(payment_type_label);
            elements.push(max_gas_label);
        },
        PricingMode::Prepaid { receipt } => {
            let payment_type_label = Element::expert("payment", "prepaid".into());
            let receipt_label = Element::expert("receipt", receipt.encode_hex::<String>());
            elements.push(payment_type_label);
            elements.push(receipt_label);
        },
    }

    elements
}

pub(crate) fn parse_v1_meta(v1: &TransactionV1) -> Vec<Element> {
    let meta = TransactionV1Meta::deserialize_from(v1);

    match &meta.target {
        TransactionTarget::Native => match meta.entry_point {
            TransactionEntryPoint::Transfer => {
                let args = meta.args.as_named().unwrap();
                let mut elements: Vec<Element> = v1_type(&meta);
                elements.extend(parse_transfer_args(args));
                let args_sans_transfer = remove_transfer_args(args.clone());
                if !args_sans_transfer.is_empty() {
                    elements.extend(parse_runtime_args_v1(args));
                }
                elements
            }
            TransactionEntryPoint::Delegate => parse_delegation(&meta),
            TransactionEntryPoint::Undelegate => parse_undelegation(&meta),
            TransactionEntryPoint::Redelegate => parse_redelegation(&meta),
            TransactionEntryPoint::AddBid => parse_add_bid(&meta),
            TransactionEntryPoint::ActivateBid => parse_activate_bid(&meta),
            _ => panic!(
                "Generator doesn't support the native entrypoint '{}'",
                meta.entry_point
            ),
        },
        TransactionTarget::Stored { .. } => {
            let mut elements: Vec<Element> = v1_type(&meta);
            let entry_point_str = match meta.entry_point {
                TransactionEntryPoint::Call | TransactionEntryPoint::Custom(_) => {
                    meta.entry_point.custom_entry_point().unwrap()
                }
                _ => meta.entry_point.to_string()
            };
            elements.push(entrypoint(&entry_point_str));
            match meta.args {
                TransactionArgs::Named(args) => {
                    elements.extend(parse_amount(&args));
                    elements.extend(parse_runtime_args_v1(&args));
                },
                TransactionArgs::Bytesrepr(bytes) => {
                    elements.extend(parse_bytesrepr_args(bytes));
                },
            }
            elements
        }
        TransactionTarget::Session { module_bytes, .. } => {
            let mut elements: Vec<Element> = v1_type(&meta);
            match meta.args {
                TransactionArgs::Named(args) => {
                    if is_system_payment(module_bytes) {
                        elements.extend(parse_fee(&args));
                        let args_sans_amount = remove_amount_arg(args.clone());
                        if !args_sans_amount.is_empty() {
                            elements.extend(parse_runtime_args_v1(&args));
                        }
                    } else {
                        elements.extend(parse_amount(&args));
                        elements.extend(parse_runtime_args_v1(&args));
                    }
                },
                TransactionArgs::Bytesrepr(bytes) => {
                    elements.extend(parse_bytesrepr_args(bytes));
                },
            }
            elements
        }
    }
}

/// Returns the main elements describing the deploy:
/// Is it a raw contract bytes, call by name, by hash, versioned, etc.?
pub(crate) fn v1_type(item: &TransactionV1Meta) -> Vec<Element> {
    match &item.target {
        TransactionTarget::Native => {
            vec![]
        }
        TransactionTarget::Stored { id, .. } => match id {
            TransactionInvocationTarget::ByHash(hash) => {
                vec![
                    Element::regular("execution", "by-hash".to_string()),
                    Element::regular("address", hash.iter().map(|x| x.to_string()).collect()),
                ]
            }
            TransactionInvocationTarget::ByName(name) => {
                vec![
                    Element::regular("execution", "by-name".to_string()),
                    Element::regular("name", name.clone()),
                ]
            }
            TransactionInvocationTarget::ByPackageHash { addr, version } => {
                vec![
                    Element::regular("execution", "by-hash-versioned".to_string()),
                    Element::regular("address", addr.iter().map(|x| x.to_string()).collect()),
                    parse_version(version),
                ]
            }
            TransactionInvocationTarget::ByPackageName { name, version } => {
                vec![
                    Element::regular("execution", "by-name-versioned".to_string()),
                    Element::regular("name", name.to_string()),
                    parse_version(version),
                ]
            }
        },
        TransactionTarget::Session { module_bytes, .. } => {
            if is_system_payment(module_bytes) {
                vec![]
            } else {
                let contract_hash = format!("{:?}", Digest::hash(module_bytes.as_slice()));
                vec![
                    Element::regular("execution", "contract".to_string()),
                    Element::regular("Cntrct hash", contract_hash),
                ]
            }
        }
    }
}

fn parse_version(version: &Option<u32>) -> Element {
    let version = match version {
        None => "latest".to_string(),
        Some(version) => format!("{}", version),
    };
    Element::expert("version", version)
}

// Payment is a system type of payment when the `module_bytes` are empty.
fn is_system_payment(module_bytes: &Bytes) -> bool {
    module_bytes.inner_bytes().is_empty()
}

fn remove_amount_arg(args: RuntimeArgs) -> RuntimeArgs {
    let mut tree: BTreeMap<String, CLValue> = args.into();
    tree.remove(mint::ARG_AMOUNT);
    tree.into()
}

/// Removes all arguments that are used in the Transfer.
fn remove_transfer_args(args: RuntimeArgs) -> RuntimeArgs {
    let mut tree: BTreeMap<String, CLValue> = args.into();
    tree.remove(ARG_TO);
    tree.remove(ARG_SOURCE);
    tree.remove(ARG_TARGET);
    tree.remove(mint::ARG_AMOUNT);
    tree.remove(ARG_ID);
    tree.into()
}

pub(crate) fn parse_v1_approvals(d: &TransactionV1) -> Vec<Element> {
    let approvals_count = d.approvals().len();
    vec![Element::expert(
        "Approvals #",
        format!("{}", approvals_count),
    )]
}

fn entrypoint(entry_point: &str) -> Element {
    Element::expert("entry-point", entry_point.to_string())
}
