use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use crate::resp::Value;

pub async fn send_hand_shake(host: String, master_port_id: String, cur_port_id: String) {
    println!("Start handshake with master port!");
            
    let mut hand_shake = TcpStream::connect(format!("{}:{}", host, cur_port_id)).await.expect("Unable to connect master port");

    let hs_1 = Value::Array(vec![Value::BulkString("PING".to_string())]);
    hand_shake.write(hs_1.serialize().as_bytes()).await.expect("Handshake 1 failed");
    let mut ping_response_buffer = [0; 1024];
    hand_shake.read(&mut ping_response_buffer).await.expect("Failed to receive ping response when handshaking");

    let hs_2_1 = Value::Array(vec![Value::BulkString("REPLCONF".to_string()), Value::BulkString("listening-port".to_string()), Value::BulkString(format!("{}", master_port_id).to_string())]);
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