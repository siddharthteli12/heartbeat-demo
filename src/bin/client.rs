// client.rs
use std::io::{Result, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

const SERVER_ADDR: &str = "127.0.0.1:9999";

fn send_ping(stream: &mut TcpStream) -> Result<()> {
    stream.write_all(b"Ping")?;
    stream.flush()?;
    println!("Ping sent to server");
    Ok(())
}

fn main() -> Result<()> {
    loop {
        match TcpStream::connect(SERVER_ADDR) {
            Ok(mut stream) => {
                match send_ping(&mut stream) {
                    Ok(_) => {}
                    Err(e) => eprintln!("Error sending ping: {}", e),
                }
                thread::sleep(Duration::from_secs(5)); // Wait for 5 seconds before sending the next ping
            }
            Err(e) => {
                eprintln!("Error connecting to server: {}", e);
                thread::sleep(Duration::from_secs(1)); // Retry connection after 1 second
            }
        }
    }
}
