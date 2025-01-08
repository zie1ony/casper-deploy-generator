use std::{collections::BTreeMap, convert::TryInto};

use casper_types::{
    bytesrepr::{Bytes, ToBytes},
    Deploy, ExecutableDeployItem, InitiatorAddr, PricingMode, Transaction, TransactionEntryPoint,
    TransactionScheduling, TransactionTarget, TransactionV1, TransactionV1Hash,
    TransactionV1Payload, U512,
};
use deterministic::DeterministicTestRng;
use ledger::{LimitedLedgerConfig, ZondaxRepr};
use sample::Sample;
use strum::EnumIter;
use test_data::{
    delegate_samples, generic_samples, native_transfer_samples, redelegate_samples,
    undelegate_samples,
};

use crate::test_data::sign_message::{invalid_casper_message_sample, valid_casper_message_sample};

pub mod checksummed_hex;
mod deterministic;
mod ledger;
mod message;
mod parser;
mod sample;
mod test_data;
mod utils;

const ARGS_MAP_KEY: u16 = 0;
const TARGET_MAP_KEY: u16 = 1;
const ENTRY_POINT_MAP_KEY: u16 = 2;
const SCHEDULING_MAP_KEY: u16 = 3;

fn transaction_deploys() -> impl Iterator<Item = Sample<Deploy>> {
    // Single rng is created here and used for all generators to minimize diff per old Deploy test vectors.
    let mut rng = DeterministicTestRng::default();

    undelegate_samples(&mut rng)
        .into_iter()
        .chain(delegate_samples(&mut rng))
        .chain(native_transfer_samples(&mut rng))
        .chain(redelegate_samples(&mut rng))
        .chain(generic_samples(&mut rng))
        .map(|sample| {
            let (label, transaction, valid) = sample.destructure();
            let deploy = match transaction {
                Transaction::Deploy(deploy) => deploy,
                _ => unreachable!(),
            };
            Sample::new(label, deploy, valid)
        })
}

fn delegate_samples_v1() -> Vec<Sample<Transaction>> {
    let mut rng = DeterministicTestRng::default();

    let mut vec = Vec::new();

    for deploy in delegate_samples(&mut rng) {
        let (mut label, transaction_deploy, is_valid) = deploy.destructure();

        let deploy = match transaction_deploy {
            Transaction::Deploy(deploy) => deploy,
            _ => unreachable!(),
        };

        let approvals = deploy.approvals().clone();

        let pricing_mode = match deploy.payment() {
            ExecutableDeployItem::ModuleBytes { module_bytes, args } => {
                let payment_amount = match args.get("amount") {
                    Some(amount) => {
                        let amount = amount.clone();
                        let u512_value: U512 = amount.into_t().unwrap();
                        let payment_amount: u64 =
                            u512_value.try_into().expect("U512 too large to fit in u64");
                        payment_amount
                    }
                    None => {
                        // Unable to reasonably translate invalid payment arguments (i.e. there's no "amount" arg) to a valid payment mode.
                        continue;
                    }
                };

                PricingMode::PaymentLimited {
                    payment_amount,
                    gas_price_tolerance: 1,
                    standard_payment: module_bytes.is_empty(),
                }
            }
            _other => unreachable!("Unexpected ExecutableDeployItem variant"),
        };

        let initiator_addr = InitiatorAddr::AccountHash(deploy.account().clone().to_account_hash());
        let hash = TransactionV1Hash::new(deploy.hash().inner().clone());

        let target = TransactionTarget::Native;
        let scheduling = TransactionScheduling::Standard;
        let fields = {
            let mut fields: BTreeMap<u16, Bytes> = BTreeMap::new();
            let session_args = deploy.session().args().clone();
            fields.insert(ARGS_MAP_KEY, session_args.to_bytes().unwrap().into());
            fields.insert(
                ENTRY_POINT_MAP_KEY,
                TransactionEntryPoint::Delegate.to_bytes().unwrap().into(),
            );
            fields.insert(TARGET_MAP_KEY, target.to_bytes().unwrap().into());
            fields.insert(SCHEDULING_MAP_KEY, scheduling.to_bytes().unwrap().into());
            fields
        };

        let payload = TransactionV1Payload::new(
            deploy.chain_name().to_owned(),
            deploy.timestamp(),
            deploy.ttl(),
            pricing_mode,
            initiator_addr,
            fields,
        );

        // let transaction_hash_v1 = TransactionV1::new(hash, payload, approvals);
        let transaction_v1 = TransactionV1::new(hash, payload, approvals);
        let transaction = Transaction::V1(transaction_v1);

        label.push_str("_delegate_sample_v1");

        vec.push(Sample::new(label, transaction, is_valid));
    }

    vec
}

fn transaction_v1s() -> impl Iterator<Item = Sample<Transaction>> {
    // Single rng is created here and used for all generators to minimize diff per old Deploy test vectors.
    delegate_samples_v1().into_iter()
}

fn main() {
    let page_limit = 15;

    let limited_ledger_config = LimitedLedgerConfig::new(page_limit);

    let mut id = 0;
    let mut data: Vec<ZondaxRepr> = vec![];

    for sample_transaction in transaction_deploys() {
        data.push(ledger::deploy_to_json(
            id,
            sample_transaction,
            &limited_ledger_config,
        ));
        id += 1;
    }

    for sample_casper_message in valid_casper_message_sample()
        .into_iter()
        .chain(invalid_casper_message_sample())
    {
        data.push(ledger::message_to_json(
            id,
            sample_casper_message,
            &limited_ledger_config,
        ));
        id += 1;
    }

    for _sample_transaction_v1 in transaction_v1s() {
        // TODO: Implement this
        // data.push(ledger::transaction_to_json(
        //     id,
        //     sample_transaction_v1,
        //     &limited_ledger_config,
        // ));
        // id += 1;
    }

    println!("{}", serde_json::to_string_pretty(&data).unwrap());
}
