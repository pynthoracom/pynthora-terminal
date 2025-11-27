use colored::*;
use std::fmt::Display;

pub struct Logger;

impl Logger {
    pub fn info<T: Display>(message: T) {
        println!("{} {}", "ℹ".blue(), message);
    }

    pub fn success<T: Display>(message: T) {
        println!("{} {}", "✓".green(), message);
    }

    pub fn warn<T: Display>(message: T) {
        eprintln!("{} {}", "⚠".yellow(), message);
    }

    pub fn error<T: Display>(message: T) {
        eprintln!("{} {}", "✗".red(), message);
    }

    pub fn debug<T: Display>(message: T) {
        eprintln!("{} {}", "🔍".cyan(), message);
    }
}

