//! Shared wire-only RPC harness compiled once against registry providers and
//! once against the upgraded providers. Do not ship this debugging executable.
#![forbid(unsafe_code)]
use std::io::{self, BufRead, Write};
use std::num::NonZeroU16;

use anyhow::{Context, Result, bail};
use davey::{DaveSession, MediaType, ProposalsOperationType};
use openmls::prelude::{tls_codec::*, *};
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;
use serde_json::{Value, json};

#[derive(Default)]
struct Peer {
    session: Option<DaveSession>,
    delivery_signer: Option<SignatureKeyPair>,
}

fn bytes(request: &Value, field: &str) -> Result<Vec<u8>> {
    serde_json::from_value(request[field].clone()).with_context(|| format!("missing/invalid {field} byte array"))
}

fn number(request: &Value, field: &str) -> Result<u64> {
    request[field].as_u64().with_context(|| format!("missing/invalid {field} integer"))
}

impl Peer {
    fn session(&self) -> Result<&DaveSession> {
        self.session.as_ref().context("session not initialized")
    }

    fn session_mut(&mut self) -> Result<&mut DaveSession> {
        self.session.as_mut().context("session not initialized")
    }

    fn rpc(&mut self, request: Value) -> Result<Value> {
        match request["op"].as_str().context("missing operation")? {
            "delivery" => {
                let signer = SignatureKeyPair::new(SignatureScheme::ECDSA_SECP256R1_SHA256)?;
                let external = ExternalSender::new(
                    signer.public().into(),
                    BasicCredential::new(b"offline-interop-delivery-service".to_vec()).into(),
                ).tls_serialize_detached()?;
                self.delivery_signer = Some(signer);
                Ok(json!({"external": external, "helper": env!("CARGO_PKG_NAME")}))
            }
            "setup" => {
                let mut session = DaveSession::new(
                    NonZeroU16::new(1).unwrap(), number(&request, "user")?, number(&request, "channel")?, None,
                )?;
                session.set_external_sender(&bytes(&request, "external")?)?;
                self.session = Some(session);
                Ok(json!({"helper": env!("CARGO_PKG_NAME")}))
            }
            "keypackage" => Ok(json!(self.session_mut()?.create_key_package()?)),
            "add" => {
                let provider = OpenMlsRustCrypto::default();
                let package = KeyPackageIn::tls_deserialize_exact_bytes(&bytes(&request, "keypackage")?)?
                    .validate(provider.crypto(), ProtocolVersion::Mls10)?;
                let group = self.session()?.group().context("missing MLS group")?;
                let proposal = ExternalProposal::new_add::<OpenMlsRustCrypto>(
                    package, group.group_id().clone(), group.epoch(),
                    self.delivery_signer.as_ref().context("delivery signer not initialized")?,
                    SenderExtensionIndex::new(0),
                )?;
                let payload = VLBytes::new(proposal.tls_serialize_detached()?).tls_serialize_detached()?;
                Ok(json!(payload))
            }
            "proposals" => {
                let payload = bytes(&request, "bytes")?;
                let expected: Vec<u64> = serde_json::from_value(request["expected"].clone())?;
                let output = self.session_mut()?.process_proposals(ProposalsOperationType::APPEND, &payload, Some(&expected))?;
                Ok(output.map(|out| json!({"commit": out.commit, "welcome": out.welcome})).unwrap_or(Value::Null))
            }
            "commit" => {
                self.session_mut()?.process_commit(&bytes(&request, "bytes")?)?;
                Ok(Value::Null)
            }
            "welcome" => {
                self.session_mut()?.process_welcome(&bytes(&request, "bytes")?)?;
                Ok(Value::Null)
            }
            "encrypt" => {
                let plaintext = bytes(&request, "bytes")?;
                let ciphertext = self.session_mut()?.encrypt_opus(&plaintext)?.into_owned();
                Ok(json!(ciphertext))
            }
            "decrypt" => {
                let user = number(&request, "user")?;
                let packet = bytes(&request, "bytes")?;
                Ok(json!(self.session_mut()?.decrypt(user, MediaType::AUDIO, &packet)?))
            }
            "info" => {
                let session = self.session()?;
                let verification = request["verify_user"].as_u64().map(|user| session.get_verification_code(user)).transpose()?;
                Ok(json!({
                    "helper": env!("CARGO_PKG_NAME"), "user": session.user_id(),
                    "ready": session.is_ready(), "status": format!("{:?}", session.status()),
                    "epoch": session.epoch().map(|epoch| epoch.as_u64()),
                    "authenticator": session.get_epoch_authenticator().map(|auth| auth.as_slice()),
                    "privacy_code": session.voice_privacy_code(), "users": session.get_user_ids(),
                    "verification": verification,
                }))
            }
            other => bail!("unknown operation: {other}"),
        }
    }
}

fn main() -> Result<()> {
    let mut peer = Peer::default();
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        let result = serde_json::from_str(&line).context("invalid JSON request").and_then(|request| peer.rpc(request));
        let response = match result {
            Ok(output) => json!({"ok": output}),
            Err(error) => json!({"error": format!("{error:#}")}),
        };
        serde_json::to_writer(&mut stdout, &response)?;
        writeln!(&mut stdout)?;
        stdout.flush()?;
    }
    Ok(())
}
