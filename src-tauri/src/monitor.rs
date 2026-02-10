use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use pinger::{ping, PingResult, PingOptions};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::task::AbortHandle;
use std::fs::OpenOptions;
use std::io::Write;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisplayRule {
    pub id: Uuid,
    pub condition: String,
    pub threshold: f64,
    pub label: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HostConfig {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub command: String,
    pub display_rules: Vec<DisplayRule>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HostPreset {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PingStats {
    pub host_id: Uuid,
    pub current: f64,
    pub mean: f64,
    pub std_dev: f64, // Jitter
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub total_pings: usize,
    pub successful_pings: usize,
    pub failed_pings: usize,
    pub packet_loss_rate: f64,
    pub success_rate: f64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub peaks_count: usize,
    pub peaks_per_minute: f64,
    pub peaks_mean: f64,
    pub peaks_max: f64,
    pub last_peak: Option<DateTime<Utc>>,
    pub status: String,
    pub labels: Vec<String>,
    pub start_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PingData {
    pub timestamp: DateTime<Utc>,
    pub latency: f64,
    pub is_peak: bool,
    pub success: bool,
}

pub struct Monitor {
    pub host_id: Uuid,
    pub target: String,
    pub history: Arc<Mutex<VecDeque<PingData>>>,
    pub peak_threshold: f64,
    pub stats: Arc<Mutex<PingStats>>,
    pub tx: broadcast::Sender<PingStats>,
    pub log_path: String,
    pub display_rules: Arc<Mutex<Vec<DisplayRule>>>,
    pub ping_interval: Duration,
    pub abort_handles: Mutex<Vec<AbortHandle>>,
}

impl Monitor {
    pub fn new(host_id: Uuid, target: &str, peak_threshold: f64, log_path: &str, rules: Vec<DisplayRule>, ping_interval: u64) -> (Arc<Self>, broadcast::Receiver<PingStats>) {
        let (tx, rx) = broadcast::channel(100);
        let monitor = Arc::new(Self {
            host_id,
            target: target.to_string(),
            history: Arc::new(Mutex::new(VecDeque::with_capacity(3600))),
            peak_threshold,
            stats: Arc::new(Mutex::new(PingStats {
                host_id,
                current: 0.0,
                mean: 0.0,
                std_dev: 0.0,
                median: 0.0,
                min: 0.0,
                max: 0.0,
                total_pings: 0,
                successful_pings: 0,
                failed_pings: 0,
                packet_loss_rate: 0.0,
                success_rate: 0.0,
                bytes_sent: 0,
                bytes_received: 0,
                peaks_count: 0,
                peaks_per_minute: 0.0,
                peaks_mean: 0.0,
                peaks_max: 0.0,
                last_peak: None,
                status: "Initializing".to_string(),
                labels: vec![],
                start_time: Utc::now(),
            })),
            tx,
            log_path: log_path.to_string(),
            display_rules: Arc::new(Mutex::new(rules)),
            ping_interval: Duration::from_secs(ping_interval),
            abort_handles: Mutex::new(Vec::new()),
        });
        (monitor, rx)
    }

    pub fn add_abort_handle(&self, handle: AbortHandle) {
        self.abort_handles.lock().unwrap().push(handle);
    }

    fn update_stats(&self, now: DateTime<Utc>, latency: f64, success: bool, is_peak: bool) {
        let mut h = self.history.lock().unwrap();
        
        h.push_back(PingData {
            timestamp: now,
            latency,
            is_peak,
            success,
        });

        if h.len() > 3600 {
            h.pop_front();
        }

        let total_pings = h.len();
        let successful_pings = h.iter().filter(|d| d.success).count();
        let failed_pings = total_pings - successful_pings;
        let success_rate = if total_pings > 0 { (successful_pings as f64 / total_pings as f64) * 100.0 } else { 0.0 };
        let packet_loss_rate = if total_pings > 0 { (failed_pings as f64 / total_pings as f64) * 100.0 } else { 0.0 };
        let bytes_sent = total_pings as u64 * 64;
        let bytes_received = successful_pings as u64 * 64;

        let success_latencies: Vec<f64> = h.iter().filter(|d| d.success).map(|d| d.latency).collect();
        let (mean, std_dev, median, min, max) = if !success_latencies.is_empty() {
            let sum: f64 = success_latencies.iter().sum();
            let avg = sum / success_latencies.len() as f64;
            let mut sorted = success_latencies.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let med = sorted[sorted.len() / 2];
            let mn = sorted.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let mx = sorted.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let variance = success_latencies.iter().map(|value| {
                let diff = avg - (*value as f64);
                diff * diff
            }).sum::<f64>() / success_latencies.len() as f64;
            let std_dev = variance.sqrt();

            (avg, std_dev, med, mn, mx)
        } else {
            (0.0, 0.0, 0.0, 0.0, 0.0)
        };

        let peaks: Vec<&PingData> = h.iter().filter(|d| d.is_peak).collect();
        let peaks_in_last_minute = h.iter()
            .filter(|d| d.is_peak && (now - d.timestamp).num_seconds() < 60)
            .count();
        
        let peaks_sum: f64 = peaks.iter().map(|d| d.latency).sum();
        let peaks_mean = if !peaks.is_empty() { peaks_sum / peaks.len() as f64 } else { 0.0 };
        let peaks_max = peaks.iter().map(|d| d.latency).fold(0.0, f64::max);

        let status = match peaks_in_last_minute {
            0..=2 => "Good",
            3..=5 => "Moderate",
            6..=10 => "Bad",
            _ => "Unusable",
        };

        // Evaluate Display Rules
        let rules = self.display_rules.lock().unwrap();
        let labels: Vec<String> = rules.iter()
            .filter(|r| r.enabled)
            .filter(|r| {
                if r.condition == "less" { latency < r.threshold } else { latency > r.threshold }
            })
            .map(|r| r.label.clone())
            .collect();

        let mut s = self.stats.lock().unwrap();
        *s = PingStats {
            host_id: self.host_id,
            current: if success { latency } else { 0.0 },
            mean,
            std_dev,
            median,
            min,
            max,
            total_pings,
            successful_pings,
            failed_pings,
            packet_loss_rate,
            success_rate,
            bytes_sent,
            bytes_received,
            peaks_count: peaks.len(),
            peaks_per_minute: peaks_in_last_minute as f64,
            peaks_mean,
            peaks_max,
            last_peak: if is_peak { Some(now) } else { s.last_peak },
            status: status.to_string(),
            labels,
            start_time: s.start_time,
        };

        let _ = self.tx.send(s.clone());
    }

    pub async fn start(self: Arc<Self>) -> anyhow::Result<()> {
        // Timeout fixed at 2s, interval controlled by loop sleep
        let stream = ping(PingOptions::new(self.target.clone(), Duration::from_secs(2), None))?;
        let self_clone = self.clone();

        if !std::path::Path::new(&self.log_path).exists() {
            let mut file = OpenOptions::new().create(true).write(true).open(&self.log_path)?;
            writeln!(file, "Timestamp,Latency,IsPeak,Success")?;
        }



        let task = tokio::spawn(async move {
            for result in stream {
                // Force yield to prevent starvation
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                
                let now = Utc::now();
                match result {
                    PingResult::Pong(duration, _) => {
                        let latency = duration.as_secs_f64() * 1000.0;
                        let median = {
                            let h = self_clone.history.lock().unwrap();
                            let mut latencies: Vec<f64> = h.iter().take(60).filter(|d| d.success).map(|d| d.latency).collect();
                            latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
                            if latencies.is_empty() { latency } else { latencies[latencies.len() / 2] }
                        };
                        let is_peak = latency > (median + self_clone.peak_threshold);
                        self_clone.update_stats(now, latency, true, is_peak);
                        if let Ok(mut file) = OpenOptions::new().append(true).open(&self_clone.log_path) {
                            let _ = writeln!(file, "{},{},{},true", now.to_rfc3339(), latency, is_peak);
                        }
                    }
                    PingResult::Timeout(_) => {
                        self_clone.update_stats(now, 2000.0, false, true);
                        if let Ok(mut file) = OpenOptions::new().append(true).open(&self_clone.log_path) {
                            let _ = writeln!(file, "{},2000.0,true,false", now.to_rfc3339());
                        }
                    }
                    _ => {}
                }
                
                // Control interval here
                tokio::time::sleep(self_clone.ping_interval).await;
            }
        });

        // Store the abort handle
        self.abort_handles.lock().unwrap().push(task.abort_handle());

        Ok(())
    }

    pub fn stop(&self) {
        let mut handles = self.abort_handles.lock().unwrap();
        for handle in handles.drain(..) {
            handle.abort();
        }
        println!("[Rust] Monitor stopped for {} (killed {} tasks)", self.host_id, handles.len());
    }
}
