use casper_types::{system::mint, ExecutableDeployItem, RuntimeArgs, TransactionArgs};

use crate::{ledger::{Element, TxnPhase}, parser::runtime_args::{identity, parse_amount, parse_optional_arg}};

use super::{v1_type, TransactionV1Meta};

fn parse_auction_v1<'a, F>(
    method: &str,
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
                // For now, we choose to not display deploy's details for delegation.
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
    parse_auction_v1("delegate", v1, arg_parser)
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
    parse_auction_v1("undelegate", item, arg_parser)
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
    parse_auction_v1("redelegate", item, arg_parser)
}

/// Returns `true` when the deploy's entry point is *literally* _delegate_
pub(crate) fn is_delegate(item: &ExecutableDeployItem) -> bool {
    (is_entrypoint(item, DELEGATE_ENTRYPOINT) || has_delegate_auction_arg(item))
        && has_delegate_args(item)
}

/// Returns `true` when the deploy's entry point is *literally* _undelegate_
pub(crate) fn is_undelegate(item: &ExecutableDeployItem) -> bool {
    (is_entrypoint(item, UNDELEGATE_ENTRYPOINT) || has_undelegate_auction_arg(item))
        && has_undelegate_arg(item)
}

/// Returns `true` when the deploy's entry point is *literally* _undelegate_
pub(crate) fn is_redelegate(item: &ExecutableDeployItem) -> bool {
    (is_entrypoint(item, REDELEGATE_ENTRYPOINT) || has_redelegate_auction_arg(item))
        && has_redelegate_arg(item)
}

fn get_auction_arg(item: &ExecutableDeployItem) -> Option<String> {
    match item {
        // ModuleBytes variant does not have an entry point, it defaults to `call()`,
        // so we expect a special named argument called `auction` when detecting auction contract calls.
        ExecutableDeployItem::ModuleBytes { args, .. } => args.get("auction").map(|cl_value| {
            cl_value
                .clone()
                .into_t::<String>()
                .expect("argument should be string")
        }),
        _ => None,
    }
}

const DELEGATE_ENTRYPOINT: &str = "delegate";
const UNDELEGATE_ENTRYPOINT: &str = "undelegate";
const REDELEGATE_ENTRYPOINT: &str = "redelegate";
const DELEGATOR_ARG_KEY: &str = "delegator";
const VALIDATOR_ARG_KEY: &str = "validator";
const NEW_VALIDATOR_ARG_KEY: &str = "new_validator";

fn has_delegate_auction_arg(item: &ExecutableDeployItem) -> bool {
    get_auction_arg(item)
        .filter(|arg_value| arg_value.to_lowercase() == DELEGATE_ENTRYPOINT)
        .is_some()
}

fn has_undelegate_auction_arg(item: &ExecutableDeployItem) -> bool {
    get_auction_arg(item)
        .filter(|arg_value| arg_value.to_lowercase() == UNDELEGATE_ENTRYPOINT)
        .is_some()
}

fn has_redelegate_auction_arg(item: &ExecutableDeployItem) -> bool {
    get_auction_arg(item)
        .filter(|arg_value| arg_value.to_lowercase() == REDELEGATE_ENTRYPOINT)
        .is_some()
}

fn has_delegate_args(item: &ExecutableDeployItem) -> bool {
    item.args().get(DELEGATOR_ARG_KEY).is_some()
        && item.args().get(VALIDATOR_ARG_KEY).is_some()
        && item.args().get(mint::ARG_AMOUNT).is_some()
}

fn has_undelegate_arg(item: &ExecutableDeployItem) -> bool {
    item.args().get(DELEGATOR_ARG_KEY).is_some()
        && item.args().get(VALIDATOR_ARG_KEY).is_some()
        && item.args().get(mint::ARG_AMOUNT).is_some()
}

fn has_redelegate_arg(item: &ExecutableDeployItem) -> bool {
    item.args().get(DELEGATOR_ARG_KEY).is_some()
        && item.args().get(VALIDATOR_ARG_KEY).is_some()
        && item.args().get(NEW_VALIDATOR_ARG_KEY).is_some()
        && item.args().get(mint::ARG_AMOUNT).is_some()
}

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

fn is_entrypoint(item: &ExecutableDeployItem, expected: &str) -> bool {
    match item {
        ExecutableDeployItem::ModuleBytes { .. } | ExecutableDeployItem::Transfer { .. } => false,
        ExecutableDeployItem::StoredContractByHash { entry_point, .. }
        | ExecutableDeployItem::StoredContractByName { entry_point, .. }
        | ExecutableDeployItem::StoredVersionedContractByHash { entry_point, .. }
        | ExecutableDeployItem::StoredVersionedContractByName { entry_point, .. } => {
            entry_point == expected
        }
    }
}
