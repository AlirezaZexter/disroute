use serde::Serialize;
use std::time::{Duration, Instant};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    checked_at: u64,
    latency_ms: Option<u64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub phase: &'static str,
    mode: Option<&'static str>,
    connected_at: Option<u64>,
    checked_at: Option<u64>,
    latency_ms: Option<u64>,
    consecutive_failures: u32,
    recoveries: u32,
    samples: Vec<Sample>,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            phase: "idle",
            mode: None,
            connected_at: None,
            checked_at: None,
            latency_ms: None,
            consecutive_failures: 0,
            recoveries: 0,
            samples: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct Monitor {
    generation: u64,
    snapshot: Snapshot,
    retry_at: Option<Instant>,
    attempts: u32,
}

impl Monitor {
    pub fn snapshot(&self) -> Snapshot {
        self.snapshot.clone()
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn active(&self) -> bool {
        self.snapshot.mode.is_some()
    }
    pub fn begin(&mut self, mode: &'static str, now: u64) {
        self.stop();
        self.snapshot.mode = Some(mode);
        self.snapshot.connected_at = Some(now);
        self.snapshot.phase = "checking";
    }
    pub fn stop(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.snapshot = Snapshot::default();
        self.retry_at = None;
        self.attempts = 0;
    }
    pub fn record(&mut self, generation: u64, latency: Option<u64>, now: u64) {
        if generation != self.generation || !self.active() {
            return;
        }
        self.snapshot.checked_at = Some(now);
        self.snapshot.latency_ms = latency;
        self.snapshot.samples.push(Sample {
            checked_at: now,
            latency_ms: latency,
        });
        if self.snapshot.samples.len() > 24 {
            self.snapshot.samples.remove(0);
        }
        if latency.is_some() {
            self.snapshot.phase = "healthy";
            self.snapshot.consecutive_failures = 0;
            self.attempts = 0;
            self.retry_at = None;
        } else {
            self.snapshot.consecutive_failures =
                self.snapshot.consecutive_failures.saturating_add(1);
            self.snapshot.phase = if self.snapshot.consecutive_failures >= 3 {
                "unavailable"
            } else {
                "degraded"
            };
        }
    }
    pub fn recovery_due(&self, now: Instant) -> bool {
        self.snapshot.mode == Some("community")
            && self.snapshot.consecutive_failures >= 3
            && self.retry_at.is_none_or(|deadline| now >= deadline)
    }
    pub fn recovering(&mut self, now: Instant) {
        if !self.active() {
            return;
        }
        self.snapshot.phase = "recovering";
        self.attempts = self.attempts.saturating_add(1);
        self.retry_at =
            Some(now + Duration::from_secs(15 * 2u64.pow(self.attempts.saturating_sub(1).min(3))));
    }
    pub fn recovered(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.snapshot.phase = "checking";
        self.snapshot.latency_ms = None;
        self.snapshot.consecutive_failures = 0;
        self.snapshot.recoveries = self.snapshot.recoveries.saturating_add(1);
    }
    pub fn recovery_failed(&mut self) {
        if self.active() {
            self.snapshot.phase = "unavailable";
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transient_failure_never_switches_and_success_resets_failure_streak() {
        let mut m = Monitor::default();
        m.begin("community", 100);
        let g = m.generation();
        m.record(g, None, 101);
        m.record(g, None, 102);
        assert!(!m.recovery_due(Instant::now()));
        m.record(g, Some(160), 103);
        assert_eq!(m.snapshot.consecutive_failures, 0);
        assert_eq!(m.snapshot.phase, "healthy");
    }
    #[test]
    fn recovery_uses_failure_threshold_and_exponential_cooldown() {
        let mut m = Monitor::default();
        m.begin("community", 100);
        for i in 0..3 {
            m.record(m.generation(), None, 101 + i);
        }
        let now = Instant::now();
        assert!(m.recovery_due(now));
        m.recovering(now);
        m.recovery_failed();
        assert!(!m.recovery_due(now + Duration::from_secs(14)));
        assert!(m.recovery_due(now + Duration::from_secs(15)));
        m.recovering(now + Duration::from_secs(15));
        assert!(!m.recovery_due(now + Duration::from_secs(44)));
        assert!(m.recovery_due(now + Duration::from_secs(45)));
    }
    #[test]
    fn stopped_or_replaced_sessions_ignore_in_flight_checks() {
        let mut m = Monitor::default();
        m.begin("community", 100);
        let old = m.generation();
        m.stop();
        m.record(old, Some(1), 101);
        assert_eq!(m.snapshot.phase, "idle");
        assert!(m.snapshot.samples.is_empty());
        m.begin("personal", 102);
        m.record(old, None, 103);
        assert_eq!(m.snapshot.phase, "checking");
        for i in 0..30 {
            m.record(m.generation(), None, 104 + i);
        }
        assert_eq!(m.snapshot.samples.len(), 24);
        assert!(!m.recovery_due(Instant::now()));
    }
    #[test]
    fn successful_recovery_waits_for_a_new_real_measurement() {
        let mut m = Monitor::default();
        m.begin("community", 100);
        let old = m.generation();
        m.recovered();
        m.record(old, Some(10), 101);
        assert_eq!(m.snapshot.phase, "checking");
        assert_eq!(m.snapshot.latency_ms, None);
        assert_eq!(m.snapshot.recoveries, 1);
        m.record(m.generation(), Some(150), 102);
        assert_eq!(m.snapshot.phase, "healthy");
    }
    #[test]
    fn recovery_notifications_cannot_revive_a_stopped_session() {
        let mut m = Monitor::default();
        m.begin("community", 100);
        m.stop();
        m.recovering(Instant::now());
        m.recovery_failed();
        assert_eq!(m.snapshot.phase, "idle");
        assert!(!m.recovery_due(Instant::now()));
    }
}
