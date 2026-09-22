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

/// Bucket bounds for client-reported FPS samples (+Inf implicit).
const FPS_BOUNDS: &[u64] = &[5, 10, 15, 20, 30, 45, 60, 75, 90, 120, 144, 165, 240];

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

#[derive(Debug)]
struct OpMetrics {
    requests: AtomicU64,
    errors: AtomicU64,
    /// Sum of result counts (hits / nodes / entries) across successful calls.
    results_sum: AtomicU64,
    /// Sum of observed values (ms / fps / count).
    latency_sum_ms: AtomicU64,
    /// Per-bucket counts (inclusive upper bound); last slot is +Inf.
    bounds: &'static [u64],
    buckets: Vec<AtomicU64>,
}

impl OpMetrics {
    fn new(bounds: &'static [u64]) -> Self {
        Self {
            requests: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            results_sum: AtomicU64::new(0),
            latency_sum_ms: AtomicU64::new(0),
            bounds,
            buckets: (0..=bounds.len()).map(|_| AtomicU64::new(0)).collect(),
        }
    }

    fn observe(&self, duration_ms: u64, result_count: u64, ok: bool) {
        self.requests.fetch_add(1, Ordering::Relaxed);
        if !ok {
            self.errors.fetch_add(1, Ordering::Relaxed);
        } else {
            self.results_sum.fetch_add(result_count, Ordering::Relaxed);
        }
        self.latency_sum_ms
            .fetch_add(duration_ms, Ordering::Relaxed);
        let idx = self
            .bounds
            .iter()
            .position(|&b| duration_ms <= b)
            .unwrap_or(self.bounds.len());
        self.buckets[idx].fetch_add(1, Ordering::Relaxed);
    }

    fn snapshot(&self) -> OpSnapshot {
        let buckets: Vec<u64> = self
            .buckets
            .iter()
            .map(|b| b.load(Ordering::Relaxed))
            .collect();
        OpSnapshot {
            requests: self.requests.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            results_sum: self.results_sum.load(Ordering::Relaxed),
            latency_sum_ms: self.latency_sum_ms.load(Ordering::Relaxed),
            bounds: self.bounds,
            buckets,
        }
    }
}

#[derive(Debug, Clone)]
struct OpSnapshot {
    requests: u64,
    errors: u64,
    results_sum: u64,
    latency_sum_ms: u64,
    bounds: &'static [u64],
    buckets: Vec<u64>,
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
                return Some(if i < self.bounds.len() {
                    self.bounds[i]
                } else {
                    self.bounds[self.bounds.len() - 1].saturating_mul(2)
                });
            }
        }
        Some(self.bounds[self.bounds.len() - 1].saturating_mul(2))
    }

    fn summary_json(&self) -> serde_json::Value {
        let avg = if self.requests == 0 {
            0.0
        } else {
            self.latency_sum_ms as f64 / self.requests as f64
        };
        serde_json::json!({
            "requests": self.requests,
            "errors": self.errors,
            "results_sum": self.results_sum,
            "latency_sum_ms": self.latency_sum_ms,
            "latency_avg_ms": avg,
            "latency_p50_ms": self.quantile_ms(0.50),
            "latency_p90_ms": self.quantile_ms(0.90),
            "latency_p99_ms": self.quantile_ms(0.99),
        })
    }

    fn render_prometheus(&self, out: &mut String, family: &str, label: &str) {
        out.push_str(&format!(
            "kurultai_{family}_requests_total{{op=\"{label}\"}} {}\n",
            self.requests
        ));
        out.push_str(&format!(
            "kurultai_{family}_errors_total{{op=\"{label}\"}} {}\n",
            self.errors
        ));
        out.push_str(&format!(
            "kurultai_{family}_results_total{{op=\"{label}\"}} {}\n",
            self.results_sum
        ));
        let mut cumulative = 0u64;
        for (i, &bound) in self.bounds.iter().enumerate() {
            cumulative = cumulative.saturating_add(self.buckets[i]);
            out.push_str(&format!(
                "kurultai_{family}_latency_ms_bucket{{op=\"{label}\",le=\"{bound}\"}} {cumulative}\n"
            ));
        }
        cumulative = cumulative.saturating_add(self.buckets[self.bounds.len()]);
        out.push_str(&format!(
            "kurultai_{family}_latency_ms_bucket{{op=\"{label}\",le=\"+Inf\"}} {cumulative}\n"
        ));
        out.push_str(&format!(
            "kurultai_{family}_latency_ms_sum{{op=\"{label}\"}} {}\n",
            self.latency_sum_ms
        ));
        out.push_str(&format!(
            "kurultai_{family}_latency_ms_count{{op=\"{label}\"}} {}\n",
            self.requests
        ));
    }
}

/// Metrics the Brain web client may report (`POST /api/metrics/client`).
/// Values are unitless — bounds are chosen per metric name.
const CLIENT_METRICS: &[(&str, &[u64])] = &[
    ("nav_ms", LATENCY_BOUNDS_MS),
    ("tier_load_ms", LATENCY_BOUNDS_MS),
    ("api_ms", LATENCY_BOUNDS_MS),
    ("fps", FPS_BOUNDS),
    ("long_tasks", LATENCY_BOUNDS_MS),
    ("heap_mb", LATENCY_BOUNDS_MS),
];

const CLIENT_TIERS: &[&str] = &["low", "mid", "high", "max", "none"];

/// One browser-reported perf sample.
#[derive(Debug, serde::Deserialize)]
pub struct ClientSample {
    pub metric: String,
    #[serde(default)]
    pub tier: Option<String>,
    pub value: f64,
}

