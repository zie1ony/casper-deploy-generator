use casper_types::{RuntimeArgs, TransactionArgs};

use crate::{ledger::{Element, TxnPhase}, parser::runtime_args::{identity, parse_amount, parse_optional_arg}};

use super::{v1_type, TransactionV1Meta};

fn parse_auction_v1<'a, F>(
    v1: &'a TransactionV1Meta,
    args_parser: F,
) -> Vec<Element>
where
    F: Fn(&'a RuntimeArgs) -> Vec<Element>,
{
    let mut elements = vec![];
    elements.extend(
        v1_type(TxnPhase::Session, v1)
            .into_iter()
            .map(|mut e| {
                e.as_expert();
                e
            }),
    );
    match &v1.args {
        TransactionArgs::Named(args) => elements.extend(args_parser(args)),
        TransactionArgs::Bytesrepr(_) => panic!(),
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

const DELEGATOR_ARG_KEY: &str = "delegator";
const VALIDATOR_ARG_KEY: &str = "validator";
const NEW_VALIDATOR_ARG_KEY: &str = "new_validator";

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