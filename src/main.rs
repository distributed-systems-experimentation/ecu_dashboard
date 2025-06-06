use ecu_engine_messages::SpeedMessage;
use futures::StreamExt;
use tire_monitor_messages::TirePressureMessage;
use tokio::net::{TcpListener, UdpSocket};
use tokio::signal::unix::{SignalKind, signal};
use tokio_serde::Framed;
use tokio_serde::formats::*;
use tokio_util::codec::{BytesCodec, Framed as UtilFramed, LengthDelimitedCodec};
use tokio_util::udp::UdpFramed;

#[tokio::main]
async fn main() {
    // Create a TCP listener bound to "localhost:8080"
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Listening on port 8080...");

    // Bind to any available port
    let socket = UdpSocket::bind("0.0.0.0:8080").await.unwrap();
    let mut framed = UdpFramed::new(socket, BytesCodec::new());

    let mut sigterm = signal(SignalKind::terminate()).unwrap();

    // Accept connections and process them asynchronously
    loop {
        tokio::select! {
            incoming = listener.accept() => {
                match incoming {
                    Ok((stream, _)) => {
                        tokio::spawn(async move {
                            // Create a LengthDelimitedCodec
                            let length_delimited = UtilFramed::new(stream, LengthDelimitedCodec::new());

                            // Create a TokioSerde framed stream
                            let mut framed: Framed<
                                UtilFramed<tokio::net::TcpStream, LengthDelimitedCodec>,
                                SpeedMessage,
                                SpeedMessage,
                                Json<SpeedMessage, SpeedMessage>,
                            > = Framed::new(length_delimited, Json::<SpeedMessage, SpeedMessage>::default());

                            loop {
                                // Read data using tokio_serde
                                match framed.next().await { // Use next() again
                                    Some(Ok(speed_msg)) => {
                                        println!("Received speed: {}", speed_msg.speed);
                                    }
                                    Some(Err(e)) => {
                                        eprintln!("Error reading from connection: {}", e);
                                        break; // Exit the loop on error
                                    }
                                    None => {
                                        // Connection closed
                                        println!("Connection closed by peer.");
                                        break;
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => println!("Connection failed: {}", e),
                }
            }
            Some(Ok((message, addr))) = framed.next() => {
                let (message, _): (TirePressureMessage, _) = bincode::borrow_decode_from_slice(&message, bincode::config::standard()).unwrap();

                // Handle incoming UDP messages
                println!("Received UDP message: {:?} from {}", message, addr);
                // Here you can process the message as needed
            }
            _ = sigterm.recv() => {
                println!("SIGTERM received, shutting down.");
                break;
            }
        }
    }
}
