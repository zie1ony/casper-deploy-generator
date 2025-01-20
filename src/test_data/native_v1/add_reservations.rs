use casper_types::system::auction::{DelegationRate, DelegatorKind, Reservation};
use casper_types::{
    runtime_args, AsymmetricType, PublicKey, RuntimeArgs, TransactionArgs, TransactionEntryPoint,
    TransactionScheduling, TransactionTarget,
};

use crate::sample::Sample;

use crate::test_data::TransactionV1Meta;

/// Represents native 'add reservations' sample.
#[derive(Clone, Debug)]
struct AddReservations {
    reservations: Vec<Reservation>,
}

impl AddReservations {
    pub fn new(reservations: Vec<Reservation>) -> Self {
        Self { reservations }
    }
}

impl From<AddReservations> for RuntimeArgs {
    fn from(d: AddReservations) -> Self {
        let mut args = RuntimeArgs::new();
        args.insert("reservations", d.reservations).unwrap();
        args
    }
}

// Generate a native add reservations sample for every possible combination of parameters
fn native_add_reservations_samples(
    delegator_kinds: &[DelegatorKind],
    validator_public_keys: &[PublicKey],
    delegation_rates: &[DelegationRate],
) -> Vec<Sample<AddReservations>> {
    let mut samples: Vec<Sample<AddReservations>> = vec![];

    let mut reservations = vec![];
    for delegator_kind in delegator_kinds {
        for validator_pk in validator_public_keys {
            for delegation_rate in delegation_rates {
                reservations.push(Reservation::new(
                    validator_pk.clone(),
                    delegator_kind.clone(),
                    *delegation_rate,
                ));
            }
        }
    }

    for length in 0..4 {
        let sub_reservations = reservations.iter().take(length).cloned().collect();
        samples.push(Sample::new(
            format!("native_add_reservations_{length}_elements"),
            AddReservations::new(sub_reservations),
            true,
        ));
    }

    samples
}

/// Returns valid add reservations samples.
pub(crate) fn valid() -> Vec<Sample<TransactionV1Meta>> {
    let delegator_kinds = vec![
        DelegatorKind::PublicKey(PublicKey::ed25519_from_bytes([6u8; 32]).unwrap()),
        DelegatorKind::PublicKey(
            PublicKey::secp256k1_from_bytes(
                hex::decode(b"026e1b7a8e3243f5ff14e825b0fde15103588bb61e6ae99084968b017118e0504f")
                    .unwrap(),
            )
            .unwrap(),
        ),
        DelegatorKind::Purse([9u8; 32]),
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

    let delegation_rates = vec![1, 2, 4];

    super::make_samples_with_schedulings(
        native_add_reservations_samples(&delegator_kinds, &validator_pks, &delegation_rates),
        TransactionEntryPoint::AddReservations,
    )
}

/// Returns invalid add reservations samples.
pub(crate) fn invalid() -> Vec<Sample<TransactionV1Meta>> {
    let missing_reservations = runtime_args! {};

    let invalid_args = vec![Sample::new(
        "missing_reservations",
        missing_reservations,
        false,
    )];

    invalid_args
        .into_iter()
        .map(|sample_ra| {
            let (label, ra, validity) = sample_ra.destructure();
            let sample_invalid_delegate = TransactionV1Meta::new(
                TransactionArgs::Named(ra),
                TransactionTarget::Native,
                TransactionEntryPoint::AddReservations,
                TransactionScheduling::Standard,
            );
            let new_label = format!("native_add_reservations_{}", label);
            Sample::new(new_label, sample_invalid_delegate, validity)
        })
        .collect()
}
