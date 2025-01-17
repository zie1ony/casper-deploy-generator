use casper_types::{
    runtime_args, AsymmetricType, PublicKey, RuntimeArgs, TransactionArgs, TransactionEntryPoint,
    TransactionScheduling, TransactionTarget,
};

use crate::sample::Sample;

use crate::test_data::TransactionV1Meta;

/// Represents native delegation sample.
#[derive(Clone, Debug)]
struct ChangeBidPk {
    public_key: PublicKey,
    new_public_key: PublicKey,
}

impl ChangeBidPk {
    pub fn new(public_key: PublicKey, new_public_key: PublicKey) -> Self {
        Self {
            public_key,
            new_public_key,
        }
    }
}

impl From<ChangeBidPk> for RuntimeArgs {
    fn from(d: ChangeBidPk) -> Self {
        let mut args = RuntimeArgs::new();
        args.insert("public_key", d.public_key).unwrap();
        args.insert("new_public_key", d.new_public_key).unwrap();
        args
    }
}

fn native_change_bid_pk_samples(
    public_keys: &[PublicKey],
    new_public_keys: &[PublicKey],
) -> Vec<Sample<ChangeBidPk>> {
    let mut samples: Vec<Sample<ChangeBidPk>> = vec![];

    for old_pk in public_keys {
        for new_pk in new_public_keys {
            let label = "native_change_bid_public_key_v1".to_string();
            let bid = ChangeBidPk::new(old_pk.clone(), new_pk.clone());
            let sample = Sample::new(label, bid, true);
            samples.push(sample);
        }
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

    let new_public_keys = vec![
        PublicKey::ed25519_from_bytes([6u8; 32]).unwrap(),
        PublicKey::ed25519_from_bytes([9u8; 32]).unwrap(),
        PublicKey::secp256k1_from_bytes(
            hex::decode(b"026e1b7a8e3243f5ff14e825b0fde15103588bb61e6ae99084968b017118e0504f")
                .unwrap(),
        )
        .unwrap(),
    ];

    super::make_samples_with_schedulings(
        native_change_bid_pk_samples(&public_keys, &new_public_keys),
        TransactionEntryPoint::ChangeBidPublicKey,
    )
}

/// Returns invalid native transfer samples.
pub(crate) fn invalid() -> Vec<Sample<TransactionV1Meta>> {
    let public_key = PublicKey::ed25519_from_bytes([6u8; 32]).unwrap();
    let new_public_key = PublicKey::ed25519_from_bytes([0u8; 32]).unwrap();

    let missing_pk = runtime_args! {
        "new_public_key" => new_public_key
    };

    let missing_new_pk = runtime_args! {
        "public_key" => public_key
    };

    let invalid_args = vec![
        Sample::new("missing_public_key", missing_pk, false),
        Sample::new("missing_new_public_key", missing_new_pk, false),
    ];

    invalid_args
        .into_iter()
        .map(|sample_ra| {
            let (label, ra, validity) = sample_ra.destructure();
            let sample_invalid_delegate = TransactionV1Meta::new(
                TransactionArgs::Named(ra),
                TransactionTarget::Native,
                TransactionEntryPoint::ChangeBidPublicKey,
                TransactionScheduling::Standard,
            );
            let new_label = format!("native_change_bid_public_key_{}", label);
            Sample::new(new_label, sample_invalid_delegate, validity)
        })
        .collect()
}
