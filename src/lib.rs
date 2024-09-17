use tracing::field::Field;
use tracing::field::Visit;
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

            let mut visitor = MessageCapture::default();
            // Use the visitor pattern to access the event fields
            event.record(&mut visitor);

            if let Some(ref msg) = visitor.message {
                let fields = visitor.get_fields();

                let message = format!(
                    "\n--------\ntarget: {target}\n--------\nmessage: {msg}\n--------\nfields\n{fields}\n"
                );
                (self.dispatch)(&self.config, message);
            }
        }
    }
}

// Visitor to extract fields from the event
#[derive(Default)]
struct MessageCapture {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl MessageCapture {
    pub fn get_fields(&self) -> String {
        self.fields
            .iter()
            .map(|(key, value)| format!("\n{key} = {value}"))
            .fold(String::new(), |mut acc, x| {
                acc += &x;
                acc
            })
    }
}

impl Visit for MessageCapture {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        // Check if the field name is "message"
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
        } else {
            // Otherwise, record it as a normal field
            self.fields
                .push((field.name().to_string(), format!("{:?}", value)));
        }
    }
}

#[cfg(test)]
pub mod test {
    pub use super::*;
    use tracing_subscriber::util::SubscriberInitExt;

    use dotenv::dotenv;
    use tracing::level_filters::LevelFilter;
    use tracing::Level;

    use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
    use tracing_subscriber::EnvFilter;

    /// Set your env and then see if you get notified properly
    #[test]
    fn test_telegram_notifications() {
        dotenv().expect("need a env for the test");
        // build
        let bot_token = std::env::var("BOT_ID").unwrap();
        let chat_id = std::env::var("CHAT_ID").unwrap();
        let tag_users = std::env::var("USERS_TO_TAG")
            .unwrap()
            .split(',')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let client = reqwest::blocking::Client::new();
        let telegram =
            TelegramConfig::new("testing".to_string(), tag_users, bot_token, chat_id, client)
                .build_layer(vec![Level::ERROR]);

        let filter = EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy();

        let layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_filter(filter)
            .boxed();

        let _ = tracing_subscriber::registry()
            .with(vec![layer, telegram.boxed()])
            .try_init();

        tracing::error!("Ah shit a error happened");
        let var = "failure";
        tracing::error!(value = var, "Ah shit a error happened");
    }
}
