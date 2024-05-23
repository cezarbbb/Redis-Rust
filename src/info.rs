use crate::resp::Value;

pub fn get_info(is_master: bool) -> Value {
    let mut info = vec![];
    let role = Value::BulkString(format!("role:{}", if is_master {"master"} else {"slave"}).to_string());
    let master_replid = Value::BulkString("master_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string());
    let master_repl_offset = Value::BulkString("master_repl_offset:0".to_string());
    info.push(role);
    info.push(master_replid);
    info.push(master_repl_offset);
    Value::Array(info)
}