//! In-process query latency histograms for the local daemon (Phase 6 / #102 thin slice).
//!
//! No external Prometheus/GlitchTip dependency — counters + fixed latency buckets,
//! exposed as Prometheus text via `GET /api/metrics`.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Upper bounds in milliseconds for latency histogram buckets (+Inf is implicit).
const LATENCY_BOUNDS_MS: &[u64] = &[
    1, 2, 5, 10, 25, 50, 100, 250, 500, 1_000, 2_500, 5_000, 10_000,
];

/// Named HTTP/MCP query operations we track in this thin slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricOp {
    Search,
    Ask,
    Graph,
    Cite,
    WhoKnows,
}

impl MetricOp {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Search => "search",
            Self::Ask => "ask",
            Self::Graph => "graph",
            Self::Cite => "cite",
            Self::WhoKnows => "who_knows",
        }
    }
}

#[derive(Debug, Default)]
struct OpMetrics {
    requests: AtomicU64,
    errors: AtomicU64,
    /// Sum of result counts (hits / nodes / entries) across successful calls.
    results_sum: AtomicU64,
    /// Latency sum in milliseconds.
    latency_sum_ms: AtomicU64,
    /// Per-bucket counts (inclusive upper bound); last slot is +Inf.
    buckets: [AtomicU64; 14],
}

impl OpMetrics {
    fn observe(&self, duration_ms: u64, result_count: u64, ok: bool) {
        self.requests.fetch_add(1, Ordering::Relaxed);
        if !ok {
            self.errors.fetch_add(1, Ordering::Relaxed);
        } else {
            self.results_sum.fetch_add(result_count, Ordering::Relaxed);
        }
        self.latency_sum_ms
            .fetch_add(duration_ms, Ordering::Relaxed);
        let idx = LATENCY_BOUNDS_MS
            .iter()
            .position(|&b| duration_ms <= b)
            .unwrap_or(LATENCY_BOUNDS_MS.len());
        self.buckets[idx].fetch_add(1, Ordering::Relaxed);
    }

