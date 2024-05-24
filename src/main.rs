use std::sync::Arc;
use server::RedisServer;
use tokio::{net::{TcpListener, TcpStream}, sync::{broadcast::{self, Sender}, Mutex}};
use resp::{RespHandler, Value};
use anyhow::Result;
use crate::{command_response::handle_psync, storage::Storage, config::Config};

mod resp;
mod storage;
mod command_response;
mod config;
mod server;

#[tokio::main]
async fn main() {
    let config = Config::parse();
    let database = Arc::new(Mutex::new(Storage::default()));
    let redis_server = Arc::new(RedisServer::new(config, database));
    let (sender, _rx) = broadcast::channel(16);
    let sender = Arc::new(sender);

    redis_server.connect_to_master().await;

    let listener = TcpListener::bind(format!("127.0.0.1:{}", &redis_server.config.port)).await.unwrap();
    
    loop {
        let stream = listener.accept().await;
        let server = Arc::clone(&redis_server);
        let sender = Arc::clone(&sender);
        match stream {
            Ok((stream, _)) => {
                println!("Get new connection!");
                tokio::spawn(async move {
                    handle_conn(stream, server, sender).await;
                });
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}

async fn handle_conn(stream: TcpStream, redis_server: Arc<RedisServer>, sender: Arc<Sender<Value>>) {
    let mut handler = RespHandler::new(stream);

    println!("Start reading loop!");

    // let mut storage: Storage = Default::default();

    loop {
        let value = handler.read_value().await.unwrap();
        let command_propagate = value.clone().unwrap();
        let db = Arc::clone(&redis_server.database);
        
        println!("Got value {:?}", value);

        let mut if_send_rdb = false;
        let mut if_subscribe = false;

        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();
            match command.to_ascii_lowercase().as_str() {
                "ping" => Value::SimpleString("PONG".to_string()),
                "replconf" => Value::SimpleString("OK".to_string()),
                "psync" => {
                    if_subscribe = true;
                    if_send_rdb = true;
                    handle_psync()
                },
                "echo" => args.first().unwrap().clone(),
                "set" => {
                    match args.len() {
                        2 => db.lock().await.set(unpack_bulk_str(args[0].clone()).unwrap(), unpack_bulk_str(args[1].clone()).unwrap(), 0),
                        4 => db.lock().await.set(unpack_bulk_str(args[0].clone()).unwrap(), unpack_bulk_str(args[1].clone()).unwrap(), unpack_bulk_str(args[3].clone()).unwrap().parse().unwrap()),
                        _ => panic!("SET command has invalid params {}", args.len()),
                    }
                },
                "get" => {
                    db.lock().await.get(unpack_bulk_str(args[0].clone()).unwrap())
                },
                "info" => redis_server.info.get_info(),
                _ => panic!("Can not handle command {}", command),
            }
        } else { break;};

        println!("Sending value {:?}", response);

        handler.write_value(response.clone()).await.unwrap();

        if if_send_rdb {
            handler.write_rdb_file("524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2").await.unwrap();
        }
        
        if if_subscribe {
            let mut receiver = sender.subscribe();
            while let Ok(f) = receiver.recv().await {
                handler.write_value(f).await.unwrap();
            }
        }

        let _ = sender.send(command_propagate.clone());
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
