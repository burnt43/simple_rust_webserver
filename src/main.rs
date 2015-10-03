extern crate time;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::prelude::*;
use std::fs::{OpenOptions};
use std::fmt;
use std::collections::{HashMap};

enum LogLevel {
    Info,
    Error,
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum HttpVersion {
    V1_0,
    V1_1,
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum HttpVerb {
    Get,
}

#[derive(Clone)]
enum HttpResponseCode {
    Ok,
    BadRequest,
    NotFound,
}

#[derive(PartialEq, Eq, Hash, Clone)]
enum HttpOption {
    ContentType,
    Date,
    Server,
}

impl fmt::Display for HttpOption {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}", match *self {
            HttpOption::ContentType => "Content-Type",
            HttpOption::Date        => "Date",
            HttpOption::Server      => "Server",
        })
    }
}

struct HttpMessage {
    http_verb:    Option<HttpVerb>,
    http_version: Option<HttpVersion>,
    request_path: Option<String>,
    raw_message:  String,
}

struct HttpMessageParser {
    buffer: String,
}

struct HttpResponse {
    http_response_code: HttpResponseCode,
    http_version:       HttpVersion,
    http_options:       HashMap<HttpOption,String>,
    body:               String,
}

struct HttpResponseBuilder {
    http_response_code: HttpResponseCode,
    http_version:       HttpVersion,
    http_options:       HashMap<HttpOption,String>,
    body:               String,
}

impl HttpResponseBuilder {
    fn new () -> HttpResponseBuilder {
        let mut http_options: HashMap<HttpOption,String> = HashMap::new();
        http_options.insert(HttpOption::Date,HttpResponse::current_timestamp());
        http_options.insert(HttpOption::Server,"simple_rust_webserver (https://github.com/burnt43/simple_rust_webserver)".to_string());
        http_options.insert(HttpOption::ContentType,"text/html; charset=utf-8".to_string());

        HttpResponseBuilder {
            http_response_code: HttpResponseCode::BadRequest,
            http_version:       HttpVersion::V1_0,
            http_options:       http_options,
            body:               "<html><body><h1>400 Bad Request</h1></body></html>\n".to_string(),
        }
    }
    fn http_response_code (&mut self, http_response_code: HttpResponseCode) -> &mut HttpResponseBuilder {
        self.http_response_code = http_response_code;
        self
    }
    fn http_version (&mut self, http_version: HttpVersion) -> &mut HttpResponseBuilder {
        self.http_version = http_version;
        self
    }
    fn http_options (&mut self, pair: (HttpOption,String)) -> &mut HttpResponseBuilder {
        self.http_options.insert(pair.0,pair.1);
        self
    }
    fn body (&mut self, body: &str) -> &mut HttpResponseBuilder {
        self.body = body.to_string();
        self
    }
    fn finalize (&self) -> HttpResponse {
        HttpResponse {
            http_response_code: self.http_response_code.clone(),
            http_version:       self.http_version.clone(),
            http_options:       self.http_options.clone(),
            body:               self.body.clone(),
        }
    }
}

impl HttpResponse {
    fn current_timestamp () -> String {
        time::strftime("%a, %d %b %Y %H:%M:%S GMT",&time::now_utc()).unwrap()
    }
    fn as_string(&self) -> String {
        let mut result: String = String::new();
        match self.http_version {
            HttpVersion::V1_0 => result.push_str( "HTTP/1.0" ),
            HttpVersion::V1_1 => result.push_str( "HTTP/1.1" ),
        }
        result.push_str(" ");
        match self.http_response_code {
            HttpResponseCode::Ok         => result.push_str( "200 OK" ),
            HttpResponseCode::NotFound   => result.push_str( "404 Not Found" ),
            HttpResponseCode::BadRequest => result.push_str( "400 Bad Request" ),
        }
        result.push_str("\r\n");
        for (http_option,value) in &self.http_options {
            result.push_str( &format!("{}: {}\r\n", http_option,value) )
        }
        result.push_str( &format!("Content-Length: {}\r\n\r\n{}",self.body.len(),self.body) );
        result
    }
}

