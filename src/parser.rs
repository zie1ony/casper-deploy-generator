mod deploy;
mod runtime_args;
mod utils;
pub(crate) mod v1;

use casper_types::{Deploy, TransactionEntryPoint, TransactionV1};
use v1::{parse_v1_approvals, parse_v1_meta, parse_v1_payload, ENTRY_POINT_MAP_KEY};

use crate::{
    checksummed_hex,
    ledger::{Element, TxnPhase},
    message::CasperMessage,
    parser::deploy::{parse_approvals, parse_deploy_header, parse_phase},
};

pub(crate) fn parse_message(m: CasperMessage) -> Vec<Element> {
    vec![Element::regular("Msg hash", hex::encode(m.hashed()))]
}

pub(crate) fn parse_deploy(d: Deploy) -> Vec<Element> {
    let mut elements = vec![];
    elements.push(Element::regular(
        "Txn hash",
        checksummed_hex::encode(d.hash().inner()).to_string(),
    ));
    elements.push(deploy_type(&d));
    elements.extend(parse_deploy_header(d.header()));
    elements.extend(parse_phase(d.payment(), TxnPhase::Payment));
    elements.extend(parse_phase(d.session(), TxnPhase::Session));
    elements.extend(parse_approvals(&d));
    elements
}

pub(crate) fn parse_v1(v1: TransactionV1) -> Vec<Element> {
    let mut elements = vec![];
    elements.push(Element::regular(
        "Txn hash",
        checksummed_hex::encode(v1.hash().inner()).to_string(),
    ));
    elements.push(transaction_v1_type(&v1));
    elements.extend(parse_v1_payload(v1.payload()));
    elements.extend(parse_v1_meta(&v1));
    elements.extend(parse_v1_approvals(&v1));
    elements
}

fn deploy_type(d: &Deploy) -> Element {
    let dtype = if deploy::auction::is_delegate(d.session()) {
        "Delegate"
    } else if deploy::auction::is_undelegate(d.session()) {
        "Undelegate"
    } else if deploy::auction::is_redelegate(d.session()) {
        "Redelegate"
    } else if d.session().is_transfer() {
        "Token transfer"
    } else {
        "Contract execution"
    };
    Element::regular("Type", dtype.to_string())
}

fn transaction_v1_type(t: &TransactionV1) -> Element {
    let entry_point: TransactionEntryPoint = t.deserialize_field(ENTRY_POINT_MAP_KEY).unwrap();

    let v1_type = match entry_point {
        TransactionEntryPoint::Call | TransactionEntryPoint::Custom(_) => "Contract execution",
        TransactionEntryPoint::Transfer => "Transfer",
        TransactionEntryPoint::AddBid => "Add Bid",
        TransactionEntryPoint::WithdrawBid => "Withdraw Bid",
        TransactionEntryPoint::Delegate => "Delegate",
        TransactionEntryPoint::Undelegate => "Undelegate",
        TransactionEntryPoint::Redelegate => "Redelegate",
        TransactionEntryPoint::ActivateBid => "Activate Bid",
        TransactionEntryPoint::ChangeBidPublicKey => "Change Bid PK",
        TransactionEntryPoint::AddReservations => "Add Reservation",
        TransactionEntryPoint::CancelReservations => "Cancel Reservation",
    };

    Element::regular("Type", v1_type.to_string())
}
