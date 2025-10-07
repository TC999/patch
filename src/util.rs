pub fn fatal(msg: &str) -> ! {
    eprintln!("fatal: {}", msg);
    std::process::exit(2);
}

pub fn say(msg: &str) {
    println!("{}", msg);
}