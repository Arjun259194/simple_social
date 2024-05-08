use crate::ThreadPool;
use std::{
    error::Error,
    fmt::Display,
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
};
use termion::color;

pub const STATUS_OK: &str = "HTTP/1.1 200 OK";
pub const STATUS_NOT_FOUND: &str = "HTTP/1.1 404 NOT_FOUND";
pub const STATUS_INTERNAL_SERVER_ERROR: &str = "HTTP/1.1 500 INTERNAL_SERVER_ERROR";

#[derive(Clone, Copy)]
enum Method {
    Get,
    Post,
    Delete,
    Put,
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Method::Put => "PUT",
            Method::Get => "GET",
            Method::Delete => "DELETE",
            Method::Post => "POST",
        };
        write!(f, "{}", s)
    }
}

type HandlerFn = fn(TcpStream);

#[derive(Clone)]
struct Handler {
    method: Method,
    path: String,
    handler: Arc<HandlerFn>,
}

impl Display for Handler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c_method = format!("{}{}", color::Fg(color::Blue), self.method);
        let c_path = format!("{}{}", color::Fg(color::Green), self.path);
        write!(f, "{} {} {}", c_method, c_path, color::Fg(color::White))
    }
}

impl Handler {
    fn new(path: &str, method: Method, handler: fn(TcpStream)) -> Handler {
        Handler {
            method,
            handler: Arc::new(handler),
            path: String::from(path),
        }
    }

    fn http_str(&self) -> String {
        format!("{} {} HTTP/1.1\r\n", self.method, self.path)
    }

    fn check(&self, buffer: &[u8; 1024]) -> bool {
        buffer.starts_with(self.http_str().as_bytes())
    }
}

pub trait RequestHandler {
    fn get(&mut self, path: &str, h: HandlerFn) -> &mut Self;

    fn post(&mut self, path: &str, h: HandlerFn) -> &mut Self;

    fn put(&mut self, path: &str, h: HandlerFn) -> &mut Self;

    fn delete(&mut self, path: &str, h: HandlerFn) -> &mut Self;
}

#[derive(Clone)]
struct Route {
    method: Method,
    path: String,
    handler: HandlerFn,
}

impl Route {
    fn new(path: &str, method: Method, h: HandlerFn) -> Route {
        Route {
            method,
            path: String::from(path),
            handler: h,
        }
    }
}

pub struct Router {
    end_points: Vec<Route>,
}

impl Router {
    pub fn new() -> Router {
        Router { end_points: vec![] }
    }
}

impl RequestHandler for Router {
    fn get(&mut self, path: &str, h: HandlerFn) -> &mut Self {
        self.end_points.push(Route::new(path, Method::Get, h));
        self
    }

    fn post(&mut self, path: &str, h: HandlerFn) -> &mut Self {
        self.end_points.push(Route::new(path, Method::Post, h));
        self
    }

    fn put(&mut self, path: &str, h: HandlerFn) -> &mut Self {
        self.end_points.push(Route::new(path, Method::Put, h));
        self
    }

    fn delete(&mut self, path: &str, h: HandlerFn) -> &mut Self {
        self.end_points.push(Route::new(path, Method::Delete, h));
        self
    }
}

pub struct Server {
    addr: String,
    end_points: Vec<Handler>,
    pool_size: usize,
}

impl RequestHandler for Server {
    fn get(&mut self, path: &str, h: HandlerFn) -> &mut Self {
        self.end_points.push(Handler::new(path, Method::Get, h));
        self
    }

    fn post(&mut self, path: &str, h: HandlerFn) -> &mut Self {
        self.end_points.push(Handler::new(path, Method::Post, h));
        self
    }

    fn delete(&mut self, path: &str, h: HandlerFn) -> &mut Self {
        self.end_points.push(Handler::new(path, Method::Delete, h));
        self
    }

    fn put(&mut self, path: &str, h: HandlerFn) -> &mut Self {
        self.end_points.push(Handler::new(path, Method::Put, h));
        self
    }
}

impl Server {
    pub fn new(addr: &str, pool_size: usize) -> Server {
        Server {
            addr: String::from(addr),
            end_points: Vec::new(),
            pool_size: pool_size.max(2),
        }
    }

    pub fn mount(&mut self, path: &str, router: Router) -> &mut Self {
        let super_paths: Vec<_> = path.split("/").collect();
        for end_point in router.end_points.iter() {
            let base_paths: Vec<_> = end_point.path.split("/").collect();
            let paths = [&super_paths[..], &base_paths[..]].concat();
            let paths: Vec<_> = paths.into_iter().filter(|s| !s.is_empty()).collect();
            let path = "/".to_owned() + &paths.join("/");
            let handler_fn = end_point.handler.clone();

            self.end_points
                .push(Handler::new(&path, end_point.method, handler_fn));
        }
        self
    }

    fn log(&self) -> Result<(), Box<dyn Error>> {
        clearscreen::clear()?;
        println!("Server running...\n");
        for ep in self.end_points.iter() {
            println!("{ep}");
        }
        println!("\nserving on - {}", self.addr);
        Ok(())
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(&self.addr)?;
        let pool = ThreadPool::new(self.pool_size);
        self.log()?;
        for stream in listener.incoming() {
            let mut stream = stream?;
            let mut buffer = [0; 1024];
            stream.read(&mut buffer)?;

            if let Some(ep) = self.end_points.iter().find(|&x| x.check(&buffer)) {
                let f = ep.handler.clone();
                pool.execute(move || f(stream));
            } else {
                let content = fs::read_to_string("static/404.html")?;

                let res = format!(
                    "{}\r\nContent=Length: {}\r\n\r\n{}",
                    STATUS_NOT_FOUND,
                    content.len(),
                    content
                );

                stream.write(res.as_bytes())?;
                stream.flush()?;
            }
        }
        Ok(())
    }
}
