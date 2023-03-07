use owo_colors::OwoColorize;

pub enum Exception {
    Misc(String)
}

impl Exception {
    fn stringify(&self) -> &String {
        match self {
            Exception::Misc(s) => s
        }
    }

    pub fn log_and_ignore(&self) {
        println!("[{}] {}", "WARN".yellow(), self.stringify());
    }

    pub fn log_and_exit(&self) {
        println!("[{}] {} Aborted.", "ERR".red(), self.stringify());
        std::process::exit(1);
    }
}