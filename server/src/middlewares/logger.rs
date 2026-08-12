use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use once_cell::sync::Lazy;
use tokio::{sync::mpsc, time::Instant};

#[derive(Debug)]
struct LogEntry {
    method: String,
    path: String,
    status: u16,
    latency: u64,
}

/// 日志通道, 该通道用于发送日志信息
static LOG_CHANNEL: Lazy<mpsc::Sender<LogEntry>> = Lazy::new(|| {
    let (tx, rx) = mpsc::channel::<LogEntry>(100_000);
    spawn_log_processor(rx);
    tx
});

fn spawn_log_processor(mut rx: mpsc::Receiver<LogEntry>) {
    // 使用线程而非tokio spawn
    // tokio线程用于处理短暂的非阻塞的任务, 如果使用tokio spawn, 会导致整个异步运行时的性能
    std::thread::Builder::new()
        .name("log-processor".to_string())
        .spawn(move || {
            loop {
                match rx.blocking_recv() {
                    Some(LogEntry {
                        method,
                        path,
                        status,
                        latency,
                    }) => {
                        tracing::info!(
                            "Request completed: {} {} status={} latency={:?}ms",
                            method,
                            path,
                            status,
                            latency
                        );
                    }
                    _ => continue,
                }
            }
        })
        .expect("Failed to spawn log processor thread");
}

pub async fn logger(req: Request, next: Next) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    // 处理请求
    let response = next.run(req).await;

    let status = response.status().as_u16();

    let latency = start.elapsed().as_millis() as u64;
    let _ = LOG_CHANNEL.try_send(LogEntry {
        method,
        path,
        status,
        latency,
    });
    Ok(response)
}
