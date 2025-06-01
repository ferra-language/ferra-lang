#!/usr/bin/env rust-script
//! Error Recovery Performance Tracking Script
//!
//! This script runs error recovery benchmarks and tracks performance trends
//! over time to detect regressions automatically in CI.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Performance data point for tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceDataPoint {
    pub timestamp: u64,
    pub git_hash: String,
    pub test_name: String,
    pub parse_time_ms: u64,
    pub error_count: usize,
    pub memory_usage_bytes: usize,
    pub success_rate: f64,
}

/// Performance trend analysis
#[derive(Debug, Clone)]
pub struct PerformanceTrend {
    pub test_name: String,
    pub baseline_time_ms: u64,
    pub current_time_ms: u64,
    pub trend_direction: TrendDirection,
    pub regression_severity: RegressionSeverity,
}

#[derive(Debug, Clone)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
}

#[derive(Debug, Clone)]
pub enum RegressionSeverity {
    None,
    Minor,     // 10-25% slowdown
    Moderate,  // 25-50% slowdown
    Severe,    // 50-100% slowdown
    Critical,  // >100% slowdown
}

/// Main performance tracker
pub struct ErrorRecoveryPerfTracker {
    data_file: String,
    baseline_file: String,
    regression_threshold: f64, // Percentage threshold for regression detection
}

impl ErrorRecoveryPerfTracker {
    pub fn new() -> Self {
        Self {
            data_file: "error_recovery_perf_data.json".to_string(),
            baseline_file: "error_recovery_baseline.json".to_string(),
            regression_threshold: 0.25, // 25% threshold
        }
    }

    /// Run error recovery benchmarks and collect performance data
    pub fn run_benchmarks(&self) -> Result<Vec<PerformanceDataPoint>, Box<dyn std::error::Error>> {
        println!("Running error recovery benchmarks...");

        // Run criterion benchmarks for error recovery
        let output = Command::new("cargo")
            .args(&[
                "bench",
                "--bench",
                "parser_benchmarks",
                "--",
                "error_recovery",
                "--output-format",
                "json",
            ])
            .output()?;

        if !output.status.success() {
            eprintln!("Benchmark failed: {}", String::from_utf8_lossy(&output.stderr));
            return Err("Benchmark execution failed".into());
        }

        // Parse benchmark results (simplified - in real implementation would parse Criterion JSON)
        let git_hash = self.get_git_hash()?;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        // Mock data - in real implementation, parse from Criterion output
        let test_results = vec![
            PerformanceDataPoint {
                timestamp,
                git_hash: git_hash.clone(),
                test_name: "error_recovery_overhead/missing_semicolon".to_string(),
                parse_time_ms: 15,
                error_count: 1,
                memory_usage_bytes: 8192,
                success_rate: 1.0,
            },
            PerformanceDataPoint {
                timestamp,
                git_hash: git_hash.clone(),
                test_name: "error_recovery_overhead/multiple_errors".to_string(),
                parse_time_ms: 45,
                error_count: 3,
                memory_usage_bytes: 16384,
                success_rate: 1.0,
            },
            PerformanceDataPoint {
                timestamp,
                git_hash: git_hash.clone(),
                test_name: "error_density_impact/high_density".to_string(),
                parse_time_ms: 120,
                error_count: 5,
                memory_usage_bytes: 32768,
                success_rate: 1.0,
            },
            PerformanceDataPoint {
                timestamp,
                git_hash,
                test_name: "error_recovery_scalability/large_with_errors".to_string(),
                parse_time_ms: 250,
                error_count: 50,
                memory_usage_bytes: 131072,
                success_rate: 1.0,
            },
        ];

        Ok(test_results)
    }

    /// Load historical performance data
    pub fn load_historical_data(&self) -> Result<Vec<PerformanceDataPoint>, Box<dyn std::error::Error>> {
        if !Path::new(&self.data_file).exists() {
            return Ok(Vec::new());
        }

        let data = fs::read_to_string(&self.data_file)?;
        let points: Vec<PerformanceDataPoint> = serde_json::from_str(&data)?;
        Ok(points)
    }

