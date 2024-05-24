use crate::resp::Value;

// pub fn get_info(is_master: bool) -> Value {
//     let role = format!("role:{}\r\n", if is_master {"master"} else {"slave"});
//     let master_replid = "master_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb\r\n";
//     let master_repl_offset = "master_repl_offset:0\r\n";
//     Value::BulkString(role + master_replid + master_repl_offset)
// }

pub fn handle_psync() -> Value {
    Value::SimpleString("FULLRESYNC 8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb 0".to_string())
}