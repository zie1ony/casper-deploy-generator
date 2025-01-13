use std::{collections::BTreeMap, convert::TryInto};

use casper_types::{
    bytesrepr::{Bytes, ToBytes}, Deploy, ExecutableDeployItem, InitiatorAddr, PricingMode, Transaction, TransactionArgs, TransactionEntryPoint, TransactionInvocationTarget, TransactionRuntimeParams, TransactionScheduling, TransactionTarget, TransactionV1, TransactionV1Hash, TransactionV1Payload, U512
};
use deterministic::DeterministicTestRng;
use ledger::{LimitedLedgerConfig, ZondaxRepr};
use parser::v1::{ARGS_MAP_KEY, ENTRY_POINT_MAP_KEY, SCHEDULING_MAP_KEY, TARGET_MAP_KEY};
use sample::Sample;
use test_data::{
    delegate_samples, generic_samples, native_transfer_samples, redelegate_samples, sign_message::{invalid_casper_message_sample, valid_casper_message_sample}, undelegate_samples
};

pub mod checksummed_hex;
mod deterministic;
mod ledger;
mod message;
mod parser;
mod sample;
mod test_data;
mod utils;

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

fn deploy_to_v1(sample: Sample<Transaction>, ep: TransactionEntryPoint, new_suffix: &'static str) -> Result<Sample<Transaction>, ()> {
    let (mut label, transaction_deploy, is_valid) = sample.destructure();

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
                    return Err(());
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

    let target = match deploy.session() {
        ExecutableDeployItem::ModuleBytes { module_bytes, .. } => TransactionTarget::Session {
            is_install_upgrade: false,
            module_bytes: module_bytes.clone(),
            runtime: TransactionRuntimeParams::VmCasperV1
        },
        ExecutableDeployItem::StoredContractByHash { hash, .. } => {
            TransactionTarget::Stored {
                id: TransactionInvocationTarget::ByHash(hash.value()),
                runtime: TransactionRuntimeParams::VmCasperV1
            }
        },
        ExecutableDeployItem::StoredContractByName { name, .. } => {
            TransactionTarget::Stored {
                id: TransactionInvocationTarget::ByName(name.into()),
                runtime: TransactionRuntimeParams::VmCasperV1
            }
        },
        ExecutableDeployItem::StoredVersionedContractByHash { hash, version, .. } => {
            TransactionTarget::Stored {
                id: TransactionInvocationTarget::ByPackageHash {
                    addr: hash.value(),
                    version: version.clone()
                },
                runtime: TransactionRuntimeParams::VmCasperV1
            }
        },
        ExecutableDeployItem::StoredVersionedContractByName { name, version, .. } => {
            TransactionTarget::Stored {
                id: TransactionInvocationTarget::ByPackageName {
                    name: name.into(),
                    version: version.clone()
                },
                runtime: TransactionRuntimeParams::VmCasperV1
            }
        },
        ExecutableDeployItem::Transfer { .. } => {
            TransactionTarget::Native
        },
    };
    let scheduling = TransactionScheduling::Standard;
    let fields = {
        let mut fields: BTreeMap<u16, Bytes> = BTreeMap::new();
        let args = TransactionArgs::Named(deploy.session().args().clone());
        fields.insert(ARGS_MAP_KEY, args.to_bytes().unwrap().into());
        fields.insert(ENTRY_POINT_MAP_KEY, ep.to_bytes().unwrap().into(),);
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

    let transaction_v1 = TransactionV1::new(hash, payload, approvals);
    let transaction = Transaction::V1(transaction_v1);

    label.push_str(new_suffix);

    Ok(Sample::new(label, transaction, is_valid))
}

fn delegate_samples_v1(rng: &mut DeterministicTestRng) -> Vec<Sample<Transaction>> {
    delegate_samples(rng)
        .into_iter()
        .filter_map(|sample| deploy_to_v1(
            sample,
            TransactionEntryPoint::Delegate,
            "_delegate_sample_v1"
        ).ok())
        .collect()
}

fn native_transfer_samples_v1(rng: &mut DeterministicTestRng) -> Vec<Sample<Transaction>> {
    native_transfer_samples(rng)
        .into_iter()
        .filter_map(|sample| deploy_to_v1(
            sample,
            TransactionEntryPoint::Transfer,
            "_native_transfer_sample_v1"
        ).ok())
        .collect()
}

fn redelegate_samples_v1(rng: &mut DeterministicTestRng) -> Vec<Sample<Transaction>> {
    redelegate_samples(rng)
        .into_iter()
        .filter_map(|sample| deploy_to_v1(
            sample, 
            TransactionEntryPoint::Redelegate,
            "_redelegate_sample_v1"
        ).ok())
        .collect()
}

fn generic_samples_v1(rng: &mut DeterministicTestRng) -> Vec<Sample<Transaction>> {
    generic_samples(rng)
        .into_iter()
        .filter_map(|sample| deploy_to_v1(
            sample,
            TransactionEntryPoint::Custom("generic-txn-entrypoint".into()),
            "_generic_sample_v1"
        ).ok())
        .collect()
}

fn transaction_v1s() -> impl Iterator<Item = Sample<Transaction>> {
    // Single rng is created here and used for all generators to minimize diff per old Deploy test vectors.
    let mut rng = DeterministicTestRng::default();

    delegate_samples_v1(&mut rng).into_iter()
        .chain(native_transfer_samples_v1(&mut rng))
        .chain(redelegate_samples_v1(&mut rng))
        .chain(generic_samples_v1(&mut rng))
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

    for sample_transaction_v1 in transaction_v1s() {
        data.push(ledger::transaction_to_json(
            id,
            sample_transaction_v1,
            &limited_ledger_config,
        ));
        id += 1;
    }

    println!("{}", serde_json::to_string_pretty(&data).unwrap());
}
