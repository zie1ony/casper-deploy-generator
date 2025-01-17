use casper_types::{
    bytesrepr::ToBytes,
    system::auction::{DelegatorKind, Reservation},
    Digest, RuntimeArgs, TransactionArgs,
};

use crate::{
    ledger::Element,
    parser::runtime_args::{identity, parse_amount, parse_optional_arg},
};

use super::{parse_bytesrepr_args, v1_type, TransactionV1Meta};

fn parse_auction_v1<'a, F>(v1: &'a TransactionV1Meta, args_parser: F) -> Vec<Element>
where
    F: Fn(&'a RuntimeArgs) -> Vec<Element>,
{
    let mut elements = vec![];
    elements.extend(v1_type(v1).into_iter().map(|mut e| {
        e.as_expert();
        e
    }));
    match &v1.args {
        TransactionArgs::Named(args) => elements.extend(args_parser(args)),
        TransactionArgs::Bytesrepr(bytes) => elements.extend(parse_bytesrepr_args(bytes.clone())),
    }
    elements
}

pub(crate) fn parse_delegation(v1: &TransactionV1Meta) -> Vec<Element> {
    let arg_parser = |args| {
        let mut elements = vec![];
        // Public key of the account we're delegating from.
        elements.extend(parse_delegator(args));
        // Public key of the validator we're delegating to.
        elements.extend(parse_validator(args));
        // Amount we're delegating.
        elements.extend(parse_amount(args));
        elements
    };
    parse_auction_v1(v1, arg_parser)
}

pub(crate) fn parse_undelegation(item: &TransactionV1Meta) -> Vec<Element> {
    let arg_parser = |args| {
        let mut elements = vec![];
        // Public key of the account we're delegating from.
        elements.extend(parse_delegator(args));
        // Public key of the validator we're delegating to.
        elements.extend(parse_validator(args));
        // Amount we're delegating.
        elements.extend(parse_amount(args));
        elements
    };
    parse_auction_v1(item, arg_parser)
}

pub(crate) fn parse_redelegation(item: &TransactionV1Meta) -> Vec<Element> {
    let arg_parser = |args| {
        let mut elements = vec![];
        // Public key of the account we're delegating from.
        elements.extend(parse_delegator(args));
        // Public key of the current validator we have been redelagating to so far.
        elements.extend(parse_old_validator(args));
        // New validator we're redelegating to.
        elements.extend(parse_new_validator(args));
        // Amount we're delegating.
        elements.extend(parse_amount(args));
        elements
    };
    parse_auction_v1(item, arg_parser)
}

pub(crate) fn parse_add_bid(item: &TransactionV1Meta) -> Vec<Element> {
    let arg_parser = |args| {
        let mut elements = vec![];
        elements.extend(parse_public_key(args));
        elements.extend(parse_delegation_rate(args));
        elements.extend(parse_amount(args));
        elements.extend(parse_min_delegation_amount(args));
        elements.extend(parse_max_delegation_amount(args));
        elements.extend(parse_reserved_slots(args));
        elements
    };
    parse_auction_v1(item, arg_parser)
}

pub(crate) fn parse_activate_bid(item: &TransactionV1Meta) -> Vec<Element> {
    let arg_parser = |args| {
        let mut elements = vec![];
        elements.extend(parse_validator_public_key(args));
        elements
    };
    parse_auction_v1(item, arg_parser)
}

pub(crate) fn parse_change_bid_pk(item: &TransactionV1Meta) -> Vec<Element> {
    let arg_parser = |args| {
        let mut elements = vec![];
        elements.extend(parse_public_key(args));
        elements.extend(parse_new_public_key(args));
        elements
    };
    parse_auction_v1(item, arg_parser)
}

pub(crate) fn parse_add_reservations(item: &TransactionV1Meta) -> Vec<Element> {
    let arg_parser = |args| {
        let mut elements = vec![];
        elements.extend(parse_reservations(args));
        elements
    };
    parse_auction_v1(item, arg_parser)
}

pub(crate) fn parse_cancel_reservations(item: &TransactionV1Meta) -> Vec<Element> {
    let arg_parser = |args| {
        let mut elements = vec![];
        elements.extend(parse_validator(args));
        elements.extend(parse_delegator_kinds(args));
        elements
    };
    parse_auction_v1(item, arg_parser)
}

const DELEGATOR_ARG_KEY: &str = "delegator";
const VALIDATOR_ARG_KEY: &str = "validator";
const NEW_VALIDATOR_ARG_KEY: &str = "new_validator";
const PUBLIC_KEY_ARG_KEY: &str = "public_key";
const NEW_PUBLIC_KEY_ARG_KEY: &str = "new_public_key";
const DELEGATION_RATE_KEY_ARG_KEY: &str = "delegation_rate";
const MIN_DELEGATION_AMOUNT_KEY_ARG_KEY: &str = "minimum_delegation_amount";
const MAX_DELEGATION_AMOUNT_KEY_ARG_KEY: &str = "maximum_delegation_amount";
const RESERVED_SLOTS_ARG_KEY: &str = "reserved_slots";
const VALIDATOR_PK_ARG_KEY: &str = "validator_public_key";
const RESERVATIONS_ARG_KEY: &str = "reservations";
const DELEGATORS_ARG_KEY: &str = "delegators";

fn parse_delegator(args: &RuntimeArgs) -> Option<Element> {
    parse_optional_arg(args, DELEGATOR_ARG_KEY, "delegator", false, identity)
}

fn parse_validator(args: &RuntimeArgs) -> Option<Element> {
    parse_optional_arg(args, VALIDATOR_ARG_KEY, "validator", false, identity)
}

fn parse_old_validator(args: &RuntimeArgs) -> Option<Element> {
    parse_optional_arg(args, VALIDATOR_ARG_KEY, "old", false, identity)
}

fn parse_new_validator(args: &RuntimeArgs) -> Option<Element> {
    parse_optional_arg(args, NEW_VALIDATOR_ARG_KEY, "new", false, identity)
}

fn parse_public_key(args: &RuntimeArgs) -> Option<Element> {
    parse_optional_arg(args, PUBLIC_KEY_ARG_KEY, "pk", false, identity)
}

fn parse_new_public_key(args: &RuntimeArgs) -> Option<Element> {
    parse_optional_arg(args, NEW_PUBLIC_KEY_ARG_KEY, "new pk", false, identity)
}

fn parse_delegation_rate(args: &RuntimeArgs) -> Option<Element> {
    parse_optional_arg(
        args,
        DELEGATION_RATE_KEY_ARG_KEY,
        "deleg. rate",
        false,
        identity,
    )
}

fn parse_min_delegation_amount(args: &RuntimeArgs) -> Option<Element> {
    parse_optional_arg(
        args,
        MIN_DELEGATION_AMOUNT_KEY_ARG_KEY,
        "min. amount",
        false,
        identity,
    )
}

fn parse_max_delegation_amount(args: &RuntimeArgs) -> Option<Element> {
    parse_optional_arg(
        args,
        MAX_DELEGATION_AMOUNT_KEY_ARG_KEY,
        "max. amount",
        false,
        identity,
    )
}

fn parse_reserved_slots(args: &RuntimeArgs) -> Option<Element> {
    parse_optional_arg(args, RESERVED_SLOTS_ARG_KEY, "rsrvd slots", false, identity)
}

fn parse_validator_public_key(args: &RuntimeArgs) -> Option<Element> {
    parse_optional_arg(args, VALIDATOR_PK_ARG_KEY, "validtr pk", false, identity)
}

fn parse_reservations(args: &RuntimeArgs) -> Vec<Element> {
    let mut elements = vec![];
    if let Some(key) = args.get(RESERVATIONS_ARG_KEY) {
        let resv_array: Vec<Reservation> = key.to_t().unwrap();
        let length = resv_array.len();
        let hash = Digest::hash(resv_array.to_bytes().unwrap());
        elements.push(Element::regular("rsrv len", length.to_string()));
        elements.push(Element::regular("rsrv hash", base16::encode_lower(&hash)));
    }
    elements
}

fn parse_delegator_kinds(args: &RuntimeArgs) -> Vec<Element> {
    let mut elements = vec![];
    if let Some(key) = args.get(DELEGATORS_ARG_KEY) {
        let delegators_array: Vec<DelegatorKind> = key.to_t().unwrap();
        let length = delegators_array.len();
        let hash = Digest::hash(delegators_array.to_bytes().unwrap());
        elements.push(Element::regular("dlgtrs len", length.to_string()));
        elements.push(Element::regular("dlgtrs hash", base16::encode_lower(&hash)));
    }
    elements
}
