use casper_types::system::auction::{DelegationRate, DelegatorKind, Reservation};
use casper_types::{
    runtime_args, AsymmetricType, PublicKey, RuntimeArgs, TransactionArgs, TransactionEntryPoint, TransactionScheduling, TransactionTarget,
};

use crate::sample::Sample;

use crate::test_data::TransactionV1Meta;

/// Represents native delegation sample.
#[derive(Clone, Debug)]
struct CancelReservations {
    validator: PublicKey,
    delegators: Vec<DelegatorKind>,
}

impl CancelReservations {
    pub fn new(
        validator: PublicKey,
        delegators: Vec<DelegatorKind>,
    ) -> Self {
        Self {
            validator,
            delegators
        }
    }
}

impl From<CancelReservations> for RuntimeArgs {
    fn from(d: CancelReservations) -> Self {
        let mut args = RuntimeArgs::new();
        args.insert("validator", d.validator).unwrap();
        args.insert("delegators", d.delegators).unwrap();
        args
    }
}

fn native_cancel_reservations_samples(
    validators: &[PublicKey],
    delegators_mul: &[Vec<DelegatorKind>],
) -> Vec<Sample<CancelReservations>> {
    let mut samples: Vec<Sample<CancelReservations>> = vec![];

    for validator in validators {
        for delegators in delegators_mul {
            samples.push(Sample::new(
                format!("native_cancel_reservations"),
                CancelReservations::new(validator.clone(), delegators.clone()),
                true
            ));
        }
    }

    samples
}

/// Returns valid native delegate samples.
pub(crate) fn valid() -> Vec<Sample<TransactionV1Meta>> {
    let delegator_kinds_1 = vec![
        DelegatorKind::PublicKey(PublicKey::ed25519_from_bytes([6u8; 32]).unwrap()),
        DelegatorKind::PublicKey(PublicKey::secp256k1_from_bytes(
            hex::decode(b"026e1b7a8e3243f5ff14e825b0fde15103588bb61e6ae99084968b017118e0504f")
                .unwrap(),
        )
        .unwrap()),
        DelegatorKind::Purse([9u8; 32]),
    ];

    let delegator_kinds_2 = vec![
        DelegatorKind::PublicKey(PublicKey::ed25519_from_bytes([6u8; 32]).unwrap()),
    ];

    let delegator_kinds_3 = vec![
        DelegatorKind::Purse([9u8; 32]),
    ];

    let delegator_kinds_mul = vec![
        delegator_kinds_1,
        delegator_kinds_2,
        delegator_kinds_3
    ];

    let validator_pks = vec![
        PublicKey::secp256k1_from_bytes(
            hex::decode(b"026e1b7a8e3243f5ff14e825b0fde15103588bb61e6ae99084968b017118e0504f")
                .unwrap(),
        )
        .unwrap(),
        PublicKey::ed25519_from_bytes([1u8; 32]).unwrap(),
        PublicKey::ed25519_from_bytes([3u8; 32]).unwrap(),
    ];

    super::make_samples_with_schedulings(
        native_cancel_reservations_samples(
            &validator_pks,
            &delegator_kinds_mul
        ),
        TransactionEntryPoint::CancelReservations,
    )
}

/// Returns invalid native transfer samples.
pub(crate) fn invalid() -> Vec<Sample<TransactionV1Meta>> {
    let validator = PublicKey::ed25519_from_bytes([6u8; 32]).unwrap();
    let delegators = vec![DelegatorKind::Purse([9u8; 32]),];

    let missing_validator_pk = runtime_args! {
        "delegators" => delegators,
    };

    let missing_delegators = runtime_args! {
        "validator" => validator,
    };

    let invalid_args = vec![
        Sample::new("missing_validator_pk", missing_validator_pk, false),
        Sample::new("missing_delegators", missing_delegators, false),
    ];

    invalid_args
        .into_iter()
        .map(|sample_ra| {
            let (label, ra, validity) = sample_ra.destructure();
            let sample_invalid_delegate = TransactionV1Meta::new(
                TransactionArgs::Named(ra),
                TransactionTarget::Native,
                TransactionEntryPoint::CancelReservations,
                TransactionScheduling::Standard,
            );
            let new_label = format!("native_cancel_reservations_{}", label);
            Sample::new(new_label, sample_invalid_delegate, validity)
        })
        .collect()
}
