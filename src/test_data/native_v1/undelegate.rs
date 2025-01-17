use casper_types::{
    runtime_args, AsymmetricType, PublicKey, RuntimeArgs, TransactionArgs, TransactionEntryPoint,
    TransactionScheduling, TransactionTarget, U512,
};

use crate::sample::Sample;

use crate::test_data::TransactionV1Meta;

/// Represents native delegation sample.
#[derive(Clone, Debug)]
struct NativeUndelegate {
    delegator: PublicKey,
    validator: PublicKey,
    amount: U512,
}

impl NativeUndelegate {
    pub fn new(delegator: PublicKey, validator: PublicKey, amount: U512) -> Self {
        Self {
            delegator,
            validator,
            amount,
        }
    }
}

impl From<NativeUndelegate> for RuntimeArgs {
    fn from(d: NativeUndelegate) -> Self {
        let mut args = RuntimeArgs::new();
        args.insert("delegator", d.delegator).unwrap();
        args.insert("validator", d.validator).unwrap();
        args.insert("amount", d.amount).unwrap();
        args
    }
}

// Generate a native delegate sample for every possible combination of parameters
fn native_undelegate_samples(
    amounts: &[U512],
    validators: &[PublicKey],
    delegators: &[PublicKey],
) -> Vec<Sample<NativeUndelegate>> {
    let mut samples: Vec<Sample<NativeUndelegate>> = vec![];

    for amount in amounts {
        for validator in validators {
            for delegator in delegators {
                let label = "native_undelegate_v1".to_string();
                let nt = NativeUndelegate::new(delegator.clone(), validator.clone(), *amount);
                let sample = Sample::new(label, nt, true);
                samples.push(sample);
            }
        }
    }

    samples
}

/// Returns valid native delegate samples.
pub(crate) fn valid() -> Vec<Sample<TransactionV1Meta>> {
    let amount_min = U512::from(0u8);
    let amount_mid = U512::from(100000000);
    let amount_max = U512::MAX;
    let amounts = vec![amount_min, amount_mid, amount_max];

    let validators = vec![
        PublicKey::secp256k1_from_bytes(
            hex::decode(b"026e1b7a8e3243f5ff14e825b0fde15103588bb61e6ae99084968b017118e0504f")
                .unwrap(),
        )
        .unwrap(),
        PublicKey::ed25519_from_bytes([1u8; 32]).unwrap(),
        PublicKey::ed25519_from_bytes([3u8; 32]).unwrap(),
    ];

    let delegators = vec![
        PublicKey::ed25519_from_bytes([6u8; 32]).unwrap(),
        PublicKey::secp256k1_from_bytes(
            hex::decode(b"026e1b7a8e3243f5ff14e825b0fde15103588bb61e6ae99084968b017118e0504f")
                .unwrap(),
        )
        .unwrap(),
        PublicKey::ed25519_from_bytes([11u8; 32]).unwrap(),
    ];

    super::make_samples_with_schedulings(
        native_undelegate_samples(&amounts, &validators, &delegators),
        TransactionEntryPoint::Undelegate,
    )
}

/// Returns invalid native transfer samples.
pub(crate) fn invalid() -> Vec<Sample<TransactionV1Meta>> {
    let valid_validator = PublicKey::ed25519_from_bytes([0u8; 32]).unwrap();
    let valid_delegator = PublicKey::ed25519_from_bytes([9u8; 32]).unwrap();

    let valid_amount = U512::from(10_000_000u64);

    let missing_required_amount = runtime_args! {
        "validator" => valid_validator.clone(),
        "delegator" => valid_delegator.clone(),
    };

    let missing_required_validator = runtime_args! {
        "delegator" => valid_delegator.clone(),
        "amount" => valid_amount,
    };

    let missing_required_delegator = runtime_args! {
        "validator" => valid_validator.clone(),
        "amount" => valid_amount,
    };

    let invalid_amount_type = runtime_args! {
        "amount" => 10000u64,
        "validator" => valid_validator.clone(),
        "amount" => valid_amount,
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
                TransactionEntryPoint::Undelegate,
                TransactionScheduling::Standard,
            );
            let new_label = format!("native_undelegate_{}", label);
            Sample::new(new_label, sample_invalid_delegate, validity)
        })
        .collect()
}
