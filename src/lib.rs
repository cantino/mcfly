#[macro_use]
extern crate clap;
extern crate rusqlite;
extern crate termion;
extern crate unicode_segmentation;
extern crate core;
extern crate libc;
extern crate regex;
extern crate rand;
extern crate csv;

pub mod history;
pub mod settings;
pub mod weights;
pub mod bash_history;
pub mod interface;
pub mod fake_typer;
pub mod command_input;
pub mod trainer;
pub mod exporter;
pub mod fixed_length_grapheme_string;
pub mod simplified_command;
