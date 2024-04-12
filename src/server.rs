use crate::ThreadPool;
use std::{
    error::Error,
    io::Read,
    net::{TcpListener, TcpStream},
    sync::Arc,
};

pub const STATUS_OK: &str = "HTTP/1.1 200 OK";
pub const STATUS_NOT_FOUND: &str = "HTTP/1.1 404 NOT_FOUND";
pub const STATUS_INTERNAL_SERVER_ERROR: &str = "HTTP/1.1 500 INTERNAL_SERVER_ERROR";

enum Method {
    Get,
    Post,
    Delete,
    Put,
}

impl Method {
    fn to_str(&self) -> String {
        let s = match self {
            Method::Put => "PUT",
            Method::Get => "GET",
            Method::Delete => "DELETE",
            Method::Post => "POST",
        };
        String::from(s)
    }
}

type Handler = fn(TcpStream);

struct Request {
    method: Method,
    path: String,
    handler: Arc<fn(TcpStream)>,
}

impl Request {
    fn new(path: &str, method: Method, handler: fn(TcpStream)) -> Request {
        Request {
            method,
            handler: Arc::new(handler),
            path: String::from(path),
        }
    }

    fn http_str(&self) -> String {
        format!("{} {} HTTP/1.1\r\n", self.method.to_str(), self.path)
    }
}

pub struct Server {
    addr: String,
    end_point: Vec<Request>,
}

impl Server {
    pub fn new(addr: &str) -> Result<Server, Box<dyn Error>> {
        Ok(Server {
            addr: String::from(addr),
            end_point: vec![],
        })
    }

    pub fn get(&mut self, path: &str, h: Handler) -> &mut Server {
        self.end_point.push(Request::new(path, Method::Get, h));
        self
    }

    pub fn post(&mut self, path: &str, h: Handler) -> &mut Server {
        self.end_point.push(Request::new(path, Method::Post, h));
        self
    }

    pub fn delete(&mut self, path: &str, h: Handler) -> &mut Server {
        self.end_point.push(Request::new(path, Method::Delete, h));
        self
    }

    pub fn put(&mut self, path: &str, h: Handler) -> &mut Server {
        self.end_point.push(Request::new(path, Method::Put, h));
        self
    }

    pub fn run(&self, pool_size: Option<usize>) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(&self.addr)?;
        let pool = ThreadPool::new(pool_size.unwrap_or(4));
        println!("Server running...");
        println!("serving on - {}", self.addr);

        for stream in listener.incoming() {
            let mut stream = stream?;
            let mut buffer = [0; 1024];
            stream.read(&mut buffer)?;

            for ep in self.end_point.iter() {
                if buffer.starts_with(ep.http_str().as_bytes()) {
                    let f = ep.handler.clone();
                    pool.execute(move || f(stream));
                    break;
                }
            }
        }
        Ok(())
    }
}
