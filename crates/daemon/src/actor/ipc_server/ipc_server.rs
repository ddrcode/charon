use tokio::{
    io::AsyncReadExt,
    net::{UnixListener, UnixStream},
    sync::mpsc::{Receiver, Sender},
};

use crate::domain::{Actor, Event};

pub struct IPCServer {
    tx: Sender<Event>,
    rx: Receiver<Event>,
    client: Option<UnixStream>,
    listener: UnixListener,
}

impl IPCServer {
    // fn handle(&self, cmd: ClientCommand) {
    //     match cmd {
    //         ClientCommand::OpenVim => {
    //             self.tx.send(Event::OpenVim).unwrap();
    //         }
    //         ClientCommand::PausePassThrough => {
    //             self.tx.send(Event::PausePassThrough).unwrap();
    //         }
    //         _ => { /* etc */ }
    //     }
    // }
}

#[async_trait::async_trait]
impl Actor for IPCServer {
    async fn run(&mut self) {
        // let listener = UnixListener::bind("/tmp/charon.sock").unwrap();
        // let (stream, _addr) = listener.accept().await.unwrap();
        // loop {
        //     tokio::select! {
        //         Result(cmd) = stream.read() => {
        //             self.tx.send(Event::from(cmd));
        //         }
        //         Some(evt) = self.rx.recv() => {
        //             // socket.write(serialize(evt)).await;
        //         }
        //     }
        // }
        // for stream in listener.incoming() {
        //     match stream {
        //         Ok(mut s) => {
        //             let mut buf = String::new();
        //             s.read_to_string(&mut buf).unwrap();
        //             let cmd: ClientCommand = serde_json::from_str(&buf).unwrap();
        //             self.handle(cmd);
        //         }
        //         Err(e) => eprintln!("IPC error: {}", e),
        //     }
        // }
    }

    fn id() -> &'static str {
        "ipc-server"
    }

    fn sender(&self) -> &Sender<Event> {
        &self.tx
    }
}
