use tracing_subscriber::EnvFilter;

pub fn configure_logging() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("WARN"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}
