use chrono::Local;
use colored::{Color, Colorize};
use colored::Color::BrightWhite;

pub struct Trace;

impl Trace {
    pub async fn info<T>(output: T) where T: Into<String> {
        let output = output.into();
        let now = Local::now().format("%H:%M:%S").to_string();
        let format = format!("{0} {1} {2} {3} {4}{5}",
                             now.color(BrightWhite),
                             "|".color(Color::BrightBlack),
                             "INFO".color(Color::BrightGreen),
                             "»".color(Color::BrightBlack),
                             output.color(BrightWhite),
                             ".".color(Color::BrightBlack)
        );
        println!("{}", format)

    }

    pub async fn error<T>(output: T) where T: Into<String> {
        let output = output.into();
        let now = Local::now().format("%H:%M:%S").to_string();
        let format = format!("{0} {1} {2} {3} {4}{5}",
                             now.color(BrightWhite),
                             "|".color(Color::BrightBlack),
                             "ERROR".color(Color::BrightRed),
                             "»".color(Color::BrightBlack),
                             output.color(BrightWhite),
                             ".".color(Color::BrightBlack)
        );
        println!("{}", format)

    }
}