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
        config.port = match args.iter().position(|arg| arg == "--port") {
            Some(index) => args.get(index + 1).unwrap().to_string(),
            None => "6379".to_string(),
        };
        config.replicaof = match args.iter().position(|arg| arg == "--replicaof") {
            Some(index) => {
                let mport_params = args.get(index + 1).unwrap().split(' ').collect::<Vec<&str>>();
                let host = if mport_params[0] == "localhost" {
                    "127.0.0.1"
                } else {
                    mport_params[0]
                };
                Some(format!("{}:{}", host, mport_params[1]))
            },
            None => None,
        };

        config
    }
}