    fn snapshot(&self) -> OpSnapshot {
        let mut buckets = [0u64; 14];
        for (i, b) in self.buckets.iter().enumerate() {
            buckets[i] = b.load(Ordering::Relaxed);
        }
        OpSnapshot {
            requests: self.requests.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            results_sum: self.results_sum.load(Ordering::Relaxed),
            latency_sum_ms: self.latency_sum_ms.load(Ordering::Relaxed),
            buckets,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct OpSnapshot {
    requests: u64,
    errors: u64,
    results_sum: u64,
    latency_sum_ms: u64,
    buckets: [u64; 14],
}

impl OpSnapshot {
    fn quantile_ms(&self, q: f64) -> Option<u64> {
        if self.requests == 0 {
            return None;
        }
        let target = ((self.requests as f64) * q).ceil().max(1.0) as u64;
        let mut cum = 0u64;
        for (i, &count) in self.buckets.iter().enumerate() {
            cum = cum.saturating_add(count);
            if cum >= target {
                return Some(if i < LATENCY_BOUNDS_MS.len() {
                    LATENCY_BOUNDS_MS[i]
                } else {
                    LATENCY_BOUNDS_MS[LATENCY_BOUNDS_MS.len() - 1].saturating_mul(2)
                });
            }
        }
        Some(LATENCY_BOUNDS_MS[LATENCY_BOUNDS_MS.len() - 1].saturating_mul(2))
    }
}

/// Process-wide (per daemon) metrics registry.
#[derive(Debug, Default)]
pub struct MetricsRegistry {
    search: OpMetrics,
    ask: OpMetrics,
    graph: OpMetrics,
    cite: OpMetrics,
    who_knows: OpMetrics,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    fn op(&self, op: MetricOp) -> &OpMetrics {
        match op {
            MetricOp::Search => &self.search,
            MetricOp::Ask => &self.ask,
            MetricOp::Graph => &self.graph,
            MetricOp::Cite => &self.cite,
            MetricOp::WhoKnows => &self.who_knows,
        }
    }

    /// Record one completed operation.
    pub fn observe(&self, op: MetricOp, duration_ms: u64, result_count: u64, ok: bool) {
        self.op(op).observe(duration_ms, result_count, ok);
    }

    /// JSON summary (p50/p90/p99 approx from buckets) for `/api/status` and CLI.
    pub fn summary_json(&self) -> serde_json::Value {
        let ops = [
            MetricOp::Search,
            MetricOp::Ask,
            MetricOp::Graph,
            MetricOp::Cite,
            MetricOp::WhoKnows,
        ];
        let mut map = serde_json::Map::new();
        for op in ops {
            let s = self.op(op).snapshot();
            let avg = if s.requests == 0 {
                0.0
            } else {
                s.latency_sum_ms as f64 / s.requests as f64
            };
            map.insert(
                op.as_str().to_string(),
                serde_json::json!({
                    "requests": s.requests,
                    "errors": s.errors,
                    "results_sum": s.results_sum,
                    "latency_sum_ms": s.latency_sum_ms,
                    "latency_avg_ms": avg,
                    "latency_p50_ms": s.quantile_ms(0.50),
                    "latency_p90_ms": s.quantile_ms(0.90),
                    "latency_p99_ms": s.quantile_ms(0.99),
                }),
            );
        }
        serde_json::Value::Object(map)
    }

    /// Prometheus text exposition for `GET /api/metrics`.
    pub fn render_prometheus(&self) -> String {
        let mut out = String::with_capacity(2048);
        out.push_str("# HELP kurultai_query_requests_total Total query operations.\n");
        out.push_str("# TYPE kurultai_query_requests_total counter\n");
        out.push_str("# HELP kurultai_query_errors_total Failed query operations.\n");
        out.push_str("# TYPE kurultai_query_errors_total counter\n");
        out.push_str("# HELP kurultai_query_results_total Sum of result counts on success.\n");
        out.push_str("# TYPE kurultai_query_results_total counter\n");
        out.push_str("# HELP kurultai_query_latency_ms Query latency in milliseconds.\n");
        out.push_str("# TYPE kurultai_query_latency_ms histogram\n");

        for op in [
            MetricOp::Search,
            MetricOp::Ask,
            MetricOp::Graph,
            MetricOp::Cite,
            MetricOp::WhoKnows,
        ] {
            let s = self.op(op).snapshot();
            let label = op.as_str();
            out.push_str(&format!(
                "kurultai_query_requests_total{{op=\"{label}\"}} {}\n",
                s.requests
            ));
            out.push_str(&format!(
                "kurultai_query_errors_total{{op=\"{label}\"}} {}\n",
                s.errors
            ));
            out.push_str(&format!(
                "kurultai_query_results_total{{op=\"{label}\"}} {}\n",
                s.results_sum
            ));

            let mut cumulative = 0u64;
            for (i, &bound) in LATENCY_BOUNDS_MS.iter().enumerate() {
                cumulative = cumulative.saturating_add(s.buckets[i]);
                out.push_str(&format!(
                    "kurultai_query_latency_ms_bucket{{op=\"{label}\",le=\"{bound}\"}} {cumulative}\n"
                ));
            }
            cumulative = cumulative.saturating_add(s.buckets[LATENCY_BOUNDS_MS.len()]);
            out.push_str(&format!(
                "kurultai_query_latency_ms_bucket{{op=\"{label}\",le=\"+Inf\"}} {cumulative}\n"
            ));
            out.push_str(&format!(
                "kurultai_query_latency_ms_sum{{op=\"{label}\"}} {}\n",
                s.latency_sum_ms
            ));
            out.push_str(&format!(
                "kurultai_query_latency_ms_count{{op=\"{label}\"}} {}\n",
                s.requests
            ));
        }
        out
    }
}

/// RAII timer that records into a registry on drop (or explicit finish).
pub struct TimedObserve {
    metrics: Arc<MetricsRegistry>,
    op: MetricOp,
    start: Instant,
    result_count: u64,
    ok: bool,
    finished: bool,
}

impl TimedObserve {
    pub fn start(metrics: Arc<MetricsRegistry>, op: MetricOp) -> Self {
        Self {
            metrics,
            op,
            start: Instant::now(),
            result_count: 0,
            ok: false,
            finished: false,
        }
    }

    pub fn success(mut self, result_count: u64) {
        self.result_count = result_count;
        self.ok = true;
        self.finish();
    }

    pub fn failure(mut self) {
        self.ok = false;
        self.result_count = 0;
        self.finish();
    }

    fn finish(&mut self) {
        if self.finished {
            return;
        }
        self.finished = true;
        let ms = self.start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        self.metrics
            .observe(self.op, ms, self.result_count, self.ok);
    }
}

impl Drop for TimedObserve {
    fn drop(&mut self) {
        // Cancelled / early return without success/failure → count as error.
        if !self.finished {
            self.ok = false;
            self.finish();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram_buckets_and_prometheus() {
        let m = MetricsRegistry::new();
        m.observe(MetricOp::Search, 12, 3, true);
        m.observe(MetricOp::Search, 80, 1, true);
        m.observe(MetricOp::Ask, 5, 0, false);

        let text = m.render_prometheus();
        assert!(text.contains("kurultai_query_requests_total{op=\"search\"} 2"));
        assert!(text.contains("kurultai_query_errors_total{op=\"ask\"} 1"));
        assert!(text.contains("kurultai_query_latency_ms_bucket{op=\"search\",le=\"25\"}"));
        assert!(text.contains("le=\"+Inf\""));

        let json = m.summary_json();
        assert_eq!(json["search"]["requests"], 2);
        assert_eq!(json["search"]["results_sum"], 4);
        assert!(json["search"]["latency_p50_ms"].as_u64().unwrap() >= 10);
        assert_eq!(json["ask"]["errors"], 1);
    }

    #[test]
    fn timed_observe_records_success() {
        let m = MetricsRegistry::shared();
        {
            let t = TimedObserve::start(Arc::clone(&m), MetricOp::Graph);
            t.success(7);
        }
        assert_eq!(m.summary_json()["graph"]["requests"], 1);
        assert_eq!(m.summary_json()["graph"]["results_sum"], 7);
    }
}
