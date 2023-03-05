use byteorder::{ReadBytesExt, WriteBytesExt};
use rand::RngCore;

// C1/2 S1/2
use super::constants::{RTMP_HANDSHAKE_SIZE, RTMP_VERSION};
use super::errors::HandshakeError;
use std::io::{Read, Write};

pub trait ReadWrite: Read + Write {}

pub fn perform_handshake<T>(io: &mut T) -> Result<(), HandshakeError>
where
    T: Read + Write,
{
    let mut n = 0usize;

    io.write_u8(0x03)?; // C9

    let client_ts = chrono::Utc::now().timestamp_micros() as u32;
    io.write_u32::<byteorder::BE>(client_ts)?; // c1 timestamp
    io.write_u32::<byteorder::BE>(0)?; // s1 timestamp space
    let mut c_rnd_data = [0u8; RTMP_HANDSHAKE_SIZE - 8]; // the whole packe is that size, we subtract 8 for the two timestamp fields
    rand::thread_rng().fill_bytes(&mut c_rnd_data);
    n = io.write(&c_rnd_data)?; // handshake
    println!("{n}");

    io.flush()?;

    let server_version = io.read_u8()?;

    if server_version != RTMP_VERSION as u8 {
        return Err(HandshakeError::InvalidServerVersion(server_version));
    }

    let mut recv_client_ts = [0u8; 4];
    let mut recv_server_ts = [0u8; 4];
    let mut server_echo = [0u8; RTMP_HANDSHAKE_SIZE - 8];

    n = io.read(&mut recv_client_ts)?;
    println!("{n}");
    io.read_exact(&mut recv_server_ts)?;
    io.read_exact(&mut server_echo)?;

    Ok(())
}

struct HandshakePacket {
    client_time: u32,
    server_time: u32,
    echo: [u8; 1536],
}
