use simple_social::server::*;
use std::{fs, io::Write, process, thread::sleep, time::Duration};

const POOL_SIZE: usize = 4;

fn main() {
    let mut server = Server::new("127.0.0.1:3000", POOL_SIZE);

    server.get("/", |mut stream| {
        let content = fs::read_to_string("static/index.html").unwrap();

        let res = format!(
            "{}\r\nContent=Length: {}\r\n\r\n{}",
            STATUS_OK,
            content.len(),
            content
        );

        stream.write(res.as_bytes()).unwrap();
        stream.flush().unwrap();
    });

    server.get("/user", |mut stream| {
        sleep(Duration::from_secs(5));
        let content = fs::read_to_string("static/user.html").unwrap();
        let res = format!(
            "{}\r\nContent=Length: {}\r\n\r\n{}",
            STATUS_OK,
            content.len(),
            content
        );

        stream.write(res.as_bytes()).unwrap();
        stream.flush().unwrap();
    });

    server.get("/jsondata", |mut stream| {
        let content = String::from(
            r#"{
            "name":"Arjun",
            "age": 20,
            "email":"arjun259194@gmail.com"
        }"#,
        );

        //TODO not working
        let response = format!(
            "{}Content-Length: {}\r\n\r\nContent-Type: application/json\r\n\r\n{}",
            STATUS_OK,
            content.len(),
            content
        );

        println!("{response}");

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    });

    if let Err(e) = server.run() {
        eprintln!("Application error: {:?}", e);
        process::exit(1);
    }
}
