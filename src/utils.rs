use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub const MAX_PACKET_LENGTH: usize = 1024;

pub type ClientId = u32;

pub fn create_framed_stream(stream: TcpStream) -> Framed<TcpStream, LengthDelimitedCodec> {
   LengthDelimitedCodec::builder()
        .length_field_type::<u16>()
        .little_endian()
        .max_frame_length(MAX_PACKET_LENGTH)
        .new_framed(stream)
}