use crate::Event;

/// A chain of event processors used for sub-pipelines (e.g., parsing captions).
/// This allows block-level plugins to apply inline transformations without
/// hardcoding dependencies on specific inline plugins.
pub struct Pipeline {
    processors: Vec<Box<dyn Fn(Vec<Event>) -> Vec<Event> + Send + Sync>>,
}

/// A pipeline whose document state is borrowed explicitly for every step.
///
/// This is the migration path for stateful plugins. It avoids shared mutable
/// state hidden inside processors while preserving the existing `Pipeline`
/// API for plugins that do not need a document context.
pub struct ContextPipeline<C> {
    processors: Vec<ContextProcessor<C>>,
    post_markdown: Vec<ContextProcessor<C>>,
}

/// A single pipeline step that borrows the shared document state for the
/// duration of one transformation.
type ContextProcessor<C> = Box<dyn Fn(&mut C, Vec<Event>) -> Vec<Event>>;

pub trait InlinePipeline<C> {
    fn run_with(&self, context: &mut C, events: Vec<Event>) -> Vec<Event>;

    /// Runs processors that must execute *after* markdown has expanded raw text
    /// patterns into AST nodes.
    ///
    /// Math is the canonical example: its `$...$` scanner must not consume a
    /// `$` that lives inside a link destination or title (e.g.
    /// `[text](/foo/$! "Title")`), which only becomes distinguishable once
    /// markdown has turned that syntax into actual attributes. The default is a
    /// no-op so that plain pipelines keep working unchanged.
    fn run_after_markdown(&self, _context: &mut C, events: Vec<Event>) -> Vec<Event> {
        events
    }
}

impl<C> InlinePipeline<C> for Pipeline {
    fn run_with(&self, _context: &mut C, events: Vec<Event>) -> Vec<Event> {
        self.run(events)
    }
}

impl<C> ContextPipeline<C> {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
            post_markdown: Vec::new(),
        }
    }

    pub fn add<F>(&mut self, processor: F)
    where
        F: Fn(&mut C, Vec<Event>) -> Vec<Event> + 'static,
    {
        self.processors.push(Box::new(processor));
    }

    /// Registers a processor that runs after markdown within nested inline
    /// contexts (captions, table cells, dialog, ...). See
    /// [`InlinePipeline::run_after_markdown`].
    pub fn add_after_markdown<F>(&mut self, processor: F)
    where
        F: Fn(&mut C, Vec<Event>) -> Vec<Event> + 'static,
    {
        self.post_markdown.push(Box::new(processor));
    }

    pub fn run(&self, context: &mut C, mut events: Vec<Event>) -> Vec<Event> {
        for processor in &self.processors {
            events = processor(context, events);
        }
        events
    }
}

impl<C> InlinePipeline<C> for ContextPipeline<C> {
    fn run_with(&self, context: &mut C, events: Vec<Event>) -> Vec<Event> {
        self.run(context, events)
    }

    fn run_after_markdown(&self, context: &mut C, mut events: Vec<Event>) -> Vec<Event> {
        for processor in &self.post_markdown {
            events = processor(context, events);
        }
        events
    }
}

impl<C> Default for ContextPipeline<C> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{ContextPipeline, InlinePipeline};
    use crate::Event;

    #[test]
    fn after_markdown_processors_run_separately_from_inline_processors() {
        let mut pipeline = ContextPipeline::new();
        pipeline.add(|log: &mut Vec<&'static str>, events| {
            log.push("inline");
            events
        });
        pipeline.add_after_markdown(|log: &mut Vec<&'static str>, events| {
            log.push("after");
            events
        });

        let mut log: Vec<&'static str> = Vec::new();
        let events = pipeline.run(&mut log, Vec::new());
        assert_eq!(log, vec!["inline"]);

        let _ = pipeline.run_after_markdown(&mut log, events);
        assert_eq!(log, vec!["inline", "after"]);
    }

    #[test]
    fn context_pipeline_borrows_one_document_state() {
        let mut pipeline = ContextPipeline::new();
        pipeline.add(|count: &mut usize, mut events: Vec<Event>| {
            *count += 1;
            events.push(Event::Text(count.to_string()));
            events
        });
        pipeline.add(|count: &mut usize, mut events: Vec<Event>| {
            *count += 1;
            events.push(Event::Text(count.to_string()));
            events
        });

        let mut count = 0;
        let output = pipeline.run(&mut count, Vec::new());

        assert_eq!(count, 2);
        assert_eq!(output.len(), 2);
    }
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
