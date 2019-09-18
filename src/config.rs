pub struct Config {
    pub path: std::path::PathBuf,
    //    pub path: String,
    pub coefficient: isize,
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Self, &'static str> {
        args.next();

        let path: std::path::PathBuf = match args.next() {
            Some(arg) => std::path::PathBuf::from(arg),
            None => return Err("Didn't get a path"),
        };

        let coefficient: isize = match args.next() {
            Some(arg) => isize::from_str_radix(&arg, 10).expect("not a valid number"),
            None => return Err("Didn't get a coefficient"),
        };

        Ok(Self { path, coefficient })
    }
}
