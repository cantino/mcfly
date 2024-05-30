use std::cmp::min;
use std::ops::Add;

use serde::Serialize;

use crate::history::History;

#[derive(Debug)]
pub struct StatsGenerator<'a> {
    history: &'a History,
}

#[derive(Debug, Clone, Serialize)]
struct StatItem {
    cmd: String,
    count: i64,
}

impl<'a> StatsGenerator<'a> {
    pub fn generate_stats(&self, limit: i32) -> String {
        let mut lines = "".to_owned();
        let count_history = Self::count_commands_from_db_history(self);
        if count_history == 0 {
            return "No history, no stats!".to_string();
        }
        lines.push_str(format!("📊 Quick stats:\n").as_mut_str());
        lines.push_str(format!("  - history has {:?} items ;\n", count_history).as_mut_str());
        let most_used_commands = self.most_used_commands(&limit);
        lines.push_str(&*Self::generate_command_stats(
            self,
            limit,
            most_used_commands,
        ));
        lines
    }

    fn generate_command_stats(&self, limit: i32, stats: Vec<StatItem>) -> String {
        let mut lines = "".to_owned();
        if !stats.is_empty() {
            lines.push_str(
                format!(
                    "  - {:?} first commands, sorted by occurrences:\n",
                    min(limit, stats.len() as i32)
                )
                .as_mut_str(),
            );
            for item in stats {
                if !item.cmd.trim().is_empty() {
                    lines = lines.add(&*format!("    {:#} ({:?})\n", item.cmd, item.count));
                }
            }
        }
        lines
    }

    fn most_used_commands(&self, limit: &i32) -> Vec<StatItem> {
        self
            .history
            .run_query("select substr(cmd,1,instr(cmd,' ')-1), count(1) as n from commands group by 1 order by 2 desc limit :limit ", &[
                (":limit", &limit.to_owned()),
            ], |row| {
                Ok(StatItem {
                    cmd: row.get(0)?,
                    count: row.get(1)?,
                })
            })
    }

    #[inline]
    pub fn new(history: &'a History) -> Self {
        Self { history }
    }
    fn count_commands_from_db_history(&self) -> i32 {
        struct Count {
            count: i32,
        }
        let vec = self
            .history
            .run_query("select count(1) as n from commands", &[], |row| {
                Ok(Count { count: row.get(0)? })
            });
        vec.get(0).unwrap().count
    }
}

#[cfg(test)]
mod tests {
    use crate::history::History;
    use crate::settings::HistoryFormat;
    use crate::stats_generator::StatItem;

    #[test]
    fn empty_history() {
        let history = History::load(HistoryFormat::Bash);
        let stats_generator = crate::stats_generator::StatsGenerator::new(&history);
        let lines = stats_generator.generate_command_stats(10, Vec::new());
        assert_eq!(lines, "");
    }

    #[test]
    fn partial_history() {
        let history = History::load(HistoryFormat::Bash);
        let stats_generator = crate::stats_generator::StatsGenerator::new(&history);
        let lines = stats_generator.generate_command_stats(
            3,
            Vec::from([
                StatItem {
                    cmd: "git".to_string(),
                    count: 10,
                },
                StatItem {
                    cmd: "cargo".to_string(),
                    count: 9,
                },
            ]),
        );
        assert_eq!(
            lines,
            "  - 2 first commands, sorted by occurrences:\n    git (10)\n    cargo (9)\n"
        );
    }

    #[test]
    fn full_history() {
        let history = History::load(HistoryFormat::Bash);
        let stats_generator = crate::stats_generator::StatsGenerator::new(&history);
        let lines = stats_generator.generate_command_stats(
            2,
            Vec::from([
                StatItem {
                    cmd: "git".to_string(),
                    count: 10,
                },
                StatItem {
                    cmd: "cargo".to_string(),
                    count: 9,
                },
            ]),
        );
        assert_eq!(
            lines,
            "  - 2 first commands, sorted by occurrences:\n    git (10)\n    cargo (9)\n"
        );
    }
}
