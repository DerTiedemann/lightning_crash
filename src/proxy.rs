use anyhow::Result as AnyhowResult;
use clap::{arg, command, Parser};
use tokio::{
    net::{TcpListener, TcpStream},
    select,
};

use lightning_crash::interceptor::{copy, SnifferBuffer};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// which address/interface to bind
    #[arg(short, long)]
    bind: std::net::SocketAddr,
    /// upstream server to relay traffic
    #[arg(short, long, default_value = "185.40.64.65:2099")]
    target: String,
}

#[tokio::main]

async fn main() -> AnyhowResult<()> {
    let args = Args::parse();

    let listener = TcpListener::bind(args.bind).await?;

    loop {
        let (source, _) = listener.accept().await?;
        let source_address = source.peer_addr()?;
        println!("new source ({:?}) connected", source_address);
        let target = TcpStream::connect(&args.target).await?;
        let target_address = target.peer_addr()?;

        let (mut sread, mut swrite) = source.into_split();
        let (mut tread, mut twrite) = target.into_split();

        let mut s_buffer = SnifferBuffer::default();
        s_buffer.set_callback(Box::new(|data: &Vec<u8>| {
            println!("source {}", "=".repeat(20));
            hexdump::hexdump(data);
            println!("source end {}", "=".repeat(16));
        }));

        let s2t = tokio::spawn(async move { copy(&mut sread, &mut twrite, s_buffer).await });
        let mut t_buffer = SnifferBuffer::default();
        t_buffer.set_callback(Box::new(|data: &Vec<u8>| {
            println!("target {}", "=".repeat(20));
            hexdump::hexdump(data);
            println!("target end {}", "=".repeat(16));
        }));
        let t2s = tokio::spawn(async move { copy(&mut tread, &mut swrite, t_buffer).await });

        select! {
            _ = s2t => {
                println!("source ({:?}) disconnected", source_address);
                t2s.abort();
            },
            _ = t2s => println!("target ({:?}) disconnected", target_address),
        }
    }
}
