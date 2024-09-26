use std::sync::Arc;

use clap::Parser;
use color_eyre::eyre::Result;
use microcontroller::Microcontroller;
use teloxide::prelude::*;
use tokio::sync::Mutex;

mod codecs;
mod microcontroller;
mod schema;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path of the serial port where the Arduino is connected
    serial_path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    pretty_env_logger::init();

    let cli = Cli::parse();

    let microcontroller = Microcontroller::acquire(&cli.serial_path).await?;

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema::schema())
        .dependencies(dptree::deps![Arc::new(Mutex::new(microcontroller))])
        .enable_ctrlc_handler()
        .default_handler(|_| async {})
        .build()
        .dispatch()
        .await;
    Ok(())
}
