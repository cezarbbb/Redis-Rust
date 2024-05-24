use std::env;
use tokio::{io::AsyncWriteExt, net::{TcpListener, TcpStream}};
use resp::{RespHandler, Value};
use anyhow::Result;
use crate::{command_response::{get_info, handle_psync}, storage::Storage, port::{Port, PortType}};

mod resp;
mod storage;
mod command_response;
mod port;
mod handshake;

#[tokio::main]
async fn main() {
    println!("Logs from your program will appear here!");

    let args = env::args().collect::<Vec<String>>();
    let cur_port_id = match args.iter().position(|arg| arg == "--port") {
        Some(index) => args.get(index + 1).unwrap(),
        None => "6379",
    };
    let (cur_port, master_port);
    let master_port = match args.iter().position(|arg| arg == "--replicaof") {
        Some(index) => {
            let mport_params = args.get(index + 1).unwrap().split(' ').collect::<Vec<&str>>();
            let (host, mport) = (mport_params[0], mport_params[1]);

            master_port = Port::new(mport.to_string(), PortType::Master);
            cur_port = Port::new(cur_port_id.to_string().clone(), PortType::Slave);

            handshake::send_hand_shake(host.to_string(), master_port.id.clone(), cur_port.id.clone()).await;

            master_port
        },
        None => {
            master_port = Port::new(cur_port_id.to_string().clone(), PortType::Master);
            cur_port = Port::new(cur_port_id.to_string().clone(), PortType::Master);
            master_port
        },
    };
    println!("Current port:{}", cur_port.id);
    println!("Master port:{}", master_port.id);

    let listener = TcpListener::bind(format!("127.0.0.1:{}", master_port.id)).await.unwrap();
    
    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((stream, _)) => {
                println!("Get new connection!");
                tokio::spawn(async move {
                    handle_conn(stream, match cur_port.port_type {
                        PortType::Master => true,
                        PortType::Slave => false,
                    }).await;
                });
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}

async fn handle_conn(stream: TcpStream, is_master: bool) {
    let mut handler = RespHandler::new(stream);

    println!("Start reading loop!");

    let mut storage: Storage = Default::default();

    loop {
        let value = handler.read_value().await.unwrap();
        
        println!("Got value {:?}", value);

        let mut if_send_rdb = false;

        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();
            match command.to_ascii_lowercase().as_str() {
                "ping" => Value::SimpleString("PONG".to_string()),
                "replconf" => Value::SimpleString("OK".to_string()),
                "psync" => {
                    if_send_rdb = true;
                    handle_psync()
                },
                "echo" => args.first().unwrap().clone(),
                "set" => {
                    match args.len() {
                        2 => storage.set(unpack_bulk_str(args[0].clone()).unwrap(), unpack_bulk_str(args[1].clone()).unwrap(), 0),
                        4 => storage.set(unpack_bulk_str(args[0].clone()).unwrap(), unpack_bulk_str(args[1].clone()).unwrap(), unpack_bulk_str(args[3].clone()).unwrap().parse().unwrap()),
                        _ => panic!("SET command has invalid params {}", args.len()),
                    }
                },
                "get" => storage.get(unpack_bulk_str(args[0].clone()).unwrap()),
                "info" => get_info(is_master),
                _ => panic!("Can not handle command {}", command),
            }
        } else { break;};

        println!("Sending value {:?}", response);

        handler.write_value(response).await.unwrap();

        if if_send_rdb {
            handler.write_value(Value::RDBString("empty rdb file".to_string())).await.unwrap();
        }
    }
}

fn extract_command(value: Value) -> Result<(String, Vec<Value>)> {
    match value {
        Value::Array(a) => Ok((unpack_bulk_str(a.first().unwrap().clone())?, a.into_iter().skip(1).collect())),
        _ => Err(anyhow::anyhow!("Unexpected command format!")),
    }
}

fn unpack_bulk_str(value: Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Expect command to be a bulk string!")),
    }
}
