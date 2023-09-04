use std::io::{self, Write};

pub fn input(prompt: &str) -> String {
    print!("{}", prompt.to_string());
    io::stdout().flush().unwrap();
    let mut result = String::new();
    io::stdin().read_line(&mut result).ok();
    return result
        .trim_start()
        .trim_end()
        .parse()
        .ok()
        .unwrap_or("".to_string());
}
