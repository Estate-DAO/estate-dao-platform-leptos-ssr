//! Provider Registry - manages provider registration and access

use std::sync::Arc;

use crate::composite::{CompositeHotelProvider, CompositePlaceProvider, FallbackStrategy};
use crate::ports::{HotelProviderPort, PlaceProviderPort};

/// Provider registry that holds configured providers
pub struct ProviderRegistry {
    hotel_provider: Arc<dyn HotelProviderPort>,
    place_provider: Arc<dyn PlaceProviderPort>,
}

impl ProviderRegistry {
    /// Create a new registry builder
    pub fn builder() -> ProviderRegistryBuilder {
        ProviderRegistryBuilder::new()
    }

    /// Get the hotel provider
    pub fn hotel_provider(&self) -> Arc<dyn HotelProviderPort> {
        self.hotel_provider.clone()
    }

    /// Get the place provider
    pub fn place_provider(&self) -> Arc<dyn PlaceProviderPort> {
        self.place_provider.clone()
    }
}

impl std::fmt::Debug for ProviderRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderRegistry")
            .field("hotel_provider", &self.hotel_provider.name())
            .field("place_provider", &self.place_provider.name())
            .finish()
    }
}

/// Builder for constructing a ProviderRegistry
pub struct ProviderRegistryBuilder {
    hotel_providers: Vec<Arc<dyn HotelProviderPort>>,
    place_providers: Vec<Arc<dyn PlaceProviderPort>>,
    fallback_strategy: FallbackStrategy,
}

impl ProviderRegistryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            hotel_providers: Vec::new(),
            place_providers: Vec::new(),
            fallback_strategy: FallbackStrategy::default(),
        }
    }

    /// Add a hotel provider
    pub fn with_hotel_provider<P>(mut self, provider: P) -> Self
    where
        P: HotelProviderPort + 'static,
    {
        self.hotel_providers.push(Arc::new(provider));
        self
    }

    /// Add a hotel provider (already wrapped in Arc)
    pub fn with_hotel_provider_arc(mut self, provider: Arc<dyn HotelProviderPort>) -> Self {
        self.hotel_providers.push(provider);
        self
    }

    /// Add a place provider
    pub fn with_place_provider<P>(mut self, provider: P) -> Self
    where
        P: PlaceProviderPort + 'static,
    {
        self.place_providers.push(Arc::new(provider));
        self
    }

    /// Add a place provider (already wrapped in Arc)
    pub fn with_place_provider_arc(mut self, provider: Arc<dyn PlaceProviderPort>) -> Self {
        self.place_providers.push(provider);
        self
    }

    /// Set the fallback strategy
    pub fn with_fallback_strategy(mut self, strategy: FallbackStrategy) -> Self {
        self.fallback_strategy = strategy;
        self
    }

    /// Build the registry
    ///
    /// # Panics
    /// Panics if no hotel or place providers have been configured
    pub fn build(self) -> ProviderRegistry {
        if self.hotel_providers.is_empty() {
            panic!("At least one hotel provider must be configured");
        }
        if self.place_providers.is_empty() {
            panic!("At least one place provider must be configured");
        }

        let hotel_provider: Arc<dyn HotelProviderPort> = if self.hotel_providers.len() == 1 {
            // Single provider - no need for composite
            self.hotel_providers.into_iter().next().unwrap()
        } else {
            // Multiple providers - use composite with fallback
            Arc::new(CompositeHotelProvider::with_strategy(
                self.hotel_providers,
                self.fallback_strategy.clone(),
            ))
        };

        let place_provider: Arc<dyn PlaceProviderPort> = if self.place_providers.len() == 1 {
            // Single provider - no need for composite
            self.place_providers.into_iter().next().unwrap()
        } else {
            // Multiple providers - use composite with fallback
            Arc::new(CompositePlaceProvider::new(self.place_providers))
        };

        ProviderRegistry {
            hotel_provider,
            place_provider,
        }
    }

    /// Build the registry, returning an error if configuration is invalid
    pub fn try_build(self) -> Result<ProviderRegistry, &'static str> {
        if self.hotel_providers.is_empty() {
            return Err("At least one hotel provider must be configured");
        }
        if self.place_providers.is_empty() {
            return Err("At least one place provider must be configured");
        }

        Ok(self.build())
    }
}

impl Default for ProviderRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
