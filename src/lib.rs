use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
// use std::prelude::*;

pub struct Server {
    routes: Vec<Route>,
}

impl Server {
    pub fn new() -> Self {
        Server { routes: Vec::new() }
    }

    pub fn use_route(&mut self, route: Route) -> &mut Self {
        self.routes.push(route);
        self
    }

    pub fn listen(self, address: &str) {
        let listener =
            TcpListener::bind(address).expect(format!("cannot bind address {}", address).as_str());
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => Server::dispatch(stream, &self.routes),
                Err(error) => {
                    eprintln!("error receiving stream, {error}");
                }
            }
        }
    }

    fn dispatch(stream: TcpStream, routes: &Vec<Route>) {
        let (request, stream) = Server::parse(stream);

        let mut response = Response::new(stream);

        let request = match request {
            Ok(request) => request,
            Err(code) => {
                response
                    .status(code)
                    .send(Response::status_text_from_code(code).as_str());
                return;
            }
        };

        for route in routes {
            if let Ordering::Equal = route.path.cmp(&request.path) {
                match request.method {
                    Method::GET => Server::verify_hanlder(&route.get, request, response),
                    Method::POST => Server::verify_hanlder(&route.post, request, response),
                    Method::PUT => Server::verify_hanlder(&route.put, request, response),
                    Method::DELETE => Server::verify_hanlder(&route.delete, request, response),
                    Method::PATCH => Server::verify_hanlder(&route.patch, request, response),
                }
                break;
            }
        }
    }

    fn verify_hanlder(handler: &Option<Handler>, request: Request, mut response: Response) {
        match handler {
            None => {
                response.status(405).send("method not implemented");
            }
            Some(handler) => {
                match handler {
                    Handler::Sync(handler) => handler(request, response),
                    // TODO: Handler::Async
                }
            }
        }
    }

    fn parse(mut stream: TcpStream) -> (Result<Request, u16>, TcpStream) {
        let mut buffer: Vec<u8> = Vec::new();
        loop {
            let mut chunk = [0; 1024];
            match stream.read(&mut chunk) {
                Ok(0) => {
                    break;
                }
                Ok(1024) => {
                    buffer.extend_from_slice(&chunk);
                }
                Ok(size) => {
                    buffer.extend_from_slice(&chunk[..size]);
                    break;
                }
                Err(_) => {
                    return (Err(400), stream);
                }
            }
        }
        let raw_request = String::from_utf8_lossy(&buffer[..]).to_string();

        let mut lines = raw_request.lines();

        let (method, original_path) = match lines.nth(0) {
            None => return (Err(400), stream),
            Some(line1) => {
                let line1_data: Vec<&str> = line1.split_whitespace().collect();
                match (line1_data.get(0), line1_data.get(1)) {
                    (Some(method), Some(path)) => (*method, String::from(*path)),
                    _ => return (Err(400), stream),
                }
            }
        };

        let method = match Request::parse_method(method) {
            None => {
                return (Err(405), stream);
            }
            Some(method) => method,
        };

        let mut query = HashMap::new();
        
        let path = if let Some((path, queries)) = original_path.split_once("?") {
            for q in queries.split("&") {
                if let Some((k, v)) = q.split_once("=") {
                    query.insert(String::from(k), String::from(v));
                }
            }
            path.to_string()
        } else { original_path.clone() };

        let mut headers = HashMap::new();

        for line in lines.skip(1) {
            if line.is_empty() {
                break;
            }
            let header: Vec<&str> = line.splitn(2, ": ").collect();
            match (header.get(0), header.get(1)) {
                (Some(key), Some(value)) => {
                    headers.insert(String::from(*key), String::from(*value));
                }
                _ => return (Err(400), stream),
            }
        }

        let body = if let Some(index) = raw_request.find("\r\n\r\n") {
            String::from(&raw_request[index + 4..])
        } else {
            String::new()
        };

        let request = Request::new(method, path, original_path, query, headers, body);

        (Ok(request), stream)
    }
}

#[derive(Debug)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

