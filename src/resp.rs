use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use bytes::BytesMut;
use anyhow::Result;

#[derive(Clone, Debug)]
pub enum Value {
    SimpleString(String),
    BulkString(String),
    Array(Vec<Value>),
    Null,
}

impl Value {
    pub fn serialize(self) -> String {
        match self {
            Value::SimpleString(s) => format!("+{}\r\n", s),
            Value::BulkString(s) => format!("${}\r\n{}\r\n", s.chars().count(), s),
            Value::Array(s) => {
                let mut sentence = format!("*{}\r\n", s.len());
                for item in s {
                    sentence += item.serialize().as_str();
                }
                sentence
            },
            Value::Null => format!("$-1\r\n"),
            _ => panic!("Unsupported value for serialize!"),
        }
    }
}

pub struct RespHandler {
    stream: TcpStream,
    buffer: BytesMut,
}

impl RespHandler {
    pub fn new(stream: TcpStream) -> Self {
        RespHandler {
            stream,
            buffer: BytesMut::with_capacity(512),
        }
    }
    pub async fn read_value(&mut self) -> Result<Option<Value>> {
        let read_count = self.stream.read_buf(&mut self.buffer).await?;
        if read_count == 0 { return Ok(None);}
        let (v, _) = parse_message(self.buffer.split())?;
        return Ok(Some(v));
    }

    pub async fn write_value(&mut self, value: Value) -> Result<()> {
        self.stream.write(value.serialize().as_bytes()).await?;
        Ok(())
    }
}

fn parse_message(buffer: BytesMut) -> Result<(Value, usize)> {
    match buffer[0] {
        b'+' => parse_simple_string(buffer),
        b'$' => parse_bulk_string(buffer),
        b'*' => parse_array(buffer),
        _ => Err(anyhow::anyhow!("Not a known value type {:?}", buffer)),
    }
}

fn parse_simple_string(buffer: BytesMut) -> Result<(Value, usize)> {
    if let Some((line, len)) = read_until_clrf(&buffer[1..]) {
        let string = String::from_utf8(line.to_vec()).unwrap();
        return Ok((Value::SimpleString(string), len + 1));
    };
    return Err(anyhow::anyhow!("Invalid simple string {:?}", buffer));
}

fn parse_bulk_string(buffer: BytesMut) -> Result<(Value, usize)> {
    let (bulk_str_len, bytes_consumed) = if let Some((line, len)) = read_until_clrf(&buffer[1..]) {
        let bulk_str_len = parse_int(line)?;
        (bulk_str_len, len + 1)
    } else {
        return Err(anyhow::anyhow!("Invalid bulk string {:?}", buffer));
    };
    let end_of_bulk_str = bytes_consumed + bulk_str_len as usize;
    let total_parsed = end_of_bulk_str + 2;
    Ok((Value::BulkString(String::from_utf8(buffer[bytes_consumed..end_of_bulk_str].to_vec())?), total_parsed))
}

fn parse_array(buffer: BytesMut) -> Result<(Value, usize)> {
    let (array_len, mut bytes_consumed) = if let Some((line, len)) = read_until_clrf(&buffer[1..]) {
        let array_len = parse_int(line)?;
        (array_len, len + 1)
    } else {
        return Err(anyhow::anyhow!("Invalid array format {:?}", buffer));
    };
    let mut items = vec![];
    for _ in 0..array_len {
        let (array_item, len) = parse_message(BytesMut::from(&buffer[bytes_consumed..]))?;
        items.push(array_item);
        bytes_consumed += len;
    }
    Ok((Value::Array(items), bytes_consumed))
}

fn read_until_clrf(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buffer.len() {
        if buffer[i - 1] == b'\r' && buffer[i] == b'\n' {
            return Some((&buffer[0..(i - 1)], i + 1));
        }
    }
    return None;
}

fn parse_int(buffer: &[u8]) -> Result<i64> {
    Ok(String::from_utf8(buffer.to_vec())?.parse::<i64>()?)
}