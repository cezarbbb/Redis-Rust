use tokio::net::{TcpListener, TcpStream};
use resp::{RespHandler, Value};
use anyhow::Result;
use crate::storage::Storage;
mod resp;
mod storage;

#[tokio::main]
async fn main() {
    println!("Logs from your program will appear here!");

    let port = std::env::args().nth(2).unwrap_or("6379".to_string());

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await.unwrap();
    
    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((stream, _)) => {
                println!("Get new connection!");
                tokio::spawn(async move {
                    handle_conn(stream).await;
                });
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}

async fn handle_conn(stream: TcpStream) {
    let mut handler = RespHandler::new(stream);

    println!("Start reading loop!");

    let mut storage: Storage = Default::default();

    loop {
        let value = handler.read_value().await.unwrap();
        
        println!("Got value {:?}", value);

        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();
            match command.as_str() {
                "PING" => Value::SimpleString("PONG".to_string()),
                "ECHO" => args.first().unwrap().clone(),
                "SET" => {
                    match args.len() {
                        2 => storage.set(unpack_bulk_str(args[0].clone()).unwrap(), unpack_bulk_str(args[1].clone()).unwrap(), 0),
                        4 => storage.set(unpack_bulk_str(args[0].clone()).unwrap(), unpack_bulk_str(args[1].clone()).unwrap(), unpack_bulk_str(args[3].clone()).unwrap().parse().unwrap()),
                        _ => panic!("SET command has invalid params {}", args.len()),
                    }
                },
                "GET" => storage.get(unpack_bulk_str(args[0].clone()).unwrap()),
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
