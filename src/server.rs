use std::sync::Arc;

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use tokio::sync::Mutex;

use crate::resp::Value;
use crate::storage::Storage;
use crate::Config;

#[derive(PartialEq, Eq, Debug)]
pub enum Role {
    Master,
    Slave,
}
pub struct Info {
    pub role: Role,
    pub master_replid: String,
    pub master_repl_offset: i8,
}

impl Info {
    pub fn new(config: &Config) -> Self {
        Info {
            role: match config.replicaof {
                Some(_) => Role::Slave,
                None => Role::Master,
            },
            master_replid: String::from("8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb"),
            master_repl_offset: 0,
        }
    }
    pub fn get_info(&self) -> Value {
        let role = format!("role:{}\r\n", match self.role {
            Role::Master => "master",
            Role::Slave => "slave",
        });
        let master_replid = format!("master_replid:{}\r\n", self.master_replid);
        let master_repl_offset = format!("master_repl_offset:{}\r\n", self.master_repl_offset);
        Value::BulkString(role + master_replid.as_str() + master_repl_offset.as_str())
    }
}

pub struct RedisServer {
    pub info: Info,
    pub config: Config,
    pub database: Arc<Mutex<Storage>>,
}

impl RedisServer {
    pub fn new(config: Config, database: Arc<Mutex<Storage>>) -> Self {
        RedisServer {
            info: Info::new(&config),
            config: config,
            database: database,
        }
    }

    pub async fn connect_to_master(&self) {
        println!("Role of port: {:?}", self.info.role);
        if self.info.role == Role::Slave {
            if let Some(replicaof) = &self.config.replicaof {
                    Self::send_hand_shake(replicaof, &self.config.port).await;
                }
        };
    }

    pub async fn send_hand_shake(replicaof: &str, cur_port_id: &str){
        println!("Start handshake with master port!");
                
        let mut hand_shake = TcpStream::connect(format!("{}", replicaof)).await.expect("Unable to connect master port");
    
        let hs_1 = Value::Array(vec![Value::BulkString("PING".to_string())]);
        hand_shake.write(hs_1.serialize().as_bytes()).await.expect("Handshake 1 failed");
        let mut ping_response_buffer = [0; 1024];
        hand_shake.read(&mut ping_response_buffer).await.expect("Failed to receive ping response when handshaking");
    
        let hs_2_1 = Value::Array(vec![Value::BulkString("REPLCONF".to_string()), Value::BulkString("listening-port".to_string()), Value::BulkString(format!("{}", cur_port_id).to_string())]);
        hand_shake.write(hs_2_1.serialize().as_bytes()).await.expect("Handshake 1/2 failed");
        let mut replconf_listening_port_response = [0; 1024];
        hand_shake.read(&mut replconf_listening_port_response).await.expect("Failed to receive response ");
    
        let hs_2_2 = Value::Array(vec![Value::BulkString("REPLCONF".to_string()), Value::BulkString("capa".to_string()), Value::BulkString("psync2".to_string())]);
        hand_shake.write(hs_2_2.serialize().as_bytes()).await.expect("Handshake 2/2 failed");
        let mut replconf_capa_response = [0; 1024];
        hand_shake.read(&mut replconf_capa_response).await.expect("Failed to receive capa responose when handshaking");
    
        let hs_3 = Value::Array(vec![Value::BulkString("PSYNC".to_string()), Value::BulkString("?".to_string()), Value::BulkString("-1".to_string())]);
        hand_shake.write(hs_3.serialize().as_bytes()).await.expect("Handshake 3 failed");
        let mut replconf_psync_response = [0; 1024];
        hand_shake.read(&mut replconf_psync_response).await.expect("Failed to receive psync responose when handshaking");
    }
}