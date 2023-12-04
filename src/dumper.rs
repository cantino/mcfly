use std::io::{self, BufWriter, Write};

use crate::cli::DumpFormat;
use crate::history::{DumpCommand, History};
use crate::settings::Settings;
use crate::time::to_datetime;

#[derive(Debug)]
pub struct Dumper<'a> {
    settings: &'a Settings,
    history: &'a History,
}

impl<'a> Dumper<'a> {
    #[inline]
    pub fn new(settings: &'a Settings, history: &'a History) -> Self {
        Self { settings, history }
    }

    pub fn dump(&self) {
        let mut commands = self
            .history
            .dump(&self.settings.time_range, &self.settings.sort_order);
        if commands.is_empty() {
            println!("McFly: No history");
            return;
        }

        if let Some(pat) = &self.settings.pattern {
            commands.retain(|dc| pat.is_match(&dc.cmd));
        }

        match self.settings.dump_format {
            DumpFormat::Json => Self::dump2json(&commands),
            DumpFormat::Csv => Self::dump2csv(&commands),
        }
        .unwrap_or_else(|err| panic!("McFly error: Failed while output history ({err})"));
    }
}

impl<'a> Dumper<'a> {
    fn dump2json(commands: &[DumpCommand]) -> io::Result<()> {
        let mut stdout = BufWriter::new(io::stdout().lock());
        serde_json::to_writer_pretty(&mut stdout, commands).map_err(io::Error::from)?;
        stdout.flush()
    }

    fn dump2csv(commands: &[DumpCommand]) -> io::Result<()> {
        let mut wtr = csv::Writer::from_writer(io::stdout().lock());
        wtr.write_record(["cmd", "when_run"])?;
        for dc in commands {
            wtr.write_record([dc.cmd.as_str(), to_datetime(dc.when_run).as_str()])?;
        }
        wtr.flush()
    }
}
