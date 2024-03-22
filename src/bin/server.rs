// server.rs
use plotters::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

type ClientMap = Arc<Mutex<HashMap<String, Vec<(Instant, Instant)>>>>;

const PLOT_INTERVAL_SECS: u64 = 10; // Interval for plotting client activity in seconds

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:9999").await?;
    let (tx, _) = broadcast::channel(10);
    let client_map: ClientMap = Arc::new(Mutex::new(HashMap::new()));

    // Spawn a task to periodically plot client activity
    let client_map_clone = client_map.clone();
    tokio::spawn(async move {
        plot_activity(client_map_clone).await;
    });

    loop {
        let (stream, _) = listener.accept().await?;
        println!("Client connected: {}", stream.peer_addr().unwrap());
        let tx = tx.clone();
        let client_map = client_map.clone();

        tokio::spawn(async move {
            handle_client(stream, tx, client_map).await;
        });
    }
}

async fn handle_client(
    mut stream: TcpStream,
    tx: broadcast::Sender<String>,
    client_map: ClientMap,
) {
    let client_address = stream.peer_addr().unwrap().to_string();
    let mut buf = [0; 1024];

    // Register the client
    {
        let mut map = client_map.lock().unwrap();
        map.insert(client_address.clone(), Vec::new());
    }

    loop {
        let n = match stream.read(&mut buf).await {
            Ok(n) if n == 0 => break, // Client disconnected
            Ok(n) => n,
            Err(_) => break,
        };
        let _ = tx.send(client_address.clone()); // Notify other parts of the application

        // Record client activity
        {
            let mut map = client_map.lock().unwrap();
            if let Some(activity) = map.get_mut(&client_address) {
                if n > 0 {
                    let start = Instant::now();
                    let end = start + Duration::from_secs(5); // Assume a ping interval of 5 seconds
                    activity.push((start, end));
                }
            }
        }

        // Reset the buffer
        for i in 0..n {
            buf[i] = 0;
        }
    }

    // Deregister the client
    {
        let mut map = client_map.lock().unwrap();
        map.remove(&client_address);
    }
}

async fn plot_activity(client_map: ClientMap) {
    loop {
        tokio::time::sleep(Duration::from_secs(PLOT_INTERVAL_SECS)).await;

        let root = BitMapBackend::new("client_activity.png", (800, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption("Client Activity", ("sans-serif", 20))
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(0..10, 0..10)
            .unwrap();

        let mut map = client_map.lock().unwrap();
        for (i, (client, activity)) in map.iter_mut().enumerate() {
            let color = Palette99::pick(i);
            for (start, end) in activity.iter() {
                let x1 = (*start - Instant::now()).as_secs() as i32;
                let x2 = (*end - Instant::now()).as_secs() as i32;
                chart
                    .draw_series(LineSeries::new(
                        vec![(x1, i as i32), (x2, i as i32)],
                        &color,
                    ))
                    .unwrap();
            }
        }

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()
            .unwrap();
    }
}
