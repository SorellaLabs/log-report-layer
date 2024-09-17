use tracing::{Level, Subscriber};
use tracing_subscriber::Layer;

pub mod telegram;
pub use telegram::*;

pub type Message = String;

pub struct GenericNotificationLayer<C, D>
where
    C: 'static,
    D: Fn(&C, Message),
{
    capture_levels: Vec<Level>,
    config: C,
    dispatch: D,
}

impl<C, D> GenericNotificationLayer<C, D>
where
    C: 'static,
    D: Fn(&C, Message),
{
    pub fn new(capture_levels: Vec<Level>, config: C, dispatch: D) -> Self {
        Self {
            capture_levels,
            config,
            dispatch,
        }
    }
}

impl<S: Subscriber, C, D> Layer<S> for GenericNotificationLayer<C, D>
where
    C: 'static,
    D: Fn(&C, Message) + 'static,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        if self.capture_levels.contains(event.metadata().level()) {
            let target = event.metadata().target();
            let fields = event.metadata().fields();
            let message = format!("{target}\n {fields}");
            (self.dispatch)(&self.config, message);
        }
    }
}
