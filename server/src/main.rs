use inv_common::{inv::Inv, CmdCode, ServerHost, Version};

use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::{Ipv6Addr, SocketAddrV6, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};

type Server = Arc<RwLock<ServerHost<TcpStream>>>;

fn tui(save_path: &str, server: Server) {
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut cmd = String::new();
        std::io::stdin().read_line(&mut cmd).unwrap();
        _ = cmd.pop();

        println!("Recieved command...");
        let mut server = server.write().unwrap();
        let inv = &mut server.inv;
        match cmd.as_str() {
            "stop" => {
                let inv_bytes = bincode::serialize(inv).unwrap();
                match std::fs::write(save_path, inv_bytes) {
                    Ok(_) => println!("Saved inv to {save_path:?}"),
                    Err(err) => eprintln!("Failed to save inv to {save_path:?} : {err:?}"),
                }
                std::process::exit(0); // TODO properly shut down TcpListener
            }
            "save" => {
                let inv = bincode::serialize(inv).unwrap();
                match std::fs::write(save_path, inv) {
                    Ok(_) => println!("Saved inv to {save_path:?}"),
                    Err(err) => eprintln!("Failed to save inv to {save_path:?} : {err:?}"),
                }
            }
            "countItems" => {
                println!("{}", inv.items.len());
            }
            "countClients" => {
                println!("{}", server.clients.len());
            }
            "ids" => {
                let item_ids: Vec<_> = inv.items.keys().map(|id| id.0).collect();
                println!("Item IDs: {item_ids:?}");
            }
            s => eprintln!("unknown command : {s:?}"),
        }
    }
}

const INV_VERSION: Version = 1;

fn check_clients(server: &mut ServerHost<TcpStream>) {
    let mut disconnect_clients = HashSet::new();
    for id in server.clients.keys().cloned().collect::<Vec<_>>() {
        let (name, stream) = server.clients.get_mut(&id).unwrap();
        stream.set_nonblocking(true).unwrap();
        let name = name.clone();
        // get command from client
        let mut cmd_buf = [0u8];
        match stream.read(&mut cmd_buf) {
            Ok(0) => {
                println!("Client disconnected {name:?}");
                disconnect_clients.insert(id);
                continue;
            }
            Ok(1) => {}
            Ok(_) => unreachable!(),
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(err) => {
                println!("Client disconnected {name:?} : {err:?}");
                disconnect_clients.insert(id);
                continue;
            }
        }
        stream.set_nonblocking(false).unwrap();
        let Some(cmd) = CmdCode::from_u8(cmd_buf[0]) else {
            println!("Client sent unknown command {name:?} : {}", cmd_buf[0]);
            continue;
        };
        if let Err(err) = server.handle_client_cmd(id, cmd) {
            eprintln!("Encountered error while dealing with client ({name}) : {err:?}");
        }
    }
    for id in disconnect_clients {
        _ = server.clients.remove(&id).unwrap();
    }
}

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    let _program_path = args.next().unwrap();
    let port: u16 = args
        .next()
        .expect("Missing 1st arg: port number to bind to")
        .parse()
        .expect("Invalid 1st arg: port number not a valid int");
    let save_path = args
        .next()
        .expect("Missing 2nd arg: path to the save file location");

    let addr: Ipv6Addr = "::".parse().unwrap();
    let listener = TcpListener::bind(SocketAddrV6::new(addr, port, 0, 0))?;

    let mut inv = Inv::default();
    if let Ok(bytes) = std::fs::read(&save_path) {
        match bincode::deserialize(&bytes) {
            Ok(new_inv) => inv = new_inv,
            Err(err) => eprintln!("Failed to parse inv as Inv VER 1: {err:?}"),
        }
    }

    let server = Arc::new(RwLock::new(ServerHost::new(INV_VERSION, inv)));

    let save_path_clone = save_path.clone();
    let server0 = server.clone();
    std::thread::spawn(move || tui(&save_path_clone, server0));

    let server1 = server.clone();
    std::thread::spawn(move || loop {
        check_clients(&mut server1.write().unwrap());
        std::thread::sleep(std::time::Duration::from_millis(1000));
    });

    let server2 = server.clone();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut server = server2.write().unwrap();
                match server.connect_client(stream) {
                    Ok(_) => {}
                    Err(err) => eprintln!("Client failed to connect : {err:?}"),
                }
            }
            Err(e) => eprintln!("Connection to client failed {e:?}"),
        }
    }

    let inv = bincode::serialize(&server.read().unwrap().inv).unwrap();
    match std::fs::write(&save_path, inv) {
        Ok(_) => println!("Saved inv to {save_path:?}"),
        Err(err) => eprintln!("Failed to save inv to {save_path:?} : {err:?}"),
    }
    Ok(())
}
