use std::{env, thread};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::str::FromStr;
use std::process;
use std::sync::mpsc::{channel, Sender};
use std::io::{self, Write};
use std::time::Duration;

const MAX_THREAD: u16 = 65535;

#[derive(Debug)]
struct Arguments {
    ipaddr: IpAddr,
    threads: u16,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Err("Too few arguments");
        } else if args.len() > 4 {
            return Err("Too many arguments");
        }

        let f = args[1].clone();
        return if let Ok(ip) = IpAddr::from_str(&f) {
            Ok(ArgumentsBuilder::default().ipaddr(ip).build())
        } else {
            if f.contains("-h") || f.contains("-help") {
                Self::print_help();
                Err("help")
            } else if f.contains("-t") {
                if args.len() < 3 {
                    Self::print_help();
                    return Err("Too few arguments");
                }
                if let Ok(threads_number) = args[2].parse::<u16>() {
                    if args.len() < 4 {
                        Self::print_help();
                        return Err("Too few arguments, missing ipaddr");
                    }
                    let ipaddr = match IpAddr::from_str(&args[3]) {
                        Ok(s) => s,
                        Err(_) => return Err("ipaddr invalid, ipv4 or ipv6")
                    };
                    Ok(ArgumentsBuilder::default().ipaddr(ipaddr).threads(threads_number).build())
                } else {
                    Err("invalid thread number")
                }
            } else {
                Self::print_help();
                Err("coucou")
            }
        }
    }

    fn print_help() {
        println!("Usage: port-sniffer xxx.xxx.xxx.xxx");
        println!("Options:");
        println!("\t-t -> number of threads to use");
        println!("\t-h -> show this help message");
    }
}

struct ArgumentsBuilder {
    ipaddr: IpAddr,
    threads: u16,
}

impl ArgumentsBuilder {
    pub fn ipaddr(mut self, ipaddr: IpAddr) -> ArgumentsBuilder {
        self.ipaddr = ipaddr;
        self
    }
    pub fn threads(mut self, threads: u16) -> ArgumentsBuilder {
        self.threads = threads;
        self
    }

    pub fn build(self) -> Arguments {
        Arguments {
            ipaddr: self.ipaddr,
            threads: self.threads,
        }
    }
}

impl Default for ArgumentsBuilder {
    fn default() -> Self {
        ArgumentsBuilder {
            ipaddr: IpAddr::from_str("127.0.0.1").unwrap(),
            threads: 0,
        }
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let arguments = Arguments::new(&args).unwrap_or_else(
        |err| {
            if err.contains("help") {
                process::exit(0);
            } else {
                eprintln!("{}", err);
                process::exit(1);
            }
        }
    );

    let (tx, rx) = channel();

    for i in 0..arguments.threads {
        let tx = tx.clone();
        thread::spawn(move || {
            scan(tx, i, arguments.ipaddr, arguments.threads);
        });
    }

    let mut open_ports = vec![];
    drop(tx);
    for p in rx {
        open_ports.push(p)
    }

    open_ports.sort();

    println!();
    for p in open_ports {
        println!("port {} is open", p);
    }
}

fn scan(sender: Sender<u16>, start_port: u16, ipaddr: IpAddr, total_threads: u16) {
    let mut port = start_port +1;

    loop {
        match TcpStream::connect_timeout(&SocketAddr::new(ipaddr, port), Duration::new(2, 0)) {
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                sender.send(port).unwrap();
            },
            Err(_) => {}
        }

        if (MAX_THREAD - port) <= total_threads {
            break;
        }

        port += total_threads;
    }
}