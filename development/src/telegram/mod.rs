use async_trait::async_trait;

/// Thin abstraction over Telegram so handlers can be tested without a real bot token.
#[async_trait]
pub trait TelegramClient: Send + Sync + 'static {
    /// Send a plain-text or MarkdownV2 message to the configured chat.
    async fn send_message(&self, text: &str) -> anyhow::Result<i64>;

    /// Edit the text of a previously sent message.
    async fn edit_message(&self, message_id: i64, text: &str) -> anyhow::Result<()>;

    /// Answer an inline keyboard callback query (clears the spinner on the button).
    async fn answer_callback(&self, callback_query_id: &str) -> anyhow::Result<()>;
}