/// `POST /api/metrics/client` body.
#[derive(Debug, serde::Deserialize)]
pub struct ClientReport {
    pub samples: Vec<ClientSample>,
}

/// Process-wide (per daemon) metrics registry.
#[derive(Debug)]
pub struct MetricsRegistry {
    search: OpMetrics,
    ask: OpMetrics,
    graph: OpMetrics,
    cite: OpMetrics,
    who_knows: OpMetrics,
    /// Browser-reported samples keyed `"{metric}|{tier}"`.
    client: std::sync::Mutex<std::collections::HashMap<String, OpMetrics>>,
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            search: OpMetrics::new(LATENCY_BOUNDS_MS),
            ask: OpMetrics::new(LATENCY_BOUNDS_MS),
            graph: OpMetrics::new(LATENCY_BOUNDS_MS),
            cite: OpMetrics::new(LATENCY_BOUNDS_MS),
            who_knows: OpMetrics::new(LATENCY_BOUNDS_MS),
            client: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
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

    /// Record one browser-reported sample. Returns false for unknown
    /// metric/tier names or non-finite values (caller counts rejects).
    pub fn observe_client(&self, sample: &ClientSample) -> bool {
        let Some((_, bounds)) = CLIENT_METRICS
            .iter()
            .find(|(name, _)| *name == sample.metric)
        else {
            return false;
        };
        let tier = sample.tier.as_deref().unwrap_or("none");
        let tier = if tier.is_empty() { "none" } else { tier };
        if !CLIENT_TIERS.contains(&tier) || !sample.value.is_finite() || sample.value < 0.0 {
            return false;
        }
        let value = sample.value.min(u64::MAX as f64) as u64;
        let key = format!("{}|{tier}", sample.metric);
        let mut map = self.client.lock().unwrap_or_else(|e| e.into_inner());
        map.entry(key)
            .or_insert_with(|| OpMetrics::new(bounds))
            .observe(value, 0, true);
        true
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
            map.insert(
                op.as_str().to_string(),
                self.op(op).snapshot().summary_json(),
            );
        }
        let client = self
            .client
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .map(|(k, m)| (k.clone(), m.snapshot().summary_json()))
            .collect::<serde_json::Map<String, serde_json::Value>>();
        map.insert("client".to_string(), serde_json::Value::Object(client));
        serde_json::Value::Object(map)
    }

    /// Prometheus text exposition for `GET /api/metrics`.
    pub fn render_prometheus(&self) -> String {
        let mut out = String::with_capacity(4096);
        out.push_str("# HELP kurultai_query_requests_total Total query operations.\n");
        out.push_str("# TYPE kurultai_query_requests_total counter\n");
        out.push_str("# HELP kurultai_query_errors_total Failed query operations.\n");
        out.push_str("# TYPE kurultai_query_errors_total counter\n");
        out.push_str("# HELP kurultai_query_results_total Sum of result counts on success.\n");
        out.push_str("# TYPE kurultai_query_results_total counter\n");
        out.push_str("# HELP kurultai_query_latency_ms Query latency in milliseconds.\n");
        out.push_str("# TYPE kurultai_query_latency_ms histogram\n");
        out.push_str("# HELP kurultai_client_requests_total Browser-reported perf samples.\n");
        out.push_str("# TYPE kurultai_client_requests_total counter\n");
        out.push_str(
            "# HELP kurultai_client_errors_total Client samples flagged invalid (unused).\n",
        );
        out.push_str("# TYPE kurultai_client_errors_total counter\n");
        out.push_str("# HELP kurultai_client_results_total Unused for client family.\n");
        out.push_str("# TYPE kurultai_client_results_total counter\n");
        out.push_str("# HELP kurultai_client_latency_ms Client sample value histogram (ms/fps/count per metric name).\n");
        out.push_str("# TYPE kurultai_client_latency_ms histogram\n");

        for op in [
            MetricOp::Search,
            MetricOp::Ask,
            MetricOp::Graph,
            MetricOp::Cite,
            MetricOp::WhoKnows,
        ] {
            self.op(op)
                .snapshot()
                .render_prometheus(&mut out, "query", op.as_str());
        }

        let map = self.client.lock().unwrap_or_else(|e| e.into_inner());
        let mut keys: Vec<&String> = map.keys().collect();
        keys.sort();
        for key in keys {
            map[key]
                .snapshot()
                .render_prometheus(&mut out, "client", key);
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

    #[test]
    fn client_samples_validate_and_render() {
        let m = MetricsRegistry::new();
        let ok = |metric: &str, tier: Option<&str>, value: f64| {
            m.observe_client(&ClientSample {
                metric: metric.into(),
                tier: tier.map(str::to_string),
                value,
            })
        };
        assert!(ok("nav_ms", None, 420.0));
        assert!(ok("tier_load_ms", Some("max"), 3800.0));
        assert!(ok("fps", Some("max"), 11.4));
        assert!(!ok("evil_metric", None, 1.0));
        assert!(!ok("fps", Some("bogus"), 60.0));
        assert!(!ok("fps", None, f64::NAN));
        assert!(!ok("fps", None, -5.0));

        let json = m.summary_json();
        assert_eq!(json["client"]["nav_ms|none"]["requests"], 1);
        assert_eq!(json["client"]["tier_load_ms|max"]["requests"], 1);
        assert_eq!(json["client"]["fps|max"]["latency_p50_ms"], 15);

        let text = m.render_prometheus();
        assert!(text.contains("kurultai_client_requests_total{op=\"fps|max\"} 1"));
        assert!(text.contains("le=\"15\""));
        // Query series unchanged alongside the client family.
        assert!(text.contains("kurultai_query_requests_total{op=\"search\"}"));
    }
}
