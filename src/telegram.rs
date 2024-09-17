use tracing::Level;

use crate::{GenericNotificationLayer, Message};

#[derive(Debug, Clone)]
pub struct TelegramConfig {
    codebase_name: &'static str,
    bot_token: &'static str,
    chat_id: &'static str,
    request_client: reqwest::blocking::Client,
}

impl TelegramConfig {
    pub fn new(
        codebase_name: &'static str,
        bot_token: &'static str,
        chat_id: &'static str,
        request_client: reqwest::blocking::Client,
    ) -> Self {
        Self {
            codebase_name,
            request_client,
            chat_id,
            bot_token,
        }
    }

    pub fn build_layer(
        self,
        notification_levels: Vec<Level>,
    ) -> GenericNotificationLayer<Self, impl for<'a> Fn(&'a Self, Message)> {
        GenericNotificationLayer::new(notification_levels, self.clone(), |ctx, message| {
            let url = format!("https://api.telegram.org/bot{}/sendMessage", ctx.bot_token);

            let params = [
                ("chat_id", ctx.chat_id),
                (
                    "text",
                    &format!("codebase: {} had a error \n {}", ctx.codebase_name, message),
                ),
            ];

            ctx.request_client
                .post(&url)
                .form(&params)
                .send()
                .expect("failed to notify remote of error");
        })
    }
}
