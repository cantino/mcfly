#[macro_use]
extern crate clap;
extern crate core;
extern crate csv;
extern crate libc;
extern crate rand;
extern crate regex;
extern crate rusqlite;
extern crate termion;
extern crate unicode_segmentation;

pub mod bash_history;
pub mod command_input;
pub mod exporter;
pub mod fake_typer;
pub mod fixed_length_grapheme_string;
pub mod history;
pub mod interface;
pub mod settings;
pub mod simplified_command;
pub mod trainer;
pub mod weights;