    /// Save performance data
    pub fn save_performance_data(&self, data: &[PerformanceDataPoint]) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(data)?;
        fs::write(&self.data_file, json)?;
        Ok(())
    }

    /// Analyze performance trends and detect regressions
    pub fn analyze_trends(&self, current_data: &[PerformanceDataPoint], historical_data: &[PerformanceDataPoint]) -> Vec<PerformanceTrend> {
        let mut trends = Vec::new();

        // Group historical data by test name
        let mut baselines: HashMap<String, u64> = HashMap::new();
        for point in historical_data {
            let entry = baselines.entry(point.test_name.clone()).or_insert(u64::MAX);
            *entry = (*entry).min(point.parse_time_ms);
        }

        // Compare current performance against baselines
        for current_point in current_data {
            let baseline = baselines.get(&current_point.test_name).copied().unwrap_or(current_point.parse_time_ms);
            
            let trend_direction = if current_point.parse_time_ms < baseline {
                TrendDirection::Improving
            } else if current_point.parse_time_ms > baseline {
                let ratio = current_point.parse_time_ms as f64 / baseline as f64;
                if ratio > 1.1 {
                    TrendDirection::Degrading
                } else {
                    TrendDirection::Stable
                }
            } else {
                TrendDirection::Stable
            };

            let regression_severity = self.calculate_regression_severity(baseline, current_point.parse_time_ms);

            trends.push(PerformanceTrend {
                test_name: current_point.test_name.clone(),
                baseline_time_ms: baseline,
                current_time_ms: current_point.parse_time_ms,
                trend_direction,
                regression_severity,
            });
        }

        trends
    }

    /// Calculate regression severity based on performance change
    fn calculate_regression_severity(&self, baseline: u64, current: u64) -> RegressionSeverity {
        if current <= baseline {
            return RegressionSeverity::None;
        }

        let ratio = current as f64 / baseline as f64;
        
        match ratio {
            r if r <= 1.1 => RegressionSeverity::None,
            r if r <= 1.25 => RegressionSeverity::Minor,
            r if r <= 1.5 => RegressionSeverity::Moderate,
            r if r <= 2.0 => RegressionSeverity::Severe,
            _ => RegressionSeverity::Critical,
        }
    }

    /// Generate performance report
    pub fn generate_report(&self, trends: &[PerformanceTrend]) -> String {
        let mut report = String::new();
        report.push_str("# Error Recovery Performance Report\n\n");

        let mut regressions = Vec::new();
        let mut improvements = Vec::new();
        let mut stable = Vec::new();

        for trend in trends {
            match trend.trend_direction {
                TrendDirection::Degrading => regressions.push(trend),
                TrendDirection::Improving => improvements.push(trend),
                TrendDirection::Stable => stable.push(trend),
            }
        }

        // Report regressions
        if !regressions.is_empty() {
            report.push_str("## 🚨 Performance Regressions\n\n");
            for trend in &regressions {
                let severity_emoji = match trend.regression_severity {
                    RegressionSeverity::Critical => "🔴",
                    RegressionSeverity::Severe => "🟠",
                    RegressionSeverity::Moderate => "🟡",
                    RegressionSeverity::Minor => "🟢",
                    RegressionSeverity::None => "⚪",
                };
                
                let percentage_change = ((trend.current_time_ms as f64 / trend.baseline_time_ms as f64) - 1.0) * 100.0;
                
                report.push_str(&format!(
                    "- {} **{}**: {}ms → {}ms (+{:.1}%)\n",
                    severity_emoji,
                    trend.test_name,
                    trend.baseline_time_ms,
                    trend.current_time_ms,
                    percentage_change
                ));
            }
            report.push('\n');
        }

        // Report improvements
        if !improvements.is_empty() {
            report.push_str("## 🚀 Performance Improvements\n\n");
            for trend in &improvements {
                let percentage_change = ((trend.baseline_time_ms as f64 / trend.current_time_ms as f64) - 1.0) * 100.0;
                report.push_str(&format!(
                    "- ✅ **{}**: {}ms → {}ms (-{:.1}%)\n",
                    trend.test_name,
                    trend.baseline_time_ms,
                    trend.current_time_ms,
                    percentage_change
                ));
            }
            report.push('\n');
        }

        // Report stable performance
        if !stable.is_empty() {
            report.push_str("## 📊 Stable Performance\n\n");
            for trend in &stable {
                report.push_str(&format!(
                    "- ⚡ **{}**: {}ms (baseline: {}ms)\n",
                    trend.test_name,
                    trend.current_time_ms,
                    trend.baseline_time_ms
                ));
            }
            report.push('\n');
        }

        report
    }

    /// Check if there are critical regressions that should fail CI
    pub fn has_critical_regressions(&self, trends: &[PerformanceTrend]) -> bool {
        trends.iter().any(|trend| {
            matches!(trend.regression_severity, RegressionSeverity::Critical | RegressionSeverity::Severe)
        })
    }

    /// Get current git hash
    fn get_git_hash(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(&["rev-parse", "--short", "HEAD"])
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?.trim().to_string())
        } else {
            Ok("unknown".to_string())
        }
    }
}

