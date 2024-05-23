use std::env;
use tokio::net::{TcpListener, TcpStream};
use resp::{RespHandler, Value};
use anyhow::Result;
use crate::storage::Storage;
mod resp;
mod storage;

#[tokio::main]
async fn main() {
    println!("Logs from your program will appear here!");

    let args = env::args().collect::<Vec<String>>();
    let cur_port = args.iter().position(|arg| arg == "--port").and_then(|index| args.get(index + 1)).unwrap();
    let is_master;
    let master_port = match args.iter().position(|arg| arg == "--replicaof") {
        Some(index) => {
            is_master = false;
            args.get(index + 1).unwrap()
        },
        None => {
            is_master = true;
            cur_port
        },
    };
    println!("Current port:{}", cur_port);
    println!("Master port:{}", master_port);

    let listener = TcpListener::bind(format!("127.0.0.1:{}", cur_port)).await.unwrap();
    
    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((stream, _)) => {
                println!("Get new connection!");
                tokio::spawn(async move {
                    handle_conn(stream, is_master).await;
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

        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();
            match command.to_ascii_lowercase().as_str() {
                "ping" => Value::SimpleString("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                "set" => {
                    match args.len() {
                        2 => storage.set(unpack_bulk_str(args[0].clone()).unwrap(), unpack_bulk_str(args[1].clone()).unwrap(), 0),
                        4 => storage.set(unpack_bulk_str(args[0].clone()).unwrap(), unpack_bulk_str(args[1].clone()).unwrap(), unpack_bulk_str(args[3].clone()).unwrap().parse().unwrap()),
                        _ => panic!("SET command has invalid params {}", args.len()),
                    }
                },
                "get" => storage.get(unpack_bulk_str(args[0].clone()).unwrap()),
                "info" => Value::BulkString(format!("role:{}", if is_master {"master"} else {"slave"}).to_string()),
                _ => panic!("Can not handle command {}", command),
            }
        } else { break;};

        println!("Sending value {:?}", response);

        handler.write_value(response).await.unwrap();
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
