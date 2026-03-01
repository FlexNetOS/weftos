//! Health monitoring subsystem.
//!
//! The [`HealthSystem`] aggregates health checks from all registered
//! services and produces an overall [`OverallHealth`] status.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::service::ServiceRegistry;

/// Health status for a single service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Service is operating normally.
    Healthy,
    /// Service is operational but degraded.
    Degraded(String),
    /// Service is not operational.
    Unhealthy(String),
    /// Health status could not be determined (e.g. timeout).
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded(msg) => write!(f, "degraded: {msg}"),
            HealthStatus::Unhealthy(msg) => write!(f, "unhealthy: {msg}"),
            HealthStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Aggregated health status for the entire kernel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverallHealth {
    /// All services are healthy.
    Healthy,
    /// Some services are degraded or unhealthy.
    Degraded {
        /// Services that are not fully healthy.
        unhealthy_services: Vec<String>,
    },
    /// All services are unhealthy or no services registered.
    Down,
}

impl std::fmt::Display for OverallHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OverallHealth::Healthy => write!(f, "healthy"),
            OverallHealth::Degraded {
                unhealthy_services,
            } => {
                write!(f, "degraded ({})", unhealthy_services.join(", "))
            }
            OverallHealth::Down => write!(f, "down"),
        }
    }
}

/// Health monitoring system.
///
/// Periodically checks all registered services and aggregates their
/// health into an overall status.
pub struct HealthSystem {
    check_interval_secs: u64,
}

impl HealthSystem {
    /// Create a new health system with the given check interval.
    pub fn new(check_interval_secs: u64) -> Self {
        Self {
            check_interval_secs,
        }
    }

    /// Get the configured check interval in seconds.
    pub fn check_interval_secs(&self) -> u64 {
        self.check_interval_secs
    }

    /// Run a single health check cycle against all services.
    pub async fn aggregate(
        &self,
        registry: &Arc<ServiceRegistry>,
    ) -> (OverallHealth, Vec<(String, HealthStatus)>) {
        let results = registry.health_all().await;

        if results.is_empty() {
            return (OverallHealth::Down, results);
        }

        let mut unhealthy = Vec::new();
        let mut all_unhealthy = true;

        for (name, status) in &results {
            match status {
                HealthStatus::Healthy => {
                    debug!(service = %name, "health check: healthy");
                    all_unhealthy = false;
                }
                HealthStatus::Degraded(msg) => {
                    warn!(service = %name, reason = %msg, "health check: degraded");
                    unhealthy.push(name.clone());
                    all_unhealthy = false;
                }
                HealthStatus::Unhealthy(msg) => {
                    warn!(service = %name, reason = %msg, "health check: unhealthy");
                    unhealthy.push(name.clone());
                }
                HealthStatus::Unknown => {
                    warn!(service = %name, "health check: unknown");
                    unhealthy.push(name.clone());
                }
            }
        }

        let overall = if unhealthy.is_empty() {
            OverallHealth::Healthy
        } else if all_unhealthy {
            OverallHealth::Down
        } else {
            OverallHealth::Degraded {
                unhealthy_services: unhealthy,
            }
        };

        (overall, results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::{ServiceType, SystemService};
    use async_trait::async_trait;

    struct HealthyService;

    #[async_trait]
    impl SystemService for HealthyService {
        fn name(&self) -> &str {
            "healthy-svc"
        }
        fn service_type(&self) -> ServiceType {
            ServiceType::Core
        }
        async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        async fn health_check(&self) -> HealthStatus {
            HealthStatus::Healthy
        }
    }

    struct UnhealthyService;

    #[async_trait]
    impl SystemService for UnhealthyService {
        fn name(&self) -> &str {
            "unhealthy-svc"
        }
        fn service_type(&self) -> ServiceType {
            ServiceType::Core
        }
        async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        async fn health_check(&self) -> HealthStatus {
            HealthStatus::Unhealthy("test failure".into())
        }
    }

    #[tokio::test]
    async fn aggregate_all_healthy() {
        let registry = Arc::new(ServiceRegistry::new());
        registry.register(Arc::new(HealthyService)).unwrap();

        let health = HealthSystem::new(30);
        let (overall, results) = health.aggregate(&registry).await;

        assert!(matches!(overall, OverallHealth::Healthy));
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn aggregate_mixed() {
        let registry = Arc::new(ServiceRegistry::new());
        registry.register(Arc::new(HealthyService)).unwrap();
        registry.register(Arc::new(UnhealthyService)).unwrap();

        let health = HealthSystem::new(30);
        let (overall, results) = health.aggregate(&registry).await;

        assert!(matches!(overall, OverallHealth::Degraded { .. }));
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn aggregate_all_unhealthy() {
        let registry = Arc::new(ServiceRegistry::new());
        registry.register(Arc::new(UnhealthyService)).unwrap();

        let health = HealthSystem::new(30);
        let (overall, _) = health.aggregate(&registry).await;

        assert!(matches!(overall, OverallHealth::Down));
    }

    #[tokio::test]
    async fn aggregate_empty_registry() {
        let registry = Arc::new(ServiceRegistry::new());
        let health = HealthSystem::new(30);
        let (overall, results) = health.aggregate(&registry).await;

        assert!(matches!(overall, OverallHealth::Down));
        assert!(results.is_empty());
    }

    #[test]
    fn health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(
            HealthStatus::Degraded("slow".into()).to_string(),
            "degraded: slow"
        );
        assert_eq!(
            HealthStatus::Unhealthy("crash".into()).to_string(),
            "unhealthy: crash"
        );
        assert_eq!(HealthStatus::Unknown.to_string(), "unknown");
    }

    #[test]
    fn overall_health_display() {
        assert_eq!(OverallHealth::Healthy.to_string(), "healthy");
        assert_eq!(OverallHealth::Down.to_string(), "down");
        assert_eq!(
            OverallHealth::Degraded {
                unhealthy_services: vec!["svc-a".into(), "svc-b".into()]
            }
            .to_string(),
            "degraded (svc-a, svc-b)"
        );
    }

    #[test]
    fn check_interval() {
        let health = HealthSystem::new(15);
        assert_eq!(health.check_interval_secs(), 15);
    }
}
