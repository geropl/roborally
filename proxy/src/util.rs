use anyhow;

use slog::Drain;

pub fn to_anyhow(kube_err: kube::Error) -> anyhow::Error {
    anyhow!("{}", kube_err)
}

pub fn create_logger() -> slog::Logger {
    let drain = slog_json::Json::new(std::io::stdout())
        .add_default_keys()
        .build()
        .fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    slog::Logger::root(drain, o!())
}