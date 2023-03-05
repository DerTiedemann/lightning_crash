use std::{net::TcpStream, sync::Arc};

use anyhow::Result as AnyhowResult;

// c = client
// s = server

use lightning_crash::client::handshake::perform_handshake;
use rustls::OwnedTrustAnchor;

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let config = rustls::ClientConfig::builder()
        .with_cipher_suites(&[rustls::cipher_suite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256])
        .with_kx_groups(&rustls::ALL_KX_GROUPS)
        .with_protocol_versions(&[&rustls::version::TLS12])
        .unwrap()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let mut conn = rustls::ClientConnection::new(
        Arc::new(config),
        "prod.euw1.lol.riotgames.com".try_into().unwrap(),
    )?;

    let mut tcp_stream = TcpStream::connect("localhost:2099")?;
    let mut tls = rustls::Stream::new(&mut conn, &mut tcp_stream);

    // let mut tcp_stream = TcpStream::connect("prod.euw1.lol.riotgames.com:2099")?;

    perform_handshake(&mut tls)?;

    tcp_stream.shutdown(std::net::Shutdown::Both)?;

    // let mut client = rtmp::handshake::handshake_client::SimpleHandshakeClient::new(Arc::new(
    //     Mutex::new(BytesIO::new(tcp_stream)),
    // ));

    // for i in 0..3 {
    //     match client.handshake().await {
    //         Ok(_) => {
    //             println!("gucci handshake {i}");
    //         }
    //         Err(e) => return Err(anyhow::anyhow!("{e}")),
    //     }
    // }
    Ok(())
}
