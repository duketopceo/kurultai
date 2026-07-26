//! In-memory ring of recent brain tool activity for Live UI pathways.

use serde::Serialize;
use std::collections::VecDeque;
use std::sync::Mutex;

const CAP: usize = 64;

#[derive(Debug, Clone, Serialize)]
pub struct ActivityEvent {
    pub seq: u64,
    pub ts_unix: u64,
    pub tool: String,
    pub query: String,
    pub atom_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Default)]
pub struct ActivityLog {
    inner: Mutex<Inner>,
}

#[derive(Debug, Default)]
struct Inner {
    next_seq: u64,
    events: VecDeque<ActivityEvent>,
}

impl ActivityLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(
        &self,
        tool: &str,
        query: &str,
        atom_ids: Vec<String>,
        detail: Option<String>,
    ) {
        let mut g = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let seq = g.next_seq;
        g.next_seq = g.next_seq.saturating_add(1);
        let ts_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        g.events.push_back(ActivityEvent {
            seq,
            ts_unix,
            tool: tool.to_string(),
            query: query.to_string(),
            atom_ids,
            detail,
        });
        while g.events.len() > CAP {
            g.events.pop_front();
        }
    }

    /// Events with `seq >= since`, plus the next seq the client should poll with.
    pub fn since(&self, since: u64) -> (u64, Vec<ActivityEvent>) {
        let g = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let events: Vec<ActivityEvent> = g
            .events
            .iter()
            .filter(|e| e.seq >= since)
            .cloned()
            .collect();
        (g.next_seq, events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_caps_and_since_filters() {
        let log = ActivityLog::new();
        for i in 0..70 {
            log.record("search", &format!("q{i}"), vec![format!("id{i}")], None);
        }
        let (next, ev) = log.since(0);
        assert_eq!(next, 70);
        assert_eq!(ev.len(), CAP);
        assert_eq!(ev[0].seq, 6); // 0..5 dropped
        let (next2, ev2) = log.since(68);
        assert_eq!(next2, 70);
        assert_eq!(ev2.len(), 2);
        assert_eq!(ev2[0].seq, 68);
    }
}
