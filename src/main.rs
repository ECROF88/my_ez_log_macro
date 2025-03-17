use my_ez_log_macro::{
    log,
    log::{LogLevel, shutdown_logging},
};

fn main() {
    my_ez_log_macro::log::init_logging("my_application.log");
    log!(LogLevel::Info, "System initialized");

    // 动态格式化（延迟转换）
    let user = "Alice";
    let attempts = 3;
    log!(LogLevel::Info, "User {} login attempts: {}", user, attempts);
    for i in 0..20 {
        log!(
            LogLevel::Info,
            "count is  {} login attempts: {}",
            i,
            attempts
        );
    }
    // 数值延迟转换示例
    let temperature = 25.5;
    log!(LogLevel::Debug, "Temp: {:.2}°C", temperature);

    std::thread::sleep(std::time::Duration::from_millis(1000));
    shutdown_logging();
}
