use crate::ledger::{Element, TxnPhase};
use crate::utils::cl_value_to_string;
use casper_types::bytesrepr::{Bytes, ToBytes};
use casper_types::system::mint::{self, ARG_ID, ARG_SOURCE, ARG_TARGET, ARG_TO};
use casper_types::{Digest, RuntimeArgs, U512};
use thousands::Separable;

pub(crate) fn parse_runtime_args_v1(ra: &RuntimeArgs) -> Vec<Element> {
    let mut elements: Vec<Element> = vec![];
    if !ra.is_empty() {
        let args_digest = Digest::hash(ToBytes::to_bytes(ra).expect("ToBytes to work."));
        let args_hash = base16::encode_lower(&args_digest);
        elements.push(Element::regular("args hash", args_hash));
    }
    elements
}

pub(crate) fn parse_bytesrepr_args(bytes: Bytes) -> Vec<Element> {
    let mut elements: Vec<Element> = vec![];
    if !bytes.is_empty() {
        let args_digest = Digest::hash(bytes);
        let args_hash = base16::encode_lower(&args_digest);
        elements.push(Element::regular("args hash", args_hash));
    }
    elements
}

pub(crate) fn parse_runtime_args(phase: &TxnPhase, ra: &RuntimeArgs) -> Vec<Element> {
    let mut elements: Vec<Element> = vec![];
    if !ra.is_empty() {
        let args_digest = Digest::hash(ToBytes::to_bytes(ra).expect("ToBytes to work."));
        let args_hash = base16::encode_lower(&args_digest);
        elements.push(Element::regular(
            "args hash",
            format!("{}-{}", phase.to_string().to_lowercase(), args_hash),
        ));
    }

    elements
}

pub(crate) fn parse_optional_arg<F: Fn(String) -> String>(
    args: &RuntimeArgs,
    key: &str,
    label: &str,
    expert: bool,
    f: F,
) -> Option<Element> {
    match args.get(key) {
        Some(cl_value) => {
            let value = f(cl_value_to_string(cl_value));
            let element = if expert {
                Element::expert(label, value)
            } else {
                Element::regular(label, value)
            };
            Some(element)
        }
        None => None,
    }
}

/// Required fields for transfer are:
/// * source
/// * target
/// * amount
/// Optional fields:
/// * to (Option<AccountHash>)
/// * ID
pub(crate) fn parse_transfer_args(args: &RuntimeArgs) -> Vec<Element> {
    let mut elements: Vec<Element> = parse_optional_arg(args, ARG_TO, "recipient", false, identity)
        .into_iter()
        .collect();
    elements.extend(parse_optional_arg(args, ARG_SOURCE, "from", true, identity));
    elements.extend(parse_optional_arg(
        args, ARG_TARGET, "target", false, identity,
    ));
    elements.extend(parse_amount(args));
    elements.extend(parse_optional_arg(args, ARG_ID, "ID", true, identity));
    elements
}

pub(crate) fn parse_fee(args: &RuntimeArgs) -> Option<Element> {
    parse_motes(args, "fee")
}

pub(crate) fn parse_amount(args: &RuntimeArgs) -> Option<Element> {
    parse_motes(args, "amount")
}

fn parse_motes(args: &RuntimeArgs, ledger_label: &str) -> Option<Element> {
    let f = |amount_str: String| {
        let motes_amount = U512::from_dec_str(&amount_str).unwrap();
        format_amount(motes_amount)
    };
    parse_optional_arg(args, mint::ARG_AMOUNT, ledger_label, false, f)
}

pub(crate) fn format_amount(motes: U512) -> String {
    format!("{} motes", motes.separate_with_spaces())
}

pub(crate) fn identity<T>(el: T) -> T {
    el
}
