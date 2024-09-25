use std::str::FromStr;
use std::time::Duration;
use std::{ fs, thread };
use std::io::{ Read, Write, Result as IOResult };
use std::net::{ Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream };
use std::sync::Arc;

use crate::thread_pool::ThreadPool;
use crate::config::Config;

pub fn listen(config: Config) -> IOResult<()> {
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), config.port));
    let pool = ThreadPool::new(5);

    // config 只读的, 没有数据竞争问题, 所以不用互斥锁保护数据
    let config = Arc::new(config);

    print!("\x1b[1m\x1b[31m"); // 输出红色的调试信息
    println!("server started on http://{}", addr);
    print!("\x1b[22m\x1b[39m"); // 输出红色的调试信息

    // TcpListener::bind 会阻塞主线程
    let listener = TcpListener::bind(addr)?;
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let config_arc = Arc::clone(&config);

        // 使用线程池处理请求
        pool.execute(move || {
            handle_request(stream, config_arc);
        });
    }

    Ok(())
}

fn handle_request(mut stream: TcpStream, config: Arc<Config>) {
    // 注意 buffer 是一个 u8 数组 [u8; 1024], 不是一个 Vec<u8>
    let mut buffer = [0; 1024];
    Read::read(&mut stream, &mut buffer).unwrap();

    let http_req_str = String::from_utf8(buffer.to_vec()).unwrap();

    // 请求字符串
    // println!("{:?}", http_req_str);

    print!("\x1b[1m\x1b[31m"); // 输出红色的调试信息
    let (status_line, file_path) = get_response(&http_req_str, config);
    let contents = fs::read_to_string(file_path).unwrap();
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    // 响应字符串
    // println!("{:?}", response);

    print!("\x1b[22m\x1b[39m"); // 输出红色的调试信息
    Write::write_all(&mut stream, response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn get_response(http_req_str: &str, config: Arc<Config>) -> (String, String) {
    let public_path = config.path.to_string();

    // 解析请求的方法,路径,http协议版本等信息
    let (method, req_path, protocol_version) = parse_http_info(http_req_str);
    println!("request-info:{method},{req_path},{protocol_version}");

    // 处理一些特殊的请求路径
    let route_path = if req_path == "/" { // 映射首页文件
        "/index.html"
    } else if req_path == "/sleep.html" { // 测试多线程
        thread::sleep(Duration::from_secs(5));
        req_path
    } else { // 其他情况
        req_path
    };

    // 客户端请求的文件路径, 需要加上 public_path
    let mut req_file_path = public_path.clone();
    req_file_path.push_str(route_path);

    // 响应 header 信息: HTTP/1.1 200 OK\r\nxxx
    let mut status_line = String::from_str(protocol_version).unwrap();
    let mut res_file_path = req_file_path;

    // 由于这是个简单的静态文件服务器,所以:
    // 不是 GET 请求或者文件不存在都应该返回 404
    if method != "GET" || fs::metadata(res_file_path.as_str()).is_err(){
        let mut _404_file_path = public_path.clone();
        _404_file_path.push_str("/404.html");

        // HTTP/1.1 404 NOT FOUND
        status_line.push_str(" 404 NOT FOUND");
        res_file_path = _404_file_path;
    } else {
        status_line.push_str(" 200 OK");
    }

    (status_line, res_file_path)
}

fn parse_http_info(http_req_str: &str) -> (&str, &str, &str) {
    // print!("\x1b[1m \x1b[31m");
    let http_info = http_req_str.lines().next().unwrap();
    let methods = ["GET", "HEAD", "POST", "PUT", "DELETE", "CONNECT", "OPTIONS", "TRACE", "PATCH"];

    // parse request method
    let mut http_method = "GET";
    for item in methods.iter() {
        if let Some(_index) = http_info.find(item) {
            http_method = item;
        }
    }

    // parse protocol version
    let mut http_version = "HTTP/1.1";
    if let Some(start_index) = http_info.rfind("HTTP/") {
        http_version = &http_info[start_index..http_info.len()];
    }

    // parse request path
    let req_path_start = http_method.len() + 1;
    let req_path_end = http_info.len() - http_version.len() - 1;
    let req_path = &http_info[req_path_start..req_path_end];

    // print!("\x1b[22m \x1b[39m");
    (http_method, req_path, http_version)
}


#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn should_be_parse_http_method() {
        let http_req_str = String::from("GET / HTTP/1.1\r\n");
        let (http_method, _, _) = parse_http_info(&http_req_str);
        assert_eq!(http_method, "GET");

        let http_req_str = String::from("POST / HTTP/1.1\r\n");
        let (http_method, _, _) = parse_http_info(&http_req_str);
        assert_eq!(http_method, "POST");

        let http_req_str = String::from("DELETE / HTTP/1.1\r\n");
        let (http_method, _, _) = parse_http_info(&http_req_str);
        assert_eq!(http_method, "DELETE");
    }

    #[test]
    fn should_be_parse_http_version() {
        let http_req_str = String::from("GET / HTTP/1.1\r\n");
        let (_, _, http_version) = parse_http_info(&http_req_str);
        assert_eq!(http_version, "HTTP/1.1");

        let http_req_str = String::from("POST / HTTP/2\r\n");
        let (_, _, http_version) = parse_http_info(&http_req_str);
        assert_eq!(http_version, "HTTP/2");

        let http_req_str = String::from("POST / HTTP/2.0\r\n");
        let (_, _, http_version) = parse_http_info(&http_req_str);
        assert_eq!(http_version, "HTTP/2.0");
    }


    #[test]
    fn should_be_parse_request_path() {
        let http_req_str = String::from("GET / HTTP/1.1\r\n");
        let (_, request_path, _) = parse_http_info(&http_req_str);
        assert_eq!(request_path, "/");

        let http_req_str = String::from("GET /test.html HTTP/1.1\r\n");
        let (_, request_path, _) = parse_http_info(&http_req_str);
        assert_eq!(request_path, "/test.html");

        let http_req_str = String::from("GET /test.html?id=1 HTTP/1.1\r\n");
        let (_, request_path, _) = parse_http_info(&http_req_str);
        assert_eq!(request_path, "/test.html?id=1");
    }
}

