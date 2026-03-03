//! System service registry and lifecycle management.
//!
//! The [`ServiceRegistry`] manages named services that implement the
//! [`SystemService`] trait, providing start/stop lifecycle and health
//! check aggregation.

use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::health::HealthStatus;

/// Type of system service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceType {
    /// Core kernel service (message bus, process table, etc.).
    Core,
    /// Plugin-provided service.
    Plugin,
    /// Cron/scheduler service.
    Cron,
    /// API/HTTP service.
    Api,
    /// Custom service with a user-defined label.
    Custom(String),
}

impl std::fmt::Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceType::Core => write!(f, "core"),
            ServiceType::Plugin => write!(f, "plugin"),
            ServiceType::Cron => write!(f, "cron"),
            ServiceType::Api => write!(f, "api"),
            ServiceType::Custom(s) => write!(f, "custom({s})"),
        }
    }
}

/// A system service managed by the kernel.
///
/// Services are started during boot and stopped during shutdown.
/// Each service provides a health check for monitoring.
#[async_trait]
pub trait SystemService: Send + Sync {
    /// Human-readable service name.
    fn name(&self) -> &str;

    /// Service type category.
    fn service_type(&self) -> ServiceType;

    /// Start the service.
    async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Stop the service.
    async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Perform a health check.
    async fn health_check(&self) -> HealthStatus;
}

/// Registry of system services with lifecycle management.
///
/// Uses [`DashMap`] for concurrent access from multiple kernel
/// subsystems.
pub struct ServiceRegistry {
    services: DashMap<String, Arc<dyn SystemService>>,
}

impl ServiceRegistry {
    /// Create a new, empty service registry.
    pub fn new() -> Self {
        Self {
            services: DashMap::new(),
        }
    }

    /// Register a service.
    ///
    /// Returns an error if a service with the same name is already
    /// registered.
    pub fn register(
        &self,
        service: Arc<dyn SystemService>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let name = service.name().to_owned();
        if self.services.contains_key(&name) {
            return Err(format!("service already registered: {name}").into());
        }
        info!(service = %name, "registering service");
        self.services.insert(name, service);
        Ok(())
    }

    /// Unregister a service by name.
    pub fn unregister(&self, name: &str) -> Option<Arc<dyn SystemService>> {
        self.services.remove(name).map(|(_, s)| s)
    }

