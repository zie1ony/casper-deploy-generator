use std::{collections::BTreeMap, convert::TryInto};

use casper_types::{
    bytesrepr::{Bytes, ToBytes},
    ExecutableDeployItem, InitiatorAddr, PricingMode, Transaction, TransactionArgs,
    TransactionEntryPoint, TransactionInvocationTarget, TransactionRuntimeParams,
    TransactionScheduling, TransactionTarget, TransactionV1, TransactionV1Hash,
    TransactionV1Payload, U512,
};
use deterministic::DeterministicTestRng;
use ledger::{LimitedLedgerConfig, ZondaxRepr};
use parser::v1::{ARGS_MAP_KEY, ENTRY_POINT_MAP_KEY, SCHEDULING_MAP_KEY, TARGET_MAP_KEY};
use sample::Sample;
use test_data::{
    deploy_delegate_samples, deploy_generic_samples, deploy_native_transfer_samples, deploy_redelegate_samples, deploy_undelegate_samples, native_activate_bid_samples, native_add_bid_samples, native_delegate_samples, native_redelegate_samples, native_undelegate_samples, sign_message::{invalid_casper_message_sample, valid_casper_message_sample}, v1_native_transfer_samples
};

pub mod checksummed_hex;
mod deterministic;
mod ledger;
mod message;
mod parser;
mod sample;
mod test_data;
mod utils;

fn transaction_deploys() -> impl Iterator<Item = Sample<Transaction>> {
    // Single rng is created here and used for all generators to minimize diff per old Deploy test vectors.
    let mut rng = DeterministicTestRng::default();

    deploy_undelegate_samples(&mut rng)
        .into_iter()
        .chain(deploy_delegate_samples(&mut rng))
        .chain(deploy_native_transfer_samples(&mut rng))
        .chain(deploy_redelegate_samples(&mut rng))
        .chain(deploy_generic_samples(&mut rng))
}

fn deploy_to_v1_generic(
    sample: Sample<Transaction>,
    ep: TransactionEntryPoint,
    runtime: TransactionRuntimeParams,
    new_suffix: &'static str,
) -> Result<Sample<Transaction>, ()> {
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
    let hash = TransactionV1Hash::new(*deploy.hash().inner());

    let target = match deploy.session() {
        ExecutableDeployItem::ModuleBytes { module_bytes, .. } => TransactionTarget::Session {
            is_install_upgrade: false,
            module_bytes: module_bytes.clone(),
            runtime: runtime.clone(),
        },
        ExecutableDeployItem::StoredContractByHash { hash, .. } => TransactionTarget::Stored {
            id: TransactionInvocationTarget::ByHash(hash.value()),
            runtime: runtime.clone(),
        },
        ExecutableDeployItem::StoredContractByName { name, .. } => TransactionTarget::Stored {
            id: TransactionInvocationTarget::ByName(name.into()),
            runtime: runtime.clone(),
        },
        ExecutableDeployItem::StoredVersionedContractByHash { hash, version, .. } => {
            TransactionTarget::Stored {
                id: TransactionInvocationTarget::ByPackageHash {
                    addr: hash.value(),
                    version: *version,
                },
                runtime: runtime.clone(),
            }
        }
        ExecutableDeployItem::StoredVersionedContractByName { name, version, .. } => {
            TransactionTarget::Stored {
                id: TransactionInvocationTarget::ByPackageName {
                    name: name.into(),
                    version: *version,
                },
                runtime: runtime.clone(),
            }
        }
        ExecutableDeployItem::Transfer { .. } => TransactionTarget::Native,
    };

    let scheduling = TransactionScheduling::Standard;
    let fields = {
        let mut fields: BTreeMap<u16, Bytes> = BTreeMap::new();
        let args = deploy.session().args();
        let args = match &runtime {
            TransactionRuntimeParams::VmCasperV1 => {
                TransactionArgs::Named(args.clone())
            },
            TransactionRuntimeParams::VmCasperV2 { .. } => {
                TransactionArgs::Bytesrepr(args.to_bytes().unwrap().into())
            },
        };
        fields.insert(ARGS_MAP_KEY, args.to_bytes().unwrap().into());
        fields.insert(ENTRY_POINT_MAP_KEY, ep.to_bytes().unwrap().into());
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

fn generic_samples_v1(rng: &mut DeterministicTestRng) -> Vec<Sample<Transaction>> {
    deploy_generic_samples(rng)
        .into_iter()
        .filter_map(|sample| {
            deploy_to_v1_generic(
                sample,
                TransactionEntryPoint::Custom("generic-txn-entrypoint".into()),
                TransactionRuntimeParams::VmCasperV1,
                "_generic_sample_v1",
            )
            .ok()
        })
        .collect()
}

fn generic_samples_v1_vm2(rng: &mut DeterministicTestRng) -> Vec<Sample<Transaction>> {
    deploy_generic_samples(rng)
        .into_iter()
        .filter_map(|sample| {
            deploy_to_v1_generic(
                sample,
                TransactionEntryPoint::Custom("generic-vm2-ep".into()),
                TransactionRuntimeParams::VmCasperV2 { transferred_value: 0, seed: None },
                "_generic_sample_v1_vm2",
            )
            .ok()
        })
        .collect()
}

fn transaction_v1s() -> impl Iterator<Item = Sample<Transaction>> {
    // Single rng is created here and used for all generators to minimize diff per old Deploy test vectors.
    let mut rng = DeterministicTestRng::default();

    v1_native_transfer_samples(&mut rng)
        .into_iter()
        .chain(native_delegate_samples(&mut rng))
        .chain(native_undelegate_samples(&mut rng))
        .chain(native_redelegate_samples(&mut rng))
        .chain(native_add_bid_samples(&mut rng))
        .chain(native_activate_bid_samples(&mut rng))
        .chain(generic_samples_v1(&mut rng))
        .chain(generic_samples_v1_vm2(&mut rng))
}

fn main() {
    let page_limit = 15;

    let limited_ledger_config = LimitedLedgerConfig::new(page_limit);

    let mut id = 0;
    let mut data: Vec<ZondaxRepr> = vec![];

    for sample_transaction in transaction_deploys() {
        data.push(ledger::transaction_to_json(
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
