use console::{style, StyledObject};
use std::fmt::Display;

pub fn badge_ok() -> StyledObject<&'static str> {
    style("✓").green().bold()
}

pub fn badge_warn() -> StyledObject<&'static str> {
    style("!").yellow().bold()
}

pub fn badge_err() -> StyledObject<&'static str> {
    style("✗").red().bold()
}

pub fn badge_dim() -> StyledObject<&'static str> {
    style("•").dim()
}

pub fn badge_add() -> StyledObject<&'static str> {
    style("+").cyan().bold()
}

pub fn badge_fix() -> StyledObject<&'static str> {
    style("~").magenta().bold()
}

pub fn bold<T: Display>(val: T) -> StyledObject<T> {
    style(val).bold()
}

pub fn dim<T: Display>(val: T) -> StyledObject<T> {
    style(val).dim()
}

pub fn green<T: Display>(val: T) -> StyledObject<T> {
    style(val).green()
}

pub fn yellow<T: Display>(val: T) -> StyledObject<T> {
    style(val).yellow()
}

pub fn red<T: Display>(val: T) -> StyledObject<T> {
    style(val).red()
}

pub fn cyan<T: Display>(val: T) -> StyledObject<T> {
    style(val).cyan()
}

pub fn print_header(title: &str, context: Option<&str>) {
    match context {
        Some(ctx) => println!("{} · {}", style(title).bold(), style(ctx).dim()),
        None => println!("{}", style(title).bold()),
    }
}
