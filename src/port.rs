use tokio::{net::TcpStream, io::AsyncWriteExt};
use crate::resp::Value;

#[derive(Clone, Copy, Debug)]
pub enum PortType {
    Master,
    Slave,
}

#[derive(Clone, Debug)]
pub struct Port {
    pub id: String,
    pub port_type: PortType,
}

impl Port {
    pub fn new(id: String, port_type: PortType) -> Port{
        Port {
            id: id,
            port_type: port_type,
        }
    }
}

// pub struct Config {
//     pub master_port: Port,
//     pub slave_ports: Vec<Port>,
//     pub host: String,
// }

// impl Config {
//     pub fn new(master_port: Port, slave_ports: Vec<Port>, host: String) -> Config{
//         Config {
//             master_port: master_port,
//             slave_ports: slave_ports,
//             host: host,
//         }
//     }
// }

pub async fn send_hand_shake(host: String, master_port_id: String, cur_port_id: String) {
    println!("Start handshake with master port!");
            
    let mut hand_shake = TcpStream::connect(format!("{}:{}", host, master_port_id)).await.expect("Unable to connect master port");

    let hs_1 = Value::Array(vec![Value::BulkString("PING".to_string())]);
    
    hand_shake.write(hs_1.serialize().as_bytes()).await.expect("Handshake 1 failed");
    hand_shake.flush().await.unwrap();

    hand_shake.write(format!("*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n$4\r\n{}\r\n", cur_port_id).as_bytes()).await.expect("Handshake 1/2 failed");
    hand_shake.flush().await.unwrap();

    hand_shake.write(b"*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n").await.expect("Handshake 2/2 failed");
    hand_shake.flush().await.unwrap();
}