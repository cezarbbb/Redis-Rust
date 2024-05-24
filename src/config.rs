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
                Some(format!("{}:{}", mport_params[0], mport_params[1]))
            },
            None => None,
        };

        // let cur_port_id = match args.iter().position(|arg| arg == "--port") {
        //     Some(index) => args.get(index + 1).unwrap(),
        //     None => "6379",
        // };
        // let (cur_port, master_port);
        // let master_port = match args.iter().position(|arg| arg == "--replicaof") {
        //     Some(index) => {
        //         let mport_params = args.get(index + 1).unwrap().split(' ').collect::<Vec<&str>>();
        //         let (host, mport) = (mport_params[0], mport_params[1]);

        //         master_port = Port::new(mport.to_string(), PortType::Master);
        //         cur_port = Port::new(cur_port_id.to_string().clone(), PortType::Slave);

        //         handshake::send_hand_shake(host.to_string(), master_port.id.clone(), cur_port.id.clone()).await;

        //         master_port
        //     },
        //     None => {
        //         master_port = Port::new(cur_port_id.to_string().clone(), PortType::Master);
        //         cur_port = Port::new(cur_port_id.to_string().clone(), PortType::Master);
        //         master_port
        //     },
        // };
        // for (index, arg) in args.iter().enumerate() {
        //     if arg == "--port" {
        //         if let Some(port) = args.get(index + 1) {
        //             config.port = port.to_owned();
        //         }
        //     }

        //     if arg == "--replicaof" {
        //         if let (Some(host), Some(port)) = (args.get(index + 1), args.get(index + 2)) {
        //             config.replicaof = Some(format!("{}:{}", host, port));
        //         }
        //     }
        // }

        config
    }
}