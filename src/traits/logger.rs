pub trait Logger {
    fn log_debug(origin: &str, message: &str);
    fn log_info(origin: &str, message: &str);
    fn log_warning(origin: &str, message: &str);
    fn log_error(origin: &str, message: &str);
    fn log_fatal(origin: &str, message: &str);
}