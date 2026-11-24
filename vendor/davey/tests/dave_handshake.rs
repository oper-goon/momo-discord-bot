//! Offline DAVE protocol exercise with a locally generated delivery-service signer.
//! No Discord credentials, sockets, media decoding or production fixtures.
use std::num::NonZeroU16;

use davey::{DaveSession, MediaType, ProposalsOperationType, SessionStatus};
use openmls::prelude::{tls_codec::*, *};
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;

const ALICE: u64 = 1001;
const BOB: u64 = 1002;
const CHARLIE: u64 = 1003;
const CHANNEL: u64 = 9001;

fn new_session(user_id: u64, external_sender: &[u8]) -> DaveSession {
    let mut session = DaveSession::new(NonZeroU16::new(1).unwrap(), user_id, CHANNEL, None)
        .expect("create DAVE session");
    session
        .set_external_sender(external_sender)
        .expect("install delivery service external sender");
    session
}

fn external_add(
    existing: &DaveSession,
    joiner: &mut DaveSession,
    delivery_signer: &SignatureKeyPair,
) -> Vec<u8> {
    let provider = OpenMlsRustCrypto::default();
    let key_package = KeyPackageIn::tls_deserialize_exact_bytes(
        &joiner.create_key_package().expect("create joining key package"),
    )
    .expect("parse serialized joining key package")
    .validate(provider.crypto(), ProtocolVersion::Mls10)
    .expect("validate joining key package using backend crypto");
    let group = existing.group().expect("existing MLS group");
    let proposal = ExternalProposal::new_add::<OpenMlsRustCrypto>(
        key_package,
        group.group_id().clone(),
        group.epoch(),
        delivery_signer,
        SenderExtensionIndex::new(0),
    )
    .expect("sign external Add proposal");
    let serialized = proposal
        .tls_serialize_detached()
        .expect("serialize MLS Add message");
    // Discord's proposals body is a TLS variable-length byte vector containing
    // concatenated MLS messages, not the bare message serialization.
    VLBytes::new(serialized)
        .tls_serialize_detached()
        .expect("frame Discord proposals vector")
}

fn assert_same_epoch(left: &DaveSession, right: &DaveSession, members: &[u64]) {
    assert_eq!(left.status(), SessionStatus::ACTIVE);
    assert_eq!(right.status(), SessionStatus::ACTIVE);
    assert!(left.is_ready() && right.is_ready());
    assert_eq!(left.epoch(), right.epoch());
    assert_eq!(
        left.get_epoch_authenticator().unwrap().as_slice(),
        right.get_epoch_authenticator().unwrap().as_slice(),
    );
    assert_eq!(left.voice_privacy_code(), right.voice_privacy_code());
    assert!(left.voice_privacy_code().is_some());
    for session in [left, right] {
        let mut actual = session.get_user_ids().unwrap();
        actual.sort_unstable();
        assert_eq!(actual, members);
    }
}

fn assert_audio_roundtrip(sender: &mut DaveSession, receiver: &mut DaveSession, discriminator: u8) {
    // Deliberately not OPUS_SILENCE_PACKET: Davey passes silence through without
    // encryption. These bytes exercise packet framing, AEAD and sender ratchets;
    // neither peer decodes Opus in this offline test.
    let plaintext = [0x78, discriminator, 0x32, 0x43, 0x54, 0x65, 0x76, 0x87];
    let encrypted = sender
        .encrypt_opus(&plaintext)
        .expect("encrypt audio frame")
        .into_owned();
    assert_ne!(encrypted, plaintext);
    assert!(encrypted.len() > plaintext.len());
    let decrypted = receiver
        .decrypt(sender.user_id(), MediaType::AUDIO, &encrypted)
        .expect("decrypt remote sender audio");
    assert_eq!(decrypted, plaintext);
}

#[test]
fn two_peer_handshake_audio_and_third_member_epoch_rotation() {
    let delivery_signer = SignatureKeyPair::new(SignatureScheme::ECDSA_SECP256R1_SHA256)
        .expect("generate local delivery service signer");
    let external_sender = ExternalSender::new(
        delivery_signer.public().into(),
        BasicCredential::new(b"offline-test-delivery-service".to_vec()).into(),
    )
    .tls_serialize_detached()
    .expect("serialize external sender package");

    let mut alice = new_session(ALICE, &external_sender);
    let mut bob = new_session(BOB, &external_sender);
    assert!(!alice.is_ready());
    assert!(!bob.is_ready());
    assert!(alice.encrypt_opus(&[0x78, 1, 2, 3]).is_err());

    let proposal = external_add(&alice, &mut bob, &delivery_signer);
    let transition = alice
        .process_proposals(ProposalsOperationType::APPEND, &proposal, Some(&[ALICE, BOB]))
        .expect("process signed Bob Add proposal")
        .expect("proposal produces commit and welcome");
    assert_eq!(alice.status(), SessionStatus::AWAITING_RESPONSE);
    alice
        .process_commit(&transition.commit)
        .expect("delivery service echoes own commit; merge pending state");
    bob.process_welcome(transition.welcome.as_ref().expect("welcome for new member"))
        .expect("Bob joins from serialized welcome");

    assert_same_epoch(&alice, &bob, &[ALICE, BOB]);
    assert_eq!(
        alice.get_verification_code(BOB).unwrap(),
        bob.get_verification_code(ALICE).unwrap(),
    );
    assert_audio_roundtrip(&mut alice, &mut bob, 1);
    assert_audio_roundtrip(&mut bob, &mut alice, 2);
    let old_epoch = alice.epoch().unwrap().as_u64();
    let old_authenticator = alice.get_epoch_authenticator().unwrap().as_slice().to_vec();

    // A third member forces both an own-commit merge and remote staged-commit
    // merge, followed by a welcome and a fresh epoch's media-key derivation.
    let mut charlie = new_session(CHARLIE, &external_sender);
    let proposal = external_add(&alice, &mut charlie, &delivery_signer);
    let transition = alice
        .process_proposals(ProposalsOperationType::APPEND, &proposal, Some(&[ALICE, BOB, CHARLIE]))
        .expect("Alice processes Charlie Add")
        .unwrap();
    bob.process_proposals(ProposalsOperationType::APPEND, &proposal, Some(&[ALICE, BOB, CHARLIE]))
        .expect("Bob stores matching Add proposal");
    alice.process_commit(&transition.commit).expect("Alice merges own new epoch");
    bob.process_commit(&transition.commit).expect("Bob merges remote new epoch");
    charlie.process_welcome(transition.welcome.as_ref().unwrap()).expect("Charlie joins new epoch");

    assert!(alice.epoch().unwrap().as_u64() > old_epoch);
    assert_ne!(alice.get_epoch_authenticator().unwrap().as_slice(), old_authenticator);
    assert_same_epoch(&alice, &bob, &[ALICE, BOB, CHARLIE]);
    assert_same_epoch(&bob, &charlie, &[ALICE, BOB, CHARLIE]);
    assert_audio_roundtrip(&mut alice, &mut bob, 3);
    assert_audio_roundtrip(&mut bob, &mut alice, 4);
    assert_audio_roundtrip(&mut alice, &mut charlie, 5);
    assert_audio_roundtrip(&mut charlie, &mut bob, 6);

    let mut tampered = alice.encrypt_opus(&[0x78, 7, 8, 9]).unwrap().into_owned();
    tampered[1] ^= 0x40;
    assert!(bob.decrypt(ALICE, MediaType::AUDIO, &tampered).is_err());
}
