//! Translator Module Integration Tests
//!
//! These tests verify the integration between factory, multi, service, common, batch,
//! and LLM submodules (pool, routing) components. They focus on component integration
//! integrity without making actual API calls to external translation services.

pub mod factory_tests;
pub mod batch_tests;
pub mod multi_tests;
pub mod service_tests;
pub mod common_tests;
pub mod llm_pool_tests;
pub mod llm_routing_tests;
pub mod integration_flow_tests;
pub mod source_lang_tests;

