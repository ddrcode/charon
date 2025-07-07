use charon_lib::domain::{DomainEvent, Event, Mode};
use serde_json::Result as JsonResult;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixStream, unix::WriteHalf},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut stream = UnixStream::connect("/tmp/charon.sock").await?;
    println!("Connected to Charon ferry socket.");

    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let mut alive = true;

    while alive {
        match reader.read_line(&mut line).await {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                match serde_json::from_str::<Event>(&line) {
                    Ok(event) => {
                        println!("Got event: {} {:#?}", event.sender, event.payload);
                        alive = handle_event(&event, &mut writer).await?;
                    }
                    Err(e) => {
                        eprintln!("Invalid event JSON: {line}");
                    }
                }
            }
            Err(e) => eprintln!("ERROR {e}"),
        }
        line.clear();
    }

    println!("Connection closed by server.");
    Ok(())
}

async fn handle_event(event: &Event, writer: &mut WriteHalf<'_>) -> anyhow::Result<bool> {
    match event.payload {
        DomainEvent::ModeChange(Mode::InApp) => {
            let path = run_editor()?;
            let e = Event::new("client", DomainEvent::SendFile(path));
            let eser = serde_json::to_string(&e)?;
            writer.write_all(eser.as_bytes()).await?;
            writer.write_all(b"\n").await?;
        }
        DomainEvent::Exit => {
            return Ok(false);
        }
        _ => {}
    }
    Ok(true)
}

fn run_editor() -> anyhow::Result<String> {
    // use std::fs::read_to_string;
    use std::process::Command;
    use tempfile::NamedTempFile;

    let tmp = NamedTempFile::new()?;
    let path = tmp.path().to_owned();

    Command::new("nvim").arg(&path).status()?;

    // let content = read_to_string(path)?;
    //
    // let event = Event::new("client", DomainEvent::SendBuffer(content));
    // broker_tx.send(event).await?;

    Ok(path.to_string_lossy().into())
}
