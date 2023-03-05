use anyhow::Result as AnyhowResult;
use rtmp::channels::channels::ChannelsManager;
use rtmp::rtmp::RtmpServer;
use tokio::signal;

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    let mut channel = ChannelsManager::new();
    let producer = channel.get_session_event_producer();

    let listen_port = 1935;
    let address = format!("0.0.0.0:{port}", port = listen_port);

    let mut rtmp_server = RtmpServer::new(address, producer.clone());
    tokio::spawn(async move {
        if let Err(err) = rtmp_server.run().await {
            log::error!("rtmp server error: {}\n", err);
        }
    });

    tokio::spawn(async move { channel.run().await });

    signal::ctrl_c().await?;
    Ok(())
}
