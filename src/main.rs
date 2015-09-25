extern crate time;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::prelude::*;
use std::fs::{OpenOptions};

enum LogLevel {
    Info,
    Error,
}

fn write_to_log_file(level: LogLevel, message: &str) {
    let mut log_file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open("/var/log/simple_rust_webserver.log").unwrap();
    let prefix = match level {
        LogLevel::Info => "[INFO]",
        LogLevel::Error => "[ERROR]",
    };
    let _ = writeln!(log_file,"({}) {}: {}\n",time::now().strftime("%Y-%m-%d %H:%M:%S").unwrap(),prefix,message);
}

fn client_connection(mut stream: TcpStream) {
    let mut buffer = String::new();
    let read_slice = &mut[0;512];
    let _          = stream.set_read_timeout(None);

    let ip = match stream.peer_addr() {
        Ok(socket_addr) => {
            match socket_addr {
                std::net::SocketAddr::V4(v4) => format!("{}",v4.ip()),
                std::net::SocketAddr::V6(v6) => format!("{}",v6.ip()),
            }
        }
        Err(_) => "unknown".to_string(),
    };

    loop {
        match stream.read(read_slice) {
            Ok(0) => {
                write_to_log_file(LogLevel::Info,"Read 0 Bytes. Closing Socket.");
                break;
            }
            Ok(bytes_read) => {
                let ( data, _ ) = read_slice.split_at(bytes_read);
                let text = std::str::from_utf8(data).unwrap();
                buffer.push_str(text);
                let temp = buffer.clone();
                let mut messages:Vec<&str> = temp.split("\r\n\r\n").collect();
                if messages.len() > 1 {
                    let remainder = messages.pop().unwrap();
                    for message in messages {
                        write_to_log_file(LogLevel::Info,&format!("message from {}\n{}",ip,message));
                        let lines:Vec<&str> = message.lines().collect();
                        if lines[0] == "GET / HTTP/1.1" {
                            let response_body = "<html><body><h1>OK</h1></body></html>";
                            let _ = write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",response_body.len(),response_body);
                        } else {
                            let response_body = "<html><body><h1>404</h1></body></html>";
                            let _ = write!(stream,"HTTP/1.1 404 Not Found\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",response_body.len(),response_body);
                        }
                    }
                    buffer = remainder.to_string();
                }
            },
            Err(_) => {
                write_to_log_file(LogLevel::Error,"Error Reading Socket. Ending Read Loop");
                break;
            },
        }
    }
}

fn main() {
    let listener = TcpListener::bind("104.236.40.97:80").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    client_connection(stream);
                });
            },
            Err(e) => panic!("error: {}",e),
        }
    }
    drop(listener);
}
