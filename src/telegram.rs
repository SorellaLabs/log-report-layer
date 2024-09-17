use tracing::Level;

use crate::{GenericNotificationLayer, Message};

#[derive(Debug, Clone)]
pub struct TelegramConfig {
    codebase_name: String,
    users_to_tag: Vec<String>,
    bot_token: String,
    chat_id: String,
    request_client: reqwest::blocking::Client,
}

impl TelegramConfig {
    pub fn new(
        codebase_name: String,
        users_to_tag: Vec<String>,
        bot_token: String,
        chat_id: String,
        request_client: reqwest::blocking::Client,
    ) -> Self {
        Self {
            codebase_name,
            users_to_tag,
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
                ("chat_id", ctx.chat_id.clone()),
                (
                    "text",
                    format!(
                        "{:#?}\ncodebase: {} had a error \n{}",
                        ctx.users_to_tag, ctx.codebase_name, message
                    ),
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
