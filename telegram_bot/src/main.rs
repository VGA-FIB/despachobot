use std::sync::Arc;
use std::time::Duration;

use color_eyre::eyre::eyre;
use color_eyre::eyre::Context;
use color_eyre::eyre::OptionExt;
use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use teloxide::dispatching::dialogue;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::dispatching::UpdateHandler;
use teloxide::prelude::*;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::Lines;
use tokio::sync::Mutex;
use tokio_serial::SerialPort;
use tokio_serial::SerialPortType;
use tokio_serial::SerialStream;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    pretty_env_logger::init();
    log::info!("Starting despachobot...");

    let microcontroller_observer = MicrocontrollerObserver::acquire().await?;

    let bot = Bot::from_env();
    let schema = Update::filter_message()
        .filter_map(|update: Update| update.from().cloned())
        .branch(
            Message::filter_text()
                .filter(|text: String| text.to_lowercase().contains("alguien despacho"))
                .endpoint(answer_alguien_despacho),
        );

    Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![Arc::new(Mutex::new(
            microcontroller_observer
        ))])
        .build()
        .dispatch()
        .await;
    Ok(())
}

async fn answer_alguien_despacho(
    bot: Bot,
    msg: Message,
    microcontroller_observer: Arc<Mutex<MicrocontrollerObserver>>,
) -> Result<()> {
    let answer = if microcontroller_observer.lock().await.get_state().await? {
        "Ahora mismo hay alguien!"
    } else {
        let diff = microcontroller_observer
            .lock()
            .await
            .get_last_falling()
            .await?;
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

struct MicrocontrollerObserver(Lines<BufReader<SerialStream>>);

impl MicrocontrollerObserver {
    const SUPPORTED_PROTOCOL_VERSION: u8 = 0;

    pub async fn acquire() -> Result<Self> {
        let serial_port_name = "/dev/ttyACM0";

        let serial_stream = SerialStream::open(&tokio_serial::new(serial_port_name, 115200))
            .wrap_err_with(|| {
                format!(
                    "Failed to open the Arduino serial port ({})",
                    serial_port_name
                )
            })?;

        let lines = BufReader::new(serial_stream).lines();

        let mut new = Self(lines);

        // Test if the arduino responds
        new.ping().await?;

        // Test if the protocol is compatible
        if new.get_protocol_version().await? != Self::SUPPORTED_PROTOCOL_VERSION {
            return Err(eyre!(
                "Arduino is running on an unsupported protocol version"
            ));
        }

        Ok(new)
    }

    pub async fn send_command(&mut self, command: &str) -> Result<String> {
        self.0
            .get_mut()
            .write(command.as_bytes())
            .await
            .wrap_err("Failed to send command to Arduino")?;

        self.0
            .next_line()
            .await
            .wrap_err("Failed to receive command response from Arduino")?
            .ok_or_eyre("Empty command response")
            .map(|r| r.trim().to_owned())
    }

    pub async fn ping(&mut self) -> Result<()> {
        let response = self
            .send_command("ping")
            .await
            .wrap_err("Failed to ping Arduino")?;

        if response == "1" {
            Ok(())
        } else {
            Err(eyre!("Unexpected Arduino ping response"))
        }
    }

    pub async fn get_protocol_version(&mut self) -> Result<u8> {
        let response = self
            .send_command("get_protocol_version")
            .await
            .wrap_err("Failed to query Arduino protocol version")?;

        response
            .parse()
            .wrap_err("Failed to parse Arduino protocol version")
    }

    pub async fn get_state(&mut self) -> Result<bool> {
        let response = self
            .send_command("get_state")
            .await
            .wrap_err("Failed to query Arduino sensor data")?;

        response
            .parse::<u8>()
            .map(|x| x == 1)
            .wrap_err("Failed to parse Arduino sensor state")
    }

    pub async fn get_last_falling(&mut self) -> Result<u64> {
        let response = self
            .send_command("get_last_falling")
            .await
            .wrap_err("Failed to query last Arduino sensor falling edge")?;

        response
            .parse()
            .wrap_err("Failed to parse last Arduino sensor falling edge")
    }
}
