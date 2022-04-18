const LOGGING: bool = false;

pub fn log(message: String) {
    if LOGGING {
        println!("{}", message);
    }
}
