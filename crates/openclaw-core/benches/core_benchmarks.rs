//! Performance benchmarks for openclaw-core.
//!
//! Run with: cargo bench -p openclaw-core

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use openclaw_core::events::{EventStore, SessionEvent, SessionEventKind};
use openclaw_core::types::{AgentId, ChannelId, PeerId, PeerType, SessionKey};
use tempfile::TempDir;

/// Benchmark event store append operations.
fn bench_event_append(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_append");

    // Test different batch sizes
    for batch_size in [1, 10, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch", batch_size),
            batch_size,
            |b, &size| {
                let temp_dir = TempDir::new().unwrap();
                let store = EventStore::open(temp_dir.path()).unwrap();

                let session_key = SessionKey::build(
                    &AgentId::new("bench-agent"),
                    &ChannelId::telegram(),
                    "bench-account",
                    PeerType::Dm,
                    &PeerId::new("bench-peer"),
                );

                b.iter(|| {
                    for i in 0..size {
                        let event = SessionEvent::new(
                            session_key.clone(),
                            "bench-agent".to_string(),
                            SessionEventKind::MessageReceived {
                                content: format!("Benchmark message {i}"),
                                attachments: vec![],
                            },
                        );
                        store.append(black_box(&event)).unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark session key construction.
fn bench_session_key(c: &mut Criterion) {
    c.bench_function("session_key_build", |b| {
        let agent_id = AgentId::new("test-agent");
        let channel_id = ChannelId::telegram();
        let peer_id = PeerId::new("test-peer");

        b.iter(|| {
            SessionKey::build(
                black_box(&agent_id),
                black_box(&channel_id),
                black_box("test-account"),
                black_box(PeerType::Dm),
                black_box(&peer_id),
            )
        });
    });
}

/// Benchmark event projection retrieval.
fn bench_projection_read(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let store = EventStore::open(temp_dir.path()).unwrap();

    // Seed with some events
    let session_key = SessionKey::build(
        &AgentId::new("bench-agent"),
        &ChannelId::telegram(),
        "bench-account",
        PeerType::Dm,
        &PeerId::new("bench-peer"),
    );

    // Create session
    let start_event = SessionEvent::new(
        session_key.clone(),
        "bench-agent".to_string(),
        SessionEventKind::SessionStarted {
            channel: "telegram".to_string(),
            peer_id: "bench-peer".to_string(),
        },
    );
    store.append(&start_event).unwrap();

    // Add some messages
    for i in 0..100 {
        let event = SessionEvent::new(
            session_key.clone(),
            "bench-agent".to_string(),
            SessionEventKind::MessageReceived {
                content: format!("Message {i}"),
                attachments: vec![],
            },
        );
        store.append(&event).unwrap();
    }

    c.bench_function("projection_read", |b| {
        b.iter(|| {
            store.get_projection(black_box(&session_key)).unwrap()
        });
    });
}

/// Benchmark event store cold start.
fn bench_cold_start(c: &mut Criterion) {
    c.bench_function("event_store_cold_start", |b| {
        b.iter_with_setup(
            || TempDir::new().unwrap(),
            |temp_dir| {
                let _store = EventStore::open(black_box(temp_dir.path())).unwrap();
            },
        );
    });
}

/// Benchmark JSON serialization of events.
fn bench_event_serialization(c: &mut Criterion) {
    let session_key = SessionKey::build(
        &AgentId::new("bench-agent"),
        &ChannelId::telegram(),
        "bench-account",
        PeerType::Dm,
        &PeerId::new("bench-peer"),
    );

    let event = SessionEvent::new(
        session_key,
        "bench-agent".to_string(),
        SessionEventKind::MessageReceived {
            content: "A typical message that might be sent to the agent".to_string(),
            attachments: vec![],
        },
    );

    c.bench_function("event_serialize", |b| {
        b.iter(|| serde_json::to_vec(black_box(&event)).unwrap());
    });

    let serialized = serde_json::to_vec(&event).unwrap();

    c.bench_function("event_deserialize", |b| {
        b.iter(|| {
            serde_json::from_slice::<SessionEvent>(black_box(&serialized)).unwrap()
        });
    });
}

criterion_group!(
    benches,
    bench_event_append,
    bench_session_key,
    bench_projection_read,
    bench_cold_start,
    bench_event_serialization,
);
criterion_main!(benches);
