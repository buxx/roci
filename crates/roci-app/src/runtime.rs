#[macro_export]
macro_rules! prepare_runtime {
    () => {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name("my-custom-name")
            .thread_stack_size(3 * 1024 * 1024)
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        let handle = runtime.handle();
        let _handle = handle.enter();
    };
}
