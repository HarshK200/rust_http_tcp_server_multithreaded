use http_server::ThreadPool;
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    process::exit,
};

pub fn start_server() {
    let listner_creation_result = TcpListener::bind("127.0.0.1:42069");
    let tcp_listener: TcpListener;

    // NOTE:  creating tcp listener
    match listner_creation_result {
        Ok(listener) => {
            tcp_listener = listener;
            println!(
                "tcp listener local address: {}",
                tcp_listener.local_addr().unwrap()
            );
        }
        Err(e) => {
            println!(
                "Some error occured during tcp listener creation\nError: {}",
                e
            );
            exit(1);
        }
    }

    // NOTE: creating a threadpool for multithreaded code execution
    let thread_pool = ThreadPool::new(4).unwrap();

    // NOTE: listener started listening for TCP streams on port 3000
    for stream in tcp_listener.incoming() {
        let tcp_stream = stream.unwrap();

        // WARN: the tcp_stream is moved to the handler function hence no data duplication here
        let _execution_result =  thread_pool.execute(|| {
            handle_connection(tcp_stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    println!("request_line: {}", request_line);

    // HACK: pattern matching &str hence we do &request_line[..]
    let (status_line, file_name) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 gotcha homie", "pages/hello.html"),
        "GET /sleep HTTP/1.1" => {
            std::thread::sleep(std::time::Duration::from_secs(5));
            ("HTTP/1.1 200 gotcha homie", "pages/hello.html")
        }
        _ => ("HTTP/1.1 404 couldn't find that sh*t", "pages/404.html"),
    };

    let readfile_result = std::fs::read_to_string(file_name);
    let content: String;
    match readfile_result {
        Ok(x) => content = x,
        Err(e) => {
            println!("Cannot read the file\nError: {}", e);
            return;
        }
    }
    let content_length = content.len();

    let response = format!("{status_line}\r\nContent-Length: {content_length}\r\n\r\n{content}");
    let result = stream.write_all(response.as_bytes());
    match result {
        Ok(_) => {
            println!("Response sent successfully to client!");
        }
        Err(e) => {
            println!("error occured during sending response\nError: {}", e)
        }
    }
}
