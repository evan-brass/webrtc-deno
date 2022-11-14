use crate::error::Result;
use crate::{Interceptor, InterceptorBuilder};
use std::sync::Arc;

pub type MockBuilderResult = Result<Arc<dyn Interceptor>>;

/// MockBuilder is a mock Builder for testing.
pub struct MockBuilder {
    pub build: Box<dyn (Fn(&str) -> MockBuilderResult) + 'static>,
}

impl MockBuilder {
    pub fn new<F: (Fn(&str) -> MockBuilderResult) + 'static>(f: F) -> Self {
        MockBuilder { build: Box::new(f) }
    }
}

impl InterceptorBuilder for MockBuilder {
    fn build(&self, id: &str) -> MockBuilderResult {
        (self.build)(id)
    }
}
