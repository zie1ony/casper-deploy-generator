use casper_types::{runtime_args, AsymmetricType, PublicKey, RuntimeArgs, TransactionArgs, TransactionEntryPoint, TransactionScheduling, TransactionTarget, U512};

use crate::sample::Sample;

use crate::test_data::TransactionV1Meta;

/// Represents native delegation sample.
#[derive(Clone, Debug)]
struct NativeRedelegate {
    delegator: PublicKey,
    validator: PublicKey,
    amount: U512,
    new_validator: PublicKey,
}

impl NativeRedelegate {
    pub fn new(delegator: PublicKey, validator: PublicKey, new_validator: PublicKey, amount: U512) -> Self {
        Self {
            delegator,
            validator,
            new_validator,
            amount
        }
    }
}

impl From<NativeRedelegate> for RuntimeArgs {
    fn from(d: NativeRedelegate) -> Self {
        let mut args = RuntimeArgs::new();
        args.insert("delegator", d.delegator).unwrap();
        args.insert("validator", d.validator).unwrap();
        args.insert("new_validator", d.new_validator).unwrap();
        args.insert("amount", d.amount).unwrap();
        args
    }
}

// Generate a native delegate sample for every possible combination of parameters
fn native_undelegate_samples(
    amounts: &[U512],
    validators: &[PublicKey],
    delegators: &[PublicKey],
) -> Vec<Sample<NativeRedelegate>> {
    let mut samples: Vec<Sample<NativeRedelegate>> = vec![];

    for amount in amounts {
        for validator_old in validators {
            for validator_new in validators {
                if validator_new == validator_old {
                    continue;
                }
                for delegator in delegators {
                    let label = format!("native_redelegate_v1");
                    let nt = NativeRedelegate::new(
                        delegator.clone(),
                        validator_old.clone(),
                        validator_new.clone(),
                        *amount
                    );
                    let sample = Sample::new(label, nt, true);
                    samples.push(sample);
                }
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
        PublicKey::ed25519_from_bytes([0u8; 32]).unwrap(),
        PublicKey::ed25519_from_bytes([1u8; 32]).unwrap(),
        PublicKey::ed25519_from_bytes([3u8; 32]).unwrap(),
    ];

    let delegators = vec![
        PublicKey::ed25519_from_bytes([6u8; 32]).unwrap(),
        PublicKey::ed25519_from_bytes([9u8; 32]).unwrap(),
        PublicKey::ed25519_from_bytes([11u8; 32]).unwrap(),
    ];

    native_undelegate_samples(&amounts, &validators, &delegators)
        .into_iter()
        .map(|s| {
            let (label, sample, validity) = s.destructure();
            Sample::new(
                label,
                TransactionV1Meta::new(
                    TransactionArgs::Named(sample.into()),
                    TransactionTarget::Native,
                    TransactionEntryPoint::Redelegate,
                    TransactionScheduling::Standard
                ),
                validity,
            )
        })
        .collect()
}

/// Returns invalid native transfer samples.
pub(crate) fn invalid() -> Vec<Sample<TransactionV1Meta>> {
    let valid_validator_1 = PublicKey::ed25519_from_bytes([0u8; 32]).unwrap();
    let valid_validator_2 = PublicKey::ed25519_from_bytes([6u8; 32]).unwrap();
    let valid_delegator = PublicKey::ed25519_from_bytes([9u8; 32]).unwrap();

    let valid_amount = U512::from(10_000_000u64);

    let missing_required_amount = runtime_args! {
        "delegator" => valid_delegator.clone(),
        "validator" => valid_validator_1.clone(),
        "new_validator" => valid_validator_2.clone(),
    };

    let missing_required_delegator = runtime_args! {
        "validator" => valid_validator_1.clone(),
        "new_validator" => valid_validator_2.clone(),
        "amount" => valid_amount,
    };

    let missing_required_validator = runtime_args! {
        "delegator" => valid_delegator.clone(),
        "new_validator" => valid_validator_1.clone(),
        "amount" => valid_amount
    };

    let missing_required_new_validator = runtime_args! {
        "delegator" => valid_delegator.clone(),
        "validator" => valid_validator_1.clone(),
        "amount" => valid_amount,
    };

    let invalid_amount_type = runtime_args! {
        "delegator" => valid_delegator,
        "validator" => valid_validator_1,
        "new_validator" => valid_validator_2,
        "amount" => 100000u32,
    };

    let invalid_args= vec![
        Sample::new("missing_amount", missing_required_amount, true),
        Sample::new("missing_delegator", missing_required_delegator, true),
        Sample::new("missing_validator", missing_required_validator, true),
        Sample::new(
            "missing_new_validator",
            missing_required_new_validator,
            false,
        ),
        Sample::new("invalid_type_amount", invalid_amount_type, true),
    ];

    invalid_args
        .into_iter()
        .map(|sample_ra| {
            let (label, ra, validity) = sample_ra.destructure();
            let sample_invalid_delegate = TransactionV1Meta::new(
                TransactionArgs::Named(ra),
                TransactionTarget::Native,
                TransactionEntryPoint::Redelegate,
                TransactionScheduling::Standard
            );
            let new_label = format!("native_redelegate_{}", label);
            Sample::new(new_label, sample_invalid_delegate, validity)
        })
        .collect()
}
