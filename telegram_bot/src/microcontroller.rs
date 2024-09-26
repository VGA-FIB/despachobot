use color_eyre::{
    eyre::eyre,
    eyre::{Context, OptionExt},
    Result,
};
use futures_util::SinkExt;
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Framed};

use crate::codecs::{ArduinoCodec, ArduinoCommand};

pub struct Microcontroller(Framed<SerialStream, ArduinoCodec>);

impl Microcontroller {
    const SUPPORTED_PROTOCOL_VERSION: u8 = 0;

    pub async fn acquire(serial_port_name: &str) -> Result<Self> {
        let serial_stream = tokio_serial::new(serial_port_name, 115200)
            .open_native_async()
            .wrap_err_with(|| {
                format!(
                    "Failed to open the Arduino serial port ({})",
                    serial_port_name
                )
            })?;

        let mut new = Self(ArduinoCodec::new().framed(serial_stream));

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

    pub async fn send_command(&mut self, command: ArduinoCommand) -> Result<u64> {
        self.0
            .send(command.into())
            .await
            .wrap_err("Failed to send command to Arduino")?;

        let response = self
            .0
            .next()
            .await
            .ok_or_eyre("Arduino command response stream closed")?
            .wrap_err("Failed to receive command response from Arduino")?;

        Ok(response)
    }

    pub async fn ping(&mut self) -> Result<()> {
        let response = self
            .send_command(ArduinoCommand::Ping)
            .await
            .wrap_err("Failed to ping Arduino")?;

        if response == 1 {
            Ok(())
        } else {
            Err(eyre!("Unexpected Arduino ping response"))
        }
    }

    pub async fn get_protocol_version(&mut self) -> Result<u8> {
        self.send_command(ArduinoCommand::GetProtocolVersion)
            .await
            .wrap_err("Failed to query Arduino protocol version")
            .map(|x| x as u8)
    }

    pub async fn get_state(&mut self) -> Result<bool> {
        self.send_command(ArduinoCommand::GetState)
            .await
            .wrap_err("Failed to query Arduino sensor data")
            .map(|x| x == 1)
    }

    pub async fn get_last_falling(&mut self) -> Result<u64> {
        self.send_command(ArduinoCommand::GetLastFalling)
            .await
            .wrap_err("Failed to query last Arduino sensor falling edge")
    }
}
