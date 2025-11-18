use rand::Rng;
use std::{
    env, fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    process,
};
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{}", err);
        process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let config = Config::from_env()?;
    let payloads = load_payloads(&config.folder)?;
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|err| format!("Failed to bind UDP socket: {}", err))?;

    let mut rng = rand::thread_rng();
    loop {
        let idx = rng.gen_range(0..payloads.len());
        if let Err(err) = socket.send_to(&payloads[idx], config.target).await {
            eprintln!("Failed to send datagram: {}", err);
        }
    }
}

struct Config {
    target: SocketAddr,
    folder: PathBuf,
}

impl Config {
    fn from_env() -> Result<Self, String> {
        let mut args = env::args();
        let program = args.next().unwrap_or_else(|| "udp-stress".to_string());
        let usage_msg = usage(&program);
        let usage_text = usage_msg.as_str();

        let target_str = args
            .next()
            .ok_or_else(|| format!("Missing target address.\n{}", usage_text))?;

        let target = target_str.parse().map_err(|err| {
            format!(
                "Invalid target address '{}': {}.\n{}",
                target_str, err, usage_text
            )
        })?;

        let folder = args
            .next()
            .map(PathBuf::from)
            .unwrap_or_else(default_files_dir);

        Ok(Self { target, folder })
    }
}

fn load_payloads(folder: &Path) -> Result<Vec<Vec<u8>>, String> {
    let entries = fs::read_dir(folder)
        .map_err(|err| format!("Failed to read folder {}: {}", folder.display(), err))?;

    let mut payloads = Vec::new();
    for entry in entries {
        let entry = entry
            .map_err(|err| format!("Failed to read entry in {}: {}", folder.display(), err))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|err| format!("Failed to read metadata for {}: {}", path.display(), err))?;

        if !file_type.is_file() {
            continue;
        }

        let contents =
            fs::read(&path).map_err(|err| format!("Failed to read {}: {}", path.display(), err))?;
        payloads.push(contents);
    }

    if payloads.is_empty() {
        return Err(format!(
            "No readable files found in {}. Provide a folder with at least one file.",
            folder.display()
        ));
    }

    Ok(payloads)
}

fn default_files_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|parent| parent.join("files")))
        .unwrap_or_else(|| PathBuf::from("./files"))
}

fn usage(program: &str) -> String {
    format!("Usage: {} <ADDR:PORT> [FOLDER]", program)
}
