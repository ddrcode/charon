use serde_json::Result as JsonResult;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::UnixStream,
};
use your_crate::domain::Event; // adjust this path!

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 📡 Connect to your socket (adjust the path!)
    let stream = UnixStream::connect("/tmp/charon.sock").await?;
    println!("Connected to Charon ferry socket.");

    // 🗂️ Wrap in a buffered reader
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        match serde_json::from_str::<Event>(&line) {
            Ok(event) => {
                println!("Got event: {:#?}", event);
            }
            Err(e) => {
                eprintln!("Invalid event JSON: {line} ({e})");
            }
        }
    }

    println!("Connection closed by server.");
    Ok(())
}