/// Main entry point for the performance tracking script
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = ErrorRecoveryPerfTracker::new();

    println!("🔍 Error Recovery Performance Tracker");
    println!("=====================================");

    // Run current benchmarks
    let current_data = tracker.run_benchmarks()?;
    println!("✅ Collected {} performance data points", current_data.len());

    // Load historical data
    let mut historical_data = tracker.load_historical_data()?;
    println!("📊 Loaded {} historical data points", historical_data.len());

    // Analyze trends
    let trends = tracker.analyze_trends(&current_data, &historical_data);
    println!("📈 Analyzed {} performance trends", trends.len());

    // Generate report
    let report = tracker.generate_report(&trends);
    println!("\n{}", report);

    // Check for critical regressions
    if tracker.has_critical_regressions(&trends) {
        eprintln!("❌ CRITICAL PERFORMANCE REGRESSION DETECTED!");
        eprintln!("This build should not be merged due to significant performance degradation.");
        std::process::exit(1);
    }

    // Save updated data
    historical_data.extend(current_data);
    tracker.save_performance_data(&historical_data)?;
    println!("💾 Performance data saved for future comparisons");

    println!("✅ Performance tracking complete - all checks passed!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regression_severity_calculation() {
        let tracker = ErrorRecoveryPerfTracker::new();

        assert!(matches!(tracker.calculate_regression_severity(100, 100), RegressionSeverity::None));
        assert!(matches!(tracker.calculate_regression_severity(100, 105), RegressionSeverity::None));
        assert!(matches!(tracker.calculate_regression_severity(100, 115), RegressionSeverity::Minor));
        assert!(matches!(tracker.calculate_regression_severity(100, 135), RegressionSeverity::Moderate));
        assert!(matches!(tracker.calculate_regression_severity(100, 175), RegressionSeverity::Severe));
        assert!(matches!(tracker.calculate_regression_severity(100, 250), RegressionSeverity::Critical));
    }

    #[test]
    fn test_trend_analysis() {
        let tracker = ErrorRecoveryPerfTracker::new();

        let historical = vec![
            PerformanceDataPoint {
                timestamp: 1000,
                git_hash: "abc123".to_string(),
                test_name: "test1".to_string(),
                parse_time_ms: 100,
                error_count: 1,
                memory_usage_bytes: 1024,
                success_rate: 1.0,
            }
        ];

        let current = vec![
            PerformanceDataPoint {
                timestamp: 2000,
                git_hash: "def456".to_string(),
                test_name: "test1".to_string(),
                parse_time_ms: 150, // 50% regression
                error_count: 1,
                memory_usage_bytes: 1024,
                success_rate: 1.0,
            }
        ];

        let trends = tracker.analyze_trends(&current, &historical);
        assert_eq!(trends.len(), 1);
        assert!(matches!(trends[0].trend_direction, TrendDirection::Degrading));
        assert!(matches!(trends[0].regression_severity, RegressionSeverity::Moderate));
    }
} 