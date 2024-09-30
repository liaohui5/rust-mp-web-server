use std::sync::{mpsc, Arc, Mutex};
use std::thread;

// Job 是一个闭包, 但闭包是未知大小的类型 ?Sized
// 所以需要一个 Box 智能指针来包裹, 之后直接通过函数指针来调用
type Job = Box<dyn FnOnce() + Send + 'static>;

// 用枚举来区分, 发送的消息是一个发送新任务还是要终止线程
#[allow(unused)]
pub enum TaskMessage {
    NewTask(Job),
    Exit,
}

// 实际执行任务的对象,用 id 来标记 thread, 就可以很方便的区分不同的线程
#[allow(unused)]
pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}
impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<TaskMessage>>>) -> Worker {
        // 实例化 worker 需要传入 id 和 多线程引用计数指针
        // 指向一个 被互斥锁保护的 信道接收器对象
        // 信道对象接收 Job 类型的数据
        let thread = thread::spawn(move || {
            loop {
                // 由于 recv 只会接收一次, 所以要用循环一直不停的接收(阻塞线程)
                // 看是否有任务被发送过来, 如果有任务发送过来就需要处理
                let message = receiver.lock().unwrap().recv().unwrap();

                // 接收到任务消息过来之后还需要判断
                // 是新任务就执行,是终止消息就终止线程
                match message {
                    TaskMessage::NewTask(job) => {
                        println!("worker-{}-execute-task.", id);
                        job();
                    }
                    TaskMessage::Exit => {
                        // 终止线程: 停止接收信道对象发送的任务消息
                        // 退出 loop 循环, 那么这个线程自然就执行完了
                        println!("worker-{}-exit.", id);
                        break;
                    }
                }
            }
        });

        let thread = Some(thread);

        Worker { id, thread }
    }
}

// 线程池
// workers: 实际执行任务的线程(id+thread)
// sender:  信道对象的发送者
#[allow(unused)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<TaskMessage>,
}
impl ThreadPool {
    pub fn new(thread_count: usize) -> ThreadPool {
        assert!(thread_count > 0);
        let mut workers = Vec::with_capacity(thread_count);

        // 初始化信道
        let (sender, receiver) = mpsc::channel();

        // 由于是多线程, 为了避免数据竞争的问题, 所以需要互斥器锁来保护数据
        // 由于是多线程, 所以需要多所有权(否则只有第一个线程能获得所有权)
        let receiver = Arc::new(Mutex::new(receiver));

        // 初始化 worker 并保存, 注意传入需要引用计数智能指针
        // 让 worker 中的线程 闭包 获得所有权
        for id in 0..thread_count {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    // 为什么这样定义泛型? 因为 f 是一个跨线程传输的闭包
    // 或者说闭包类型的数据就是应该这样定义泛型约束:
    // FnOnce:  必须传入闭包
    // Send:    可以跨线程传输的类型
    // 'static: 让传入的闭包在程序运行期间都存活
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(TaskMessage::NewTask(job)).unwrap();
    }
}

// 优雅的停止服务器(停止主线程之前先关闭所有子线程)
// 如果直接 Ctrl-C 直接强行终止主线程, 那么子线程就算没有执行完也会退出
// 为 ThreadPool 实现 Drop 特性, 在主线程结束时, 会自动调用 drop 方法
// 所以可以在 drop 方法中先停止所有的子线程
impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
