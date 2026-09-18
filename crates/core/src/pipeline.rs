use crate::Event;

/// A chain of event processors used for sub-pipelines (e.g., parsing captions).
/// This allows block-level plugins to apply inline transformations without
/// hardcoding dependencies on specific inline plugins.
pub struct Pipeline {
    processors: Vec<Box<dyn Fn(Vec<Event>) -> Vec<Event> + Send + Sync>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    /// Registers a new processor into the chain.
    pub fn add<F>(&mut self, processor: F)
    where
        F: Fn(Vec<Event>) -> Vec<Event> + Send + Sync + 'static,
    {
        self.processors.push(Box::new(processor));
    }

    /// Runs the event stream through all registered processors sequentially.
    pub fn run(&self, mut events: Vec<Event>) -> Vec<Event> {
        for processor in &self.processors {
            events = processor(events);
        }
        events
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}
