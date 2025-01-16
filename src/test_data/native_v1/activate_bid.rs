use casper_types::{
    runtime_args, AsymmetricType, EraId, PublicKey, RuntimeArgs, Timestamp, TransactionArgs, TransactionEntryPoint, TransactionScheduling, TransactionTarget, U512
};

use crate::sample::Sample;

use crate::test_data::TransactionV1Meta;

/// Represents native delegation sample.
#[derive(Clone, Debug)]
struct ActivateBid {
    validator_public_key: PublicKey
}

impl ActivateBid {
    pub fn new(
        validator_public_key: PublicKey
    ) -> Self {
        Self {
            validator_public_key
        }
    }
}

impl From<ActivateBid> for RuntimeArgs {
    fn from(d: ActivateBid) -> Self {
        let mut args = RuntimeArgs::new();
        args.insert("validator_public_key", d.validator_public_key).unwrap();
        args
    }
}

// Generate a native delegate sample for every possible combination of parameters
fn native_activate_bid_samples(
    validator_public_keys: &[PublicKey]
) -> Vec<Sample<ActivateBid>> {
    let mut samples: Vec<Sample<ActivateBid>> = vec![];

    for validator_pk in validator_public_keys {
        let label = format!("native_activate_bid_v1");
        let bid = ActivateBid::new(validator_pk.clone());
        let sample = Sample::new(label, bid, true);
        samples.push(sample);
    }

    samples
}

/// Returns valid native delegate samples.
pub(crate) fn valid() -> Vec<Sample<TransactionV1Meta>> {
    let public_keys = vec![
        PublicKey::ed25519_from_bytes([0u8; 32]).unwrap(),
        PublicKey::ed25519_from_bytes([1u8; 32]).unwrap(),
        PublicKey::ed25519_from_bytes([3u8; 32]).unwrap(),
    ];

    let schedulings = [
        (TransactionScheduling::Standard, "standard_scheduling"),
        (TransactionScheduling::FutureEra(EraId::new(6000)), "future_era"),
        (TransactionScheduling::FutureTimestamp(Timestamp::from(6000)), "future_timestamp"),
    ];

    super::make_samples_with_schedulings(
        native_activate_bid_samples(&public_keys),
        TransactionEntryPoint::ActivateBid,
        &schedulings
    )
}

/// Returns invalid native transfer samples.
pub(crate) fn invalid() -> Vec<Sample<TransactionV1Meta>> {
    let missing_pk = runtime_args! {};

    let invalid_args = vec![
        Sample::new("missing_public_key", missing_pk, false),
    ];

    invalid_args
        .into_iter()
        .map(|sample_ra| {
            let (label, ra, validity) = sample_ra.destructure();
            let sample_invalid_delegate = TransactionV1Meta::new(
                TransactionArgs::Named(ra),
                TransactionTarget::Native,
                TransactionEntryPoint::ActivateBid,
                TransactionScheduling::Standard,
            );
            let new_label = format!("native_activate_bid_{}", label);
            Sample::new(new_label, sample_invalid_delegate, validity)
        })
        .collect()
}