impl HttpMessage {
    fn http_verb_from_str( s: &str ) -> Option<HttpVerb> {
        match &*s.to_uppercase() {
            "GET" => Some(HttpVerb::Get),
            _     => None,
        }
    }
    fn http_version_from_str( s: &str ) -> Option<HttpVersion> {
        match &*s.to_uppercase() {
            "HTTP/1.0" => Some(HttpVersion::V1_0),
            "HTTP/1.1" => Some(HttpVersion::V1_1),
            _          => None,
        }
    }
    fn create_from_str( s: &str ) -> HttpMessage {
        let mut lines:             Vec<&str> = s.lines().collect();
        let     request_line:      &str      = lines.remove(0);
        let     request_line_info: Vec<&str> = request_line.split_whitespace().collect();
        
        if request_line_info.len() == 3 {
            let (http_verb_str, request_path_str, http_version_str) = (request_line_info[0],request_line_info[1],request_line_info[2]);
            HttpMessage {
                http_verb:    HttpMessage::http_verb_from_str( http_verb_str ),
                http_version: HttpMessage::http_version_from_str( http_version_str ),
                request_path: Some( request_path_str.to_string() ),
                raw_message:  s.to_string(),
            }
        } else {
            HttpMessage {
                http_verb:    None,
                http_version: None,
                request_path: None,
                raw_message:  s.to_string(),
            }
        }
    }
    fn process( &self ) -> HttpResponse {
        match ( self.http_verb.clone(), self.http_version.clone(), self.request_path.clone() ) {
            ( Some(http_verb), Some(http_version), Some(request_path) ) => {
                if http_verb == HttpVerb::Get && &request_path == "/" {
                    HttpResponseBuilder::new()
                        .http_response_code(HttpResponseCode::Ok)
                        .http_version(HttpVersion::V1_1)
                        .body("<html><body><h1>200 OK</h1></body></html>\n")
                        .finalize()
                } else {
                    HttpResponseBuilder::new()
                        .http_response_code(HttpResponseCode::NotFound)
                        .http_version(HttpVersion::V1_1)
                        .body("<html><body><h1>404 Not Found</h1></body></html>\n")
                        .finalize()
                }
            },
            ( _, _, _ ) => {
                HttpResponseBuilder::new()
                    .finalize()
            },
        }
    }
}

impl fmt::Display for HttpMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.raw_message)
    }
}

impl fmt::Debug for HttpMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"HttpMessage\nhttp_verb: {:?}\nhttp_version: {:?}\nrequest_path: {:?}\nraw_message: {}\n\n",
               self.http_verb,
               self.http_version,
               self.request_path,
               self.raw_message)
    }
}

impl HttpMessageParser {
    fn new() -> HttpMessageParser {
        HttpMessageParser{ buffer: String::new() }
    }
    fn push_bytes( &mut self, bytes: &[u8] ) -> Vec<HttpMessage> {
        let mut temp_buffer:   String           = self.buffer.clone();
        let mut http_messages: Vec<HttpMessage> = Vec::new(); 
        let bytes_as_str:      &str             = std::str::from_utf8(bytes).unwrap();

        temp_buffer.push_str(bytes_as_str);

        let mut message_strings: Vec<&str> = temp_buffer.split("\r\n\r\n").collect();

        if message_strings.len() > 1 {
            let remainder: &str = message_strings.pop().unwrap();
            for message_string in message_strings {
                http_messages.push( HttpMessage::create_from_str( message_string) );
            }
            self.buffer = remainder.to_string();
        } else {
            self.buffer = temp_buffer.clone();
        }
        http_messages
    }
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
    let mut http_message_parser: HttpMessageParser = HttpMessageParser::new();
    let read_slice:              &mut[u8]          = &mut[0;512];
    let _                                          = stream.set_read_timeout(None);

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
                for http_message in http_message_parser.push_bytes( data ) {
                    write_to_log_file( LogLevel::Info, &format!("Received from {}\n{:?}",ip,http_message) );
                    let http_response:   HttpResponse = http_message.process();
                    let response_string: String       = http_response.as_string();
                    write_to_log_file( LogLevel::Info, &format!("Responding\n{}",response_string) );
                    let _ = write!(stream,"{}",response_string);
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
