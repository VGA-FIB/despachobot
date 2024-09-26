use std::fmt::Display;

use color_eyre::eyre::{Context, Error};
use tokio_util::codec::{Decoder, Encoder, LinesCodec};

pub enum ArduinoCommand {
    Ping,
    GetProtocolVersion,
    GetState,
    GetLastFalling,
}

impl Display for ArduinoCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Ping => "ping",
            Self::GetProtocolVersion => "get_protocol_version",
            Self::GetState => "get_state",
            Self::GetLastFalling => "get_last_falling",
        })
    }
}

pub struct ArduinoCodec(LinesCodec);

impl ArduinoCodec {
    pub fn new() -> Self {
        Self(LinesCodec::new())
    }
}

impl Decoder for ArduinoCodec {
    type Item = u64;
    type Error = Error;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        self.0.decode(src).map(|x| {
            x.map(|s| s.parse().wrap_err("Unexpected value format"))
                .transpose()
        })?
    }
}

impl Encoder<ArduinoCommand> for ArduinoCodec {
    type Error = <LinesCodec as Encoder<String>>::Error;

    fn encode(
        &mut self,
        item: ArduinoCommand,
        dst: &mut tokio_util::bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        self.0.encode(item.to_string(), dst)
    }
}
