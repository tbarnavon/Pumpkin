//! Pumpkin's `fabric:registry/sync/direct` for Storage Drawers must carry the same bytes Fabric's
//! server sends for the same mod. The reference payloads were produced by
//! `DirectRegistryPacketHandler.sendPacket` on a Fabric 1.21.1 server with Storage Drawers
//! 1.21.1-13.11.4 (Extractor mod dump).
//!
//! The registry overlay is process-global, so everything that needs it runs in one test.
#![allow(clippy::expect_used, clippy::panic)]

use std::path::Path;

use base64::Engine;
use pumpkin_fabric::handshake::{FabricHandshake, Outgoing, PING_ID, Step};
use pumpkin_fabric::sync_map;
use pumpkin_fabric::wire::{common, register, registry_sync};

fn reference_payloads() -> Vec<Vec<u8>> {
    let text = std::fs::read_to_string("tests/fixtures/fabric_registry_sync_payload.json")
        .expect("fixture");
    let json: serde_json::Value = serde_json::from_str(&text).expect("json");
    assert_eq!(json["channel"], registry_sync::SYNC_CHANNEL);
    json["payloads"]
        .as_array()
        .expect("payloads")
        .iter()
        .map(|payload| {
            base64::engine::general_purpose::STANDARD
                .decode(payload.as_str().expect("payload"))
                .expect("base64")
        })
        .collect()
}

fn payloads(step: &Step, channel: &str) -> Vec<Vec<u8>> {
    let Step::Send(out) = step else {
        panic!("expected packets, got {step:?}");
    };
    out.iter()
        .filter_map(|o| match o {
            Outgoing::Payload { channel: c, data } if *c == channel => Some(data.clone()),
            _ => None,
        })
        .collect()
}

fn payload(step: &Step, channel: &str) -> Vec<u8> {
    payloads(step, channel)
        .pop()
        .unwrap_or_else(|| panic!("no {channel} in {step:?}"))
}

#[test]
fn storage_drawers_sync_is_byte_identical_and_handshake_completes() {
    let dumps = pumpkin_registry_ext::read_dumps(Path::new(
        "../pumpkin-registry-ext/tests/fixtures/mod-data",
    ))
    .expect("read dumps");
    pumpkin_registry_ext::install(dumps).expect("install");

    let reference = reference_payloads();
    let ours = registry_sync::encode(&sync_map::build());
    // Compare decoded first for a readable failure, then the exact bytes.
    assert_eq!(
        registry_sync::decode(&ours).expect("decode ours"),
        registry_sync::decode(&reference.concat()).expect("decode reference")
    );
    // Fabric sends its buffer's whole backing array: the same bytes, then zeros up to the
    // buffer's capacity, which the client never reads.
    let reference_body = reference.concat();
    assert_eq!(&reference_body[..ours.len()], ours.as_slice());
    assert!(reference_body[ours.len()..].iter().all(|byte| *byte == 0));
    let ours_sent = registry_sync::into_payloads(&ours);
    assert_eq!(ours_sent.len(), reference.len());

    // Fabric client: register -> sync -> complete -> c:version -> c:register -> done.
    let mut handshake = FabricHandshake::new(
        vec!["storagedrawers".to_string()],
        vec!["storagedrawers:player_bool_config".to_string()],
    );
    let start = handshake.start();
    assert!(start.contains(&Outgoing::Ping(PING_ID)));
    let client_channels = register::encode(&[
        registry_sync::SYNC_CHANNEL,
        common::VERSION_CHANNEL,
        common::REGISTER_CHANNEL,
    ]);
    let sync = handshake.on_payload(register::REGISTER_CHANNEL, &client_channels);
    assert_eq!(payloads(&sync, registry_sync::SYNC_CHANNEL), ours_sent);
    // The client also answers the ping; that must not be taken for a vanilla client now.
    assert_eq!(handshake.on_pong(PING_ID), Step::Wait);
    let version = handshake.on_payload(registry_sync::SYNC_COMPLETE_CHANNEL, &[]);
    assert_eq!(
        payload(&version, common::VERSION_CHANNEL),
        common::encode_version(&[1])
    );
    let register = handshake.on_payload(common::VERSION_CHANNEL, &common::encode_version(&[1]));
    let announced =
        common::decode_register(&payload(&register, common::REGISTER_CHANNEL)).expect("c:register");
    assert_eq!(announced.protocol, "play");
    assert_eq!(announced.channels, ["storagedrawers:player_bool_config"]);
    let done = handshake.on_payload(
        common::REGISTER_CHANNEL,
        &common::encode_register(1, "play", &["storagedrawers:count_update"]),
    );
    assert_eq!(done, Step::Done(Vec::new()));
    assert!(handshake.is_done());
    assert_eq!(
        handshake.client_play_channels(),
        ["storagedrawers:count_update"]
    );
    assert_eq!(
        handshake.on_payload("minecraft:brand", b"x"),
        Step::NotHandled
    );

    // Vanilla client: the pong comes back without any registration.
    let mut vanilla = FabricHandshake::new(vec!["storagedrawers".to_string()], Vec::new());
    assert!(
        matches!(vanilla.on_pong(PING_ID), Step::Disconnect(message) if message.contains("storagedrawers"))
    );

    // Fabric Loader without registry sync support.
    let mut no_sync = FabricHandshake::new(vec!["storagedrawers".to_string()], Vec::new());
    assert!(matches!(
        no_sync.on_payload(
            register::REGISTER_CHANNEL,
            &register::encode(&["c:version"])
        ),
        Step::Disconnect(_)
    ));
}
