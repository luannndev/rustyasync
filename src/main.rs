use std::{env, io};
use std::io::{Read, stdin};
use async_recursion::async_recursion;
use colored::{Color, Colorize};
use crate::terminal::JToolOrder;

mod information_meta;
mod terminal;
mod trace;
mod meta;
mod conan;
mod conan;
mod information_meta;
mod terminal;
mod trace;
mod meta;

#[tokio::main]
pub async fn main() -> io::Result<()> {
    let arguments: Vec<String> = env::args().collect();

    trace::Trace::info(format!("Trying to start rustyasync with {} arguments", (arguments.len()-1).to_string().color(Color::Cyan))).await;
    let result = terminal::JToolTerminal::handle_arguments(arguments).await?;
    trace::Trace::info(format!("yTool version: {0}, author: {1}", "1.0.1".cyan(), "nuralex.jig".cyan())).await;

    check_result(result).await?;

    Ok(())
}

#[async_recursion]
pub async fn check_result(result: JToolOrder) -> io::Result<()> {
    match result {
        JToolOrder::CreateProject(name, location, version, author) => {
            terminal::JToolTerminal::create_project(name, location, version, author).await?;
        }
        JToolOrder::RepairProject(location) => {
            terminal::JToolTerminal::repair_project(location).await;
        }
        JToolOrder::None => {
            let mut buf = String::new();
            let _ = stdin().read_to_string(&mut buf)?;
            buf = buf.trim_end().to_string();
            println!("{}", buf);
            if buf.eq("quit") { return Ok(()) }
            let buf: Vec<String> = buf.split(" ").collect::<Vec<_>>().iter().map(|item| item.to_string()).collect();
            let result = terminal::JToolTerminal::handle_arguments(buf).await?;
            check_result(result).await?;
        }
    }
    Ok(())
}

#[test]
pub fn test() {

}