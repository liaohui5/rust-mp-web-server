## 介绍

学习 [Rust 程序设计语言](https://rustwiki.org/zh-CN/book/ch20-00-final-project-a-web-server.html)
最后一章的构建多线程静态文件服务器项目的实现, 带有详细理解注释

但是没有原封不动的按照书上写的实现, 我添加了一些东西

## 启动

```sh
git clone git@github.com:liaohui5/rust-mp-web-server.git
cd rust-mp-web-server
cargo run
```

## 命令行参数

- `--port` 指定服务器监听的端口

- `--dir` 指定静态文件目录

## 测试

1. `/` 返回首页 index.html: 首页
2. 除了 `GET` 外的其他[请求方式](https://developer.mozilla.org/zh-CN/docs/Web/HTTP/Methods)都是返回 404.html
3. 未找到的请求路径也是返回 404.html, 如果请求的文件存在则自动读取文件内容响应
4. `/sleep` 可测试多线程, 因为线程会等待五秒后再响应数据
