// Verifies that telemetry::init() wires a non-noop TracerProvider.
//
// Run the full stdout export check manually:
//   cargo build --quiet && \
//   ./target/debug/visdom-harness &>/tmp/vh.log & PID=$!; \
//   sleep 1; curl -fsS http://127.0.0.1:3000/health; sleep 1; \
//   kill $PID; grep 'http.health' /tmp/vh.log

use opentelemetry::trace::{Tracer, TracerProvider as _};
use std::sync::OnceLock;

static INIT: OnceLock<()> = OnceLock::new();

fn init_once() {
    INIT.get_or_init(|| {
        visdom_harness::telemetry::init("info").expect("telemetry init failed");
    });
}

#[test]
fn tracer_provider_is_non_noop() {
    init_once();

    let provider = opentelemetry::global::tracer_provider();
    let tracer = provider.tracer("test");
    // A noop tracer returns an invalid span context; a real SDK tracer returns a valid one.
    let span = tracer.start("test-span");
    use opentelemetry::trace::Span as _;
    assert!(
        span.span_context().is_valid(),
        "expected a valid (non-noop) span from the SDK tracer provider"
    );
}
