// #[derive(Clone, Copy, Debug)]
// pub enum PortType {
//     Master,
//     Slave,
// }

// #[derive(Clone, Debug)]
// pub struct Port {
//     pub id: String,
//     pub port_type: PortType,
// }

// impl Port {
//     pub fn new(id: String, port_type: PortType) -> Port{
//         Port {
//             id: id,
//             port_type: port_type,
//         }
//     }
// }

use std::env;
pub struct Config {
    pub port: String,
    pub replicaof: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            port: String::from("6379"),
            replicaof: None,
        }
    }

    pub fn parse() -> Self {
        let args = env::args().collect::<Vec<String>>();
        let mut config = Config::new();
        for (index, arg) in args.iter().enumerate() {
            if arg == "--port" {
                if let Some(port) = args.get(index + 1) {
                    config.port = port.to_owned();
                }
            }

            if arg == "--replicaof" {
                if let (Some(host), Some(port)) = (args.get(index + 1), args.get(index + 2)) {
                    config.replicaof = Some(format!("{}:{}", host, port));
                }
            }
        }

        config
    }
}