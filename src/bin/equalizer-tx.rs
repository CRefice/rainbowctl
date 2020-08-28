use std::fs::File;
use std::io::{BufReader, Read as _};
use std::net::UdpSocket;

const NUM_CHANNELS: usize = 24;
const BUF_SIZE: usize = NUM_CHANNELS + 4;
const PORT: u16 = 9999;

const FIFO_PATH: &str = "/tmp/cava.fifo";

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind(("0.0.0.0", PORT)).expect("couldn't create UDP socket");
    socket
        .connect(("192.168.1.4", PORT))
        .expect("couldn't connect to remote server");

    let mut fifo = BufReader::new(File::open(FIFO_PATH)?);
    let mut buf = [0; BUF_SIZE];

    for counter in 0u32.. {
        buf[..4].copy_from_slice(&counter.to_le_bytes());
        fifo.read_exact(&mut buf[4..])?;
        socket.send(&buf).expect("couldn't send message");
    }
    Ok(())
}
