use std::{
    collections::HashMap,
    io::Read,
    net::{TcpListener, TcpStream},
    string::FromUtf8Error,
};

struct Parser {
    header: String,
}
impl Parser {
    fn new(header: String) -> Self {
        Self { header }
    }

    fn parse_headers(&self) -> HashMap<String, String> {
        let mut hash_header = HashMap::new();
        let lines = self.header.lines().skip(1);
        for header in lines {
            if let Some((key, value)) = header.split_once(":") {
                hash_header.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        hash_header
    }

    fn parse_request_line(&self) -> (String, String) {
        let mut method_url_version = self.header.split_ascii_whitespace();
        method_url_version.next().expect("HTTP Method not found");
        if let Some(url) = method_url_version.next() {
            if let Some(version) = method_url_version.next() {
                return (url.trim().to_string(), version.trim().to_string());
            } else {
                return (url.trim().to_string(), "".to_string());
            };
        }
        panic!("URL or Version not parsed correctly");
    }

    fn parse_http_method(&self) -> HttpMethod {
        match self.header.split_ascii_whitespace().next() {
            Some(method) => match method {
                "GET" => HttpMethod::Get,
                "POST" => HttpMethod::Post,
                _ => unimplemented!(),
            },
            None => panic!("HTTP Method Not found"),
        }
    }
}
#[derive(Debug, Clone, Copy)]
enum HttpMethod {
    Get,
    Post,
}

#[derive(Debug, Clone)]
struct HttpRequest {
    method: HttpMethod,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HttpRequest {
    fn new(
        method: HttpMethod,
        path: String,
        version: String,
        headers: HashMap<String, String>,
    ) -> Self {
        Self {
            method,
            path,
            version,
            headers,
            body: Vec::new(),
        }
    }

    fn set_body(&mut self, body: Vec<u8>) {
        self.body = body;
    }
}
fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        match handle_client_http_request(&stream?) {
            Ok(msg) => {
                dbg!(msg);
            }
            Err(_) => {
                eprint!("Something went Wrong reading the stream");
            }
        }
    }

    Ok(())
}

fn handle_client_http_request(mut stream: &TcpStream) -> Result<HttpRequest, FromUtf8Error> {
    //bytes raw
    let mut buf: Vec<u8> = Vec::new();
    let position_body_start: usize;

    let delimiter = [13u8, 10, 13, 10];

    let mut http_request: HttpRequest;
    loop {
        let mut tmp = [0u8; 1024];
        let n = stream.read(&mut tmp).unwrap();
        //logic: I populate a tmp buffer then i add it to the actual one
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = buf.windows(4).position(|w| w == delimiter) {
            //I have the position that gives me the start of body (+4 to do)
            // I have all the headers
            let parser = Parser::new(
                std::str::from_utf8(&buf[..pos])
                    .expect("Parsing Header not in the UTF8 range")
                    .to_string(),
            );
            let request_type: HttpMethod = parser.parse_http_method();
            let (path, version): (String, String) = parser.parse_request_line();
            let request_headers: HashMap<String, String> = parser.parse_headers();

            position_body_start = pos;
            http_request = HttpRequest::new(request_type, path, version, request_headers);
            break;
        }
    }
    let content: usize = match http_request.headers.get("Content-Length") {
        Some(len) => len
            .parse()
            .expect("The Content-Length value is not parsable to number"),
        None => return Ok(http_request),
    };
    //Logic: i read from the buffer from the point i know the body start. i read until the lenght
    //of the body is the one of Content-Length
    let mut body: Vec<u8> = Vec::with_capacity(content);
    body.extend_from_slice(&buf[position_body_start + 4..]);
    while body.len() < content {
        let mut tmp = [0u8; 1024];
        let n = stream.read(&mut tmp).unwrap();
        body.extend_from_slice(&tmp[..n]);
    }
    http_request.set_body(body);
    Ok(http_request)
}
