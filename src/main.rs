use std::env;
use tokio::{io::AsyncWriteExt, net::{TcpListener, TcpStream}};
use resp::{RespHandler, Value};
use anyhow::Result;
use crate::{info::get_info, storage::Storage};

mod resp;
mod storage;
mod info;

#[tokio::main]
async fn main() {
    println!("Logs from your program will appear here!");

    let args = env::args().collect::<Vec<String>>();
    let cur_port = match args.iter().position(|arg| arg == "--port") {
        Some(index) => args.get(index + 1).unwrap(),
        None => "6379",
    };
    let is_master;
    let master_port = match args.iter().position(|arg| arg == "--replicaof") {
        Some(index) => {
            is_master = false;

            let mport_params = args.get(index + 1).unwrap().split(' ').collect::<Vec<&str>>();
            let (host, mport) = (mport_params[0], mport_params[1]);

            println!("Start handshake with master port!");
            
            let mut hand_shake = TcpStream::connect(format!("{}:{}", host, mport)).await.expect("Unable to connect master port");

            hand_shake.write_all(b"*1\r\n$4\r\nping\r\n").await.expect("Handshake 1 failed");

            hand_shake.write_all(format!("*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n$4\r\n{}\r\n", cur_port).as_bytes()).await.expect("Handshake 1/2 failed");

            hand_shake.write_all(b"*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n").await.expect("Handshake 2/2 failed");

            mport
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
                "replconf" => Value::SimpleString("OK".to_string()),
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
