use casper_types::{
    account::AccountHash, runtime_args, AccessRights, AsymmetricType, CLValue,
    ExecutableDeployItem, Key, PublicKey, RuntimeArgs, URef, U512,
};

use crate::sample::Sample;

use super::commons::UREF_ADDR;

/// Represents native transfer sample.
#[derive(Clone, Debug)]
pub(crate) struct NativeTransfer {
    target: TransferTarget,
    amount: U512,
    id: u64,
    source: TransferSource,
}

impl NativeTransfer {
    pub fn new(target: TransferTarget, amount: U512, id: u64, source: TransferSource) -> Self {
        NativeTransfer {
            target,
            amount,
            id,
            source,
        }
    }
}

impl From<NativeTransfer> for RuntimeArgs {
    fn from(nt: NativeTransfer) -> Self {
        let mut ra = RuntimeArgs::new();
        ra.insert("amount", nt.amount).unwrap();
        ra.insert("id", Some(nt.id)).unwrap();
        if let TransferSource::URef(uref) = nt.source {
            ra.insert("source", uref).unwrap();
        }
        ra.insert_cl_value("target", nt.target.into_cl());
        ra
    }
}

#[derive(Clone, Debug)]
pub(crate) enum TransferSource {
    // Transfer source is account's main purse.
    None,
    // Transfer source is a defined purse.
    URef(URef),
}

impl TransferSource {
    pub fn uref(uref: URef) -> Self {
        TransferSource::URef(uref)
    }

    pub fn none() -> Self {
        TransferSource::None
    }

    pub fn label(&self) -> &str {
        match self {
            TransferSource::None => "source_none",
            TransferSource::URef(_) => "source_uref",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum TransferTarget {
    // raw bytes representing account hash
    Bytes([u8; 32]),
    // transfer to a specific purse
    URef(URef),
    // transfer to an account.
    Key(Key),
    // transfer to public key
    PublicKey(PublicKey),
}

impl TransferTarget {
    pub fn into_cl(self) -> CLValue {
        let cl_value_res = match self {
            TransferTarget::Bytes(bytes) => CLValue::from_t(bytes),
            TransferTarget::URef(uref) => CLValue::from_t(uref),
            TransferTarget::Key(key) => CLValue::from_t(key),
            TransferTarget::PublicKey(pk) => CLValue::from_t(pk),
        };
        cl_value_res.unwrap()
    }

    pub fn bytes() -> TransferTarget {
        TransferTarget::Bytes([255u8; 32])
    }

    pub fn uref() -> TransferTarget {
        let uref = URef::new(UREF_ADDR, AccessRights::READ_ADD_WRITE);
        TransferTarget::URef(uref)
    }

    pub fn key() -> TransferTarget {
        let account_key = Key::Account(
            AccountHash::from_formatted_str(
                "account-hash-45f3aa6ce2a450dd5a4f2cc4cc9054aded66de6b6cfc4ad977e7251cf94b649b",
            )
            .unwrap(),
        );
        TransferTarget::Key(account_key)
    }

    pub fn public_key_ed25519() -> TransferTarget {
        let public_key = PublicKey::ed25519_from_bytes(
            hex::decode(b"2bac1d0ff9240ff0b7b06d555815640497861619ca12583ddef434885416e69b")
                .unwrap(),
        )
        .unwrap();
        TransferTarget::PublicKey(public_key)
    }

    pub fn public_key_secp256k1() -> TransferTarget {
        let public_key = PublicKey::secp256k1_from_bytes(
            hex::decode(b"026e1b7a8e3243f5ff14e825b0fde15103588bb61e6ae99084968b017118e0504f")
                .unwrap(),
        )
        .unwrap();
        TransferTarget::PublicKey(public_key)
    }

    pub fn label(&self) -> String {
        match self {
            TransferTarget::Bytes(_) => "target_bytes".to_string(),
            TransferTarget::URef(_) => "target_uref".to_string(),
            TransferTarget::Key(_) => "target_key_account".to_string(),
            TransferTarget::PublicKey(pk) => {
                let variant = match pk {
                    PublicKey::Ed25519(_) => "ed25519",
                    PublicKey::Secp256k1(_) => "secp256k1",
                    PublicKey::System => panic!("unexpected key type variant"),
                    _ => panic!("Should not happen"),
                };
                format!("target_{}_public_key", variant)
            }
        }
    }
}

/// Given collection of native target inputs,
/// for every combination of them creates a `NativeTransfer` sample.
pub(crate) fn native_transfer_samples(
    amounts: &[U512],
    transfer_id: &[u64],
    targets: &[TransferTarget],
    sources: &[TransferSource],
) -> Vec<Sample<NativeTransfer>> {
    let mut samples: Vec<Sample<NativeTransfer>> = vec![];

    for amount in amounts {
        for id in transfer_id {
            for target in targets {
                for source in sources {
                    let label = format!("native_transfer_{}_{}", target.label(), source.label());
                    let nt = NativeTransfer::new(target.clone(), *amount, *id, source.clone());
                    let sample = Sample::new(label, nt, true);
                    samples.push(sample);
                }
            }
        }
    }

    samples
}

/// Returns valid native transfer samples.
pub(super) fn valid() -> Vec<Sample<ExecutableDeployItem>> {
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
            Sample::new(
                label,
                ExecutableDeployItem::Transfer {
                    args: sample.into(),
                },
                validity,
            )
        })
        .collect()
}

/// Returns invalid native transfer samples.
pub(super) fn invalid() -> Vec<Sample<ExecutableDeployItem>> {
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
            let sample_invalid_transfer = ExecutableDeployItem::Transfer { args: ra };
            let new_label = format!("native_transfer_{}", label);
            Sample::new(new_label, sample_invalid_transfer, validity)
        })
        .collect()
}
