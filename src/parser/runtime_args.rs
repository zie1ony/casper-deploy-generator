use crate::ledger::{Element, TxnPhase};
use crate::utils::cl_value_to_string;
use casper_types::bytesrepr::ToBytes;
use casper_types::system::mint::{ARG_ID, ARG_SOURCE, ARG_TARGET, ARG_TO};
use casper_types::RuntimeArgs;

use super::deploy::{identity, parse_amount};

/// Parses all contract arguments into a form:
/// arg-n-name: <name>
/// arg-n-val: <val>
/// where n is the ordinal number of the argument.
pub(crate) fn parse_runtime_args(phase: &TxnPhase, ra: &RuntimeArgs) -> Vec<Element> {
    let mut elements: Vec<Element> = vec![];
    if !ra.is_empty() {
        let args_digest =
            casper_hashing::Digest::hash(ToBytes::to_bytes(ra).expect("ToBytes to work."));
        let args_hash = base16::encode_lower(&args_digest);
        elements.push(Element::regular(
            "args hash",
            format!("{}-{}", phase.to_string().to_lowercase(), args_hash),
        ));
    }

    // NOTE: The code that follows would iterate over all args and parse them
    // for Ledger presentation in a following format:
    // Arg-n-name: <name>
    // Arg-n-val: <value>
    // But this could lead to very long confirmation screens in Ledger,
    // so we opted for shorter form above: display just hash of the runtime args.
    // If we ever decide to bring back the more elaborate version, this code would do it.
    // let named_args: BTreeMap<String, CLValue> = ra.clone().into();
    // for (idx, (name, value)) in named_args.iter().enumerate() {
    //     let name_label = format!("arg-{}-name", idx);
    //     elements.push(Element::expert(&name_label, name.to_string()));
    //     let value_label = format!("arg-{}-val", idx);
    //     let value_str = cl_value_to_string(value);
    //     elements.push(Element::expert(&value_label, value_str));
    // }
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
/// * target
/// * amount
/// * ID
/// Optional fields:
/// * source
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
