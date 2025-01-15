use casper_types::{account::AccountHash, runtime_args, AccessRights, AsymmetricType, CLValue, ExecutableDeployItem, Key, PublicKey, RuntimeArgs, TransactionArgs, TransactionEntryPoint, TransactionScheduling, TransactionTarget, URef, U512};

use crate::{sample::Sample, test_data::{native_transfer::{native_transfer_samples, TransferSource, TransferTarget}, TransactionV1Meta}};

use super::super::commons::UREF_ADDR;

/// Returns valid native transfer samples.
pub(crate) fn valid() -> Vec<Sample<TransactionV1Meta>> {
    let amount_min = U512::from(0u8);
    let amount_mid = U512::from(100000000);
    let amount_max = U512::MAX;
    let amounts = vec![amount_min, amount_mid, amount_max];
    let id_min = u64::MIN;
    let id_max = u64::MAX;
    let transfer_id = vec![id_min, id_max];
    let targets = vec![
        TransferTarget::bytes(),
        TransferTarget::uref(),
        TransferTarget::key(),
        TransferTarget::public_key_secp256k1(),
        TransferTarget::public_key_ed25519(),
    ];

    let access_rights = vec![
        AccessRights::READ,
        AccessRights::WRITE,
        AccessRights::ADD,
        AccessRights::READ_ADD,
        AccessRights::READ_WRITE,
        AccessRights::READ_ADD_WRITE,
    ];

    let sources: Vec<TransferSource> = access_rights
        .into_iter()
        .map(|ar| TransferSource::uref(URef::new(UREF_ADDR, ar)))
        .chain(vec![TransferSource::none()])
        .collect();

    native_transfer_samples(&amounts, &transfer_id, &targets, &sources)
        .into_iter()
        .map(|s| {
            let (label, sample, validity) = s.destructure();
            let transaction = TransactionV1Meta::new(
                TransactionArgs::Named(sample.into()),
                TransactionTarget::Native,
                TransactionEntryPoint::Transfer,
                TransactionScheduling::Standard
            );
            Sample::new(
                format!("v1_{label}"),
                transaction,
                validity,
            )
        })
        .collect()
}

/// Returns invalid native transfer samples.
pub(crate) fn invalid() -> Vec<Sample<TransactionV1Meta>> {
    let missing_required_amount: RuntimeArgs = runtime_args! {
        "id" => 1u64,
        "target" => URef::new(UREF_ADDR, AccessRights::READ),
    };
    let missing_required_id: RuntimeArgs = runtime_args! {
        "amount" => U512::from(100000000u64),
        "target" => URef::new(UREF_ADDR, AccessRights::READ),
    };
    let missing_required_target: RuntimeArgs = runtime_args! {
        "amount" => U512::from(100000000u64),
        "id" => 1u64,
    };
    let invalid_amount_type: RuntimeArgs = runtime_args! {
        "amount" => 10000u64,
        "target" => URef::new(UREF_ADDR, AccessRights::READ),
        "id" => 1u64,
    };

    let invalid_transfer_args: Vec<Sample<RuntimeArgs>> = vec![
        Sample::new("missing_amount", missing_required_amount, false),
        Sample::new("missing_id", missing_required_id, false),
        Sample::new("missing_target", missing_required_target, false),
        Sample::new("invalid_type_amount", invalid_amount_type, false),
    ];

    invalid_transfer_args
        .into_iter()
        .map(|sample_ra| {
            let (label, ra, validity) = sample_ra.destructure();
            let transaction = TransactionV1Meta::new(
                TransactionArgs::Named(ra), 
                TransactionTarget::Native,
                TransactionEntryPoint::Transfer,
                TransactionScheduling::Standard
            );
            let new_label = format!("v1_native_transfer_{}", label);
            Sample::new(new_label, transaction, validity)
        })
        .collect()
}