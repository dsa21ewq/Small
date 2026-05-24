use console::style;

pub fn success(msg: &str) {
    println!("{} {}", style("✓").green().bold(), msg);
}

pub fn error(msg: &str) {
    eprintln!("{} {}", style("✗").red().bold(), msg);
}

pub fn info(msg: &str) {
    println!("  {}", style(msg).dim());
}

pub fn step(msg: &str) {
    println!("{} {}", style("⏳").yellow(), msg);
}
