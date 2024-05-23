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