pub struct Route {
    path: &'static str,
    get: Option<Handler>,
    post: Option<Handler>,
    put: Option<Handler>,
    delete: Option<Handler>,
    patch: Option<Handler>,
}

type SyncHandler = fn(req: Request, res: Response);
// find a way to store async function
// type AsyncHandler = fn(req: Request, res: Response) -> dyn Future<Output = ()>;

enum Handler {
    Sync(SyncHandler),
    // Async(AsyncHandler),
}

impl Route {
    pub fn new(path: &'static str) -> Self {
        Self {
            path,
            get: None,
            post: None,
            put: None,
            delete: None,
            patch: None,
        }
    }
    pub fn get(&mut self, handler: SyncHandler) -> &mut Self {
        self.get = Some(Handler::Sync(handler));
        self
    }
    // pub fn get_async(&mut self, handler: AsyncHandler) -> &mut Self {
    //     self.get = Some(Handler::Async(handler));
    //     self
    // }
    pub fn post(&mut self, handler: SyncHandler) -> &mut Self {
        self.post = Some(Handler::Sync(handler));
        self
    }
    // pub fn post_async(&mut self, handler: AsyncHandler) -> &mut Self {
    //     self.post = Some(Handler::Async(handler));
    //     self
    // }
    pub fn put(&mut self, handler: SyncHandler) -> &mut Self {
        self.put = Some(Handler::Sync(handler));
        self
    }
    // pub fn put_async(&mut self, handler: AsyncHandler) -> &mut Self {
    //     self.put = Some(Handler::Async(handler));
    //     self
    // }
    pub fn delete(&mut self, handler: SyncHandler) -> &mut Self {
        self.delete = Some(Handler::Sync(handler));
        self
    }
    // pub fn delete_async(&mut self, handler: AsyncHandler) -> &mut Self {
    //     self.delete = Some(Handler::Async(handler));
    //     self
    // }
    pub fn patch(&mut self, handler: SyncHandler) -> &mut Self {
        self.patch = Some(Handler::Sync(handler));
        self
    }
    // pub fn patch_async(&mut self, handler: AsyncHandler) -> &mut Self {
    //     self.patch = Some(Handler::Async(handler));
    //     self
    // }
}

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub original_path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Request {
    pub fn new(
        method: Method,
        path: String,
        original_path: String,
        query: HashMap<String, String>,
        headers: HashMap<String, String>,
        body: String,
    ) -> Self {
        Self {
            method,
            path,
            original_path,
            query,
            headers,
            body,
        }
    }
    fn parse_method(method: &str) -> Option<Method> {
        let method = match method {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "PATCH" => Method::PATCH,
            _ => {
                return None;
            }
        };
        Some(method)
    }
}
#[derive(Debug)]
pub struct Response {
    stream: TcpStream,
    status: u16,
    headers: HashMap<String, String>,
}

impl Response {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            status: 200,
            headers: HashMap::new(),
        }
    }

    fn status_text_from_code(code: u16) -> String {
        match code {
            200 => "Ok",
            201 => "Created",
            204 => "No Content",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not allowed",
            500 => "Internal Server Error",
            502 => "Bad Gateway",
            503 => "Server Unavailable",
            code if code < 400 => "Success",
            code if code < 500 => "Client Error",
            _ => "Server Error",
        }
        .to_string()
    }

    pub fn status(&mut self, code: u16) -> &mut Self {
        self.status = code;
        self
    }
    pub fn header(&mut self, key: String, value: String) -> &mut Self {
        self.headers.insert(key, value);
        self
    }
    pub fn send(&mut self, body: &str) {
        self.stream
            .write(
                format!(
                    "HTTP/1.1 {} {}\r\nContent-Type: text/plain\r\nContent-Length: {}{}\r\n\r\n{}",
                    self.status,
                    Response::status_text_from_code(self.status),
                    body.len(),
                    self.headers
                        .iter()
                        .map(|(name, value)| format!("{}: {}\r\n", name, value))
                        .collect::<String>(),
                    body
                )
                .as_bytes(),
            )
            .expect("cannot write into response");
    }
}
