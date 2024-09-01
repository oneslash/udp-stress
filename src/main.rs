use rand::Rng;
use std::{fs, net::SocketAddr, sync::Arc};
use tokio::{net::UdpSocket, sync::mpsc};

#[tokio::main]
async fn main() {
    // Cli param get
    let args: Vec<String> = std::env::args().collect();
    let target_addr: SocketAddr = args.get(1).unwrap().parse().unwrap();
    let folder_path: String = args.get(2).unwrap_or(&"./files".to_string()).to_string();

    let files = match fs::read_dir(folder_path) {
        Ok(list) => list,
        Err(e) => {
            eprintln!("Folder error: {}", e);
            return;
        }
    };

    let file_vec: Vec<String> = files
        .map(|f| f.unwrap().path().to_str().unwrap().to_string())
        .collect();

    let sock = UdpSocket::bind("0.0.0.0:8080".parse::<SocketAddr>().unwrap())
        .await
        .unwrap();
    let r = Arc::new(sock);
    let s = r.clone();
    let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);

    tokio::spawn(async move {
        while let Some((bytes, addr)) = rx.recv().await {
            let len = s.send_to(&bytes, &addr).await.unwrap();
            println!("Sent {} bytes to {}:", len, addr);
        }
    });

    loop {
        // random file selection
        let random_index = rand::thread_rng().gen_range(0..file_vec.len() - 1);
        let file_contents = match fs::read(&file_vec[random_index]) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };
        tx.send((file_contents.to_vec(), target_addr))
            .await
            .unwrap();
    }
}
