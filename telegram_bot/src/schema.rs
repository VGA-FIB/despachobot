use std::{sync::Arc, time::Duration};

use color_eyre::eyre::{Context, Error};
use color_eyre::Result;
use teloxide::{dispatching::UpdateHandler, dptree::filter, prelude::*};
use tokio::sync::Mutex;

use crate::microcontroller::Microcontroller;

pub fn schema() -> UpdateHandler<Error> {
    let command_handler = Message::filter_text()
        .branch(filter(is_alguien_despacho_text).endpoint(answer_alguien_despacho));

    let message_handler = Update::filter_message().branch(command_handler);

    dptree::entry().branch(message_handler)
}

fn is_alguien_despacho_text(text: String) -> bool {
    text.to_lowercase().contains("alguien despacho")
}

async fn answer_alguien_despacho(
    bot: Bot,
    msg: Message,
    microcontroller: Arc<Mutex<Microcontroller>>,
) -> Result<()> {
    let answer = if microcontroller.lock().await.get_state().await? {
        "Ahora mismo hay alguien!"
    } else {
        let diff = microcontroller.lock().await.get_last_falling().await?;
        let diff_duration = round_duration(&Duration::from_millis(diff));

        &format!(
            "No he visto a nadie desde hace {}...",
            humantime::format_duration(diff_duration)
        )
    };

    bot.send_message(msg.chat.id, answer)
        .await
        .wrap_err("Failed to send message")?;
    Ok(())
}

fn round_duration(duration: &Duration) -> Duration {
    let mut secs = duration.as_millis() / 1000;

    if secs > 60 {
        secs = secs - (secs % 60);
    }

    if secs > 3600 {
        secs = secs - (secs % 3600);
    }

    if secs > 24 * 3600 {
        secs = secs - (secs % (24 * 3600));
    }

    Duration::from_secs(secs as u64)
}
