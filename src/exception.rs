pub enum Exception {
    Misc(String)
}

impl Exception {
    fn stringify(&self) -> &String {
        match self {
            Exception::Misc(s) => s
        }
    } 
}