    /// Get a service by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn SystemService>> {
        self.services.get(name).map(|s| s.value().clone())
    }

    /// List all registered services with their types.
    pub fn list(&self) -> Vec<(String, ServiceType)> {
        self.services
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().service_type()))
            .collect()
    }

    /// Start all registered services.
    ///
    /// Individual service failures are logged as warnings but do not
    /// prevent other services from starting.
    pub async fn start_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for entry in self.services.iter() {
            let name = entry.key().clone();
            info!(service = %name, "starting service");
            if let Err(e) = entry.value().start().await {
                warn!(service = %name, error = %e, "service failed to start");
            }
        }
        Ok(())
    }

    /// Stop all registered services.
    ///
    /// Individual service failures are logged as warnings.
    pub async fn stop_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for entry in self.services.iter() {
            let name = entry.key().clone();
            info!(service = %name, "stopping service");
            if let Err(e) = entry.value().stop().await {
                warn!(service = %name, error = %e, "service failed to stop");
            }
        }
        Ok(())
    }

    /// Return a snapshot of all services as a `Vec`.
    ///
    /// This copies all `(name, Arc<dyn SystemService>)` pairs out of
    /// the `DashMap`, so the returned collection owns no DashMap refs
    /// and is safe to hold across await points and send across threads.
    pub fn snapshot(&self) -> Vec<(String, Arc<dyn SystemService>)> {
        self.services
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Run health checks on all registered services.
    pub async fn health_all(&self) -> Vec<(String, HealthStatus)> {
        let mut results = Vec::new();
        for entry in self.services.iter() {
            let name = entry.key().clone();
            let status = entry.value().health_check().await;
            results.push((name, status));
        }
        results
    }

    /// Register a service and create a resource tree node + chain event.
    ///
    /// When the exochain feature is enabled and a tree manager is provided,
    /// creates a node at `/kernel/services/{name}` in the resource tree
    /// and appends a corresponding chain event via [`TreeManager`].
    #[cfg(feature = "exochain")]
    pub fn register_with_tree(
        &self,
        service: Arc<dyn SystemService>,
        tree_manager: &crate::tree_manager::TreeManager,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let name = service.name().to_owned();
        self.register(service)?;

        // Create tree node + chain event through the unified TreeManager path
        if let Err(e) = tree_manager.register_service(&name) {
            tracing::debug!(service = %name, error = %e, "failed to register service in tree");
        }

        Ok(())
    }

    /// Get the number of registered services.
    pub fn len(&self) -> usize {
        self.services.len()
    }

    /// Check whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.services.is_empty()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A mock service for testing.
    struct MockService {
        name: String,
        service_type: ServiceType,
    }

    impl MockService {
        fn new(name: &str, stype: ServiceType) -> Self {
            Self {
                name: name.to_owned(),
                service_type: stype,
            }
        }
    }

    #[async_trait]
    impl SystemService for MockService {
        fn name(&self) -> &str {
            &self.name
        }

        fn service_type(&self) -> ServiceType {
            self.service_type.clone()
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

    #[test]
    fn register_and_get() {
        let registry = ServiceRegistry::new();
        let svc = Arc::new(MockService::new("test-svc", ServiceType::Core));
        registry.register(svc).unwrap();

        let retrieved = registry.get("test-svc");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "test-svc");
    }

    #[test]
    fn register_duplicate_fails() {
        let registry = ServiceRegistry::new();
        let svc1 = Arc::new(MockService::new("dup-svc", ServiceType::Core));
        let svc2 = Arc::new(MockService::new("dup-svc", ServiceType::Plugin));

        registry.register(svc1).unwrap();
        let result = registry.register(svc2);
        assert!(result.is_err());
    }

    #[test]
    fn unregister() {
        let registry = ServiceRegistry::new();
        let svc = Arc::new(MockService::new("rm-svc", ServiceType::Core));
        registry.register(svc).unwrap();

        let removed = registry.unregister("rm-svc");
        assert!(removed.is_some());
        assert!(registry.get("rm-svc").is_none());
    }

    #[test]
    fn list_services() {
        let registry = ServiceRegistry::new();
        registry
            .register(Arc::new(MockService::new("svc-a", ServiceType::Core)))
            .unwrap();
        registry
            .register(Arc::new(MockService::new("svc-b", ServiceType::Cron)))
            .unwrap();

        let list = registry.list();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn start_and_stop_all() {
        let registry = ServiceRegistry::new();
        registry
            .register(Arc::new(MockService::new("svc-1", ServiceType::Core)))
            .unwrap();
        registry
            .register(Arc::new(MockService::new("svc-2", ServiceType::Plugin)))
            .unwrap();

        registry.start_all().await.unwrap();
        registry.stop_all().await.unwrap();
    }

    #[tokio::test]
    async fn health_all() {
        let registry = ServiceRegistry::new();
        registry
            .register(Arc::new(MockService::new("svc-1", ServiceType::Core)))
            .unwrap();
        registry
            .register(Arc::new(MockService::new("svc-2", ServiceType::Plugin)))
            .unwrap();

        let health = registry.health_all().await;
        assert_eq!(health.len(), 2);
        for (_, status) in &health {
            assert_eq!(*status, HealthStatus::Healthy);
        }
    }

    #[test]
    fn len_and_is_empty() {
        let registry = ServiceRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry
            .register(Arc::new(MockService::new("svc", ServiceType::Core)))
            .unwrap();
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn service_type_display() {
        assert_eq!(ServiceType::Core.to_string(), "core");
        assert_eq!(ServiceType::Plugin.to_string(), "plugin");
        assert_eq!(ServiceType::Cron.to_string(), "cron");
        assert_eq!(ServiceType::Api.to_string(), "api");
        assert_eq!(
            ServiceType::Custom("webhook".into()).to_string(),
            "custom(webhook)"
        );
    }
}
