use casper_types::{runtime_args, AsymmetricType, PublicKey, RuntimeArgs, TransactionArgs, TransactionEntryPoint, TransactionScheduling, TransactionTarget, U512};

use crate::sample::Sample;

use super::{NativeDelegate, TransactionV1Meta};

// Generate a native delegate sample for every possible combination of parameters
fn native_delegate_samples(
    amounts: &[U512],
    validators: &[PublicKey],
    delegators: &[PublicKey],
) -> Vec<Sample<NativeDelegate>> {
    let mut samples: Vec<Sample<NativeDelegate>> = vec![];

    for amount in amounts {
        for validator in validators {
            for delegator in delegators {
                let label = format!("native_delegate_v1");
                let nt = NativeDelegate::new(delegator.clone(), validator.clone(), *amount);
                let sample = Sample::new(label, nt, true);
                samples.push(sample);
            }
        }
    }

    samples
}

/// Returns valid native delegate samples.
pub(super) fn valid() -> Vec<Sample<TransactionV1Meta>> {
    let amount_min = U512::from(0u8);
    let amount_mid = U512::from(100000000);
    let amount_max = U512::MAX;
    let amounts = vec![amount_min, amount_mid, amount_max];

    let validators = vec![
        PublicKey::ed25519_from_bytes([0u8; 32]).unwrap(),
        PublicKey::ed25519_from_bytes([1u8; 32]).unwrap(),
        PublicKey::ed25519_from_bytes([3u8; 32]).unwrap(),
    ];

    let delegators = vec![
        PublicKey::ed25519_from_bytes([6u8; 32]).unwrap(),
        PublicKey::ed25519_from_bytes([9u8; 32]).unwrap(),
        PublicKey::ed25519_from_bytes([11u8; 32]).unwrap(),
    ];

    native_delegate_samples(&amounts, &validators, &delegators)
        .into_iter()
        .map(|s| {
            let (label, sample, validity) = s.destructure();
            Sample::new(
                label,
                TransactionV1Meta::new(
                    TransactionArgs::Named(sample.into()),
                    TransactionTarget::Native,
                    TransactionEntryPoint::Delegate,
                    TransactionScheduling::Standard
                ),
                validity,
            )
        })
        .collect()
}

/// Returns invalid native transfer samples.
pub(super) fn invalid() -> Vec<Sample<TransactionV1Meta>> {
    let valid_validator = PublicKey::ed25519_from_bytes([0u8; 32]).unwrap();
    let valid_delegator = PublicKey::ed25519_from_bytes([9u8; 32]).unwrap();

    let valid_amount = U512::from(10_000_000u64);

    let missing_required_amount = runtime_args! {
        "validator" => valid_validator.clone(),
        "delegator" => valid_delegator.clone(),
    };

    let missing_required_validator = runtime_args! {
        "delegator" => valid_delegator.clone(),
        "amount" => valid_amount.clone(),
    };

    let missing_required_delegator = runtime_args! {
        "validator" => valid_validator.clone(),
        "amount" => valid_amount.clone(),
    };

    let invalid_amount_type = runtime_args! {
        "amount" => 10000u64,
        "validator" => valid_validator.clone(),
        "amount" => valid_amount.clone(),
    };

    let invalid_transfer_args: Vec<Sample<RuntimeArgs>> = vec![
        Sample::new("missing_amount", missing_required_amount, false),
        Sample::new("missing_validator", missing_required_validator, false),
        Sample::new("missing_delegator", missing_required_delegator, false),
        Sample::new("invalid_type_amount", invalid_amount_type, false),
    ];

    invalid_transfer_args
        .into_iter()
        .map(|sample_ra| {
            let (label, ra, validity) = sample_ra.destructure();
            let sample_invalid_delegate = TransactionV1Meta::new(
                TransactionArgs::Named(ra),
                TransactionTarget::Native,
                TransactionEntryPoint::Delegate,
                TransactionScheduling::Standard
            );
            let new_label = format!("native_delegate_{}", label);
            Sample::new(new_label, sample_invalid_delegate, validity)
        })
        .collect()
}
