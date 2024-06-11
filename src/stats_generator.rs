use std::cmp::min;
use std::ops::Add;

use regex::Regex;
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
    pub fn generate_stats(&self, limit: i16, command_limit: Option<i16>) -> String {
        let mut lines = "".to_owned();
        let count_history = Self::count_commands_from_db_history(self);
        if count_history == 0 {
            return "No history, no stats!".to_string();
        }
        lines.push_str("📊 Quick stats:\n");
        lines.push_str(format!("  - history has {:?} items ;\n", count_history).as_mut_str());
        let most_used_commands = self.most_used_commands(&limit);
        lines.push_str(&Self::generate_command_stats(
            self,
            limit,
            command_limit,
            most_used_commands,
        ));
        lines
    }

    fn generate_command_stats(
        &self,
        limit: i16,
        command_limit: Option<i16>,
        stats: Vec<StatItem>,
    ) -> String {
        let mut lines = "".to_owned();
        if !stats.is_empty() {
            lines.push_str(
                format!(
                    "  - {:?} first commands, sorted by occurrences:\n",
                    min(limit, stats.len() as i16)
                )
                .as_mut_str(),
            );
            let re = Regex::new("^(.*)/(.*)$").unwrap();
            for item in &stats[..min(limit as usize, stats.len())] {
                let cmd = item.clone().cmd;
                if !cmd.trim().is_empty() {
                    if command_limit.is_some() && cmd.len() < command_limit.unwrap() as usize {
                        continue;
                    }
                    let relative_cmd = re.captures(&cmd).map(|dir_and_cmd| {
                        format!(
                            "{0} ({1})",
                            dir_and_cmd.get(2).unwrap().as_str(),
                            dir_and_cmd.get(1).unwrap().as_str()
                        )
                    });
                    lines = lines.add(&*format!(
                        "    {:#} ({:?})\n",
                        relative_cmd.unwrap_or(cmd),
                        item.count
                    ));
                }
            }
        }
        lines
    }

    fn most_used_commands(&self, limit: &i16) -> Vec<StatItem> {
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
        vec.first().unwrap().count
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::history::History;
    use crate::network::Network;
    use crate::stats_generator::StatItem;

    #[test]
    fn empty_history() {
        let history = History {
            connection: Connection::open_in_memory().unwrap(),
            network: Network::random(),
        };
        let stats_generator = crate::stats_generator::StatsGenerator::new(&history);
        let lines = stats_generator.generate_command_stats(10, None, Vec::new());
        assert_eq!(lines, "");
    }

    #[test]
    fn partial_history() {
        let history = History {
            connection: Connection::open_in_memory().unwrap(),
            network: Network::random(),
        };
        let stats_generator = crate::stats_generator::StatsGenerator::new(&history);
        let lines = stats_generator.generate_command_stats(
            3,
            None,
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
    fn full_history_with_simple_commands() {
        let history = History {
            connection: Connection::open_in_memory().unwrap(),
            network: Network::random(),
        };
        let stats_generator = crate::stats_generator::StatsGenerator::new(&history);
        let lines = stats_generator.generate_command_stats(
            10,
            None,
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
    fn history_with_relative_and_full_path_commands() {
        let history = History {
            connection: Connection::open_in_memory().unwrap(),
            network: Network::random(),
        };
        let stats_generator = crate::stats_generator::StatsGenerator::new(&history);
        let lines = stats_generator.generate_command_stats(
            10,
            None,
            Vec::from([
                StatItem {
                    cmd: "./bin/docker".to_string(),
                    count: 10,
                },
                StatItem {
                    cmd: "/opt/local/share/docker".to_string(),
                    count: 9,
                },
            ]),
        );
        assert_eq!(
            lines,
            "  - 2 first commands, sorted by occurrences:\n    docker (./bin) (10)\n    docker (/opt/local/share) (9)\n"
        );
    }

    #[test]
    fn command_stats_can_be_filtered_by_command_size() {
        let history = History {
            connection: Connection::open_in_memory().unwrap(),
            network: Network::random(),
        };
        let stats_generator = crate::stats_generator::StatsGenerator::new(&history);
        let lines = stats_generator.generate_command_stats(
            10,
            Option::from(3),
            Vec::from([
                StatItem {
                    cmd: "a".to_string(),
                    count: 1,
                },
                StatItem {
                    cmd: "be".to_string(),
                    count: 2,
                },
                StatItem {
                    cmd: "bee".to_string(),
                    count: 3,
                },
                StatItem {
                    cmd: "bees".to_string(),
                    count: 4,
                },
            ]),
        );
        assert_eq!(
            lines,
            "  - 4 first commands, sorted by occurrences:\n    bee (3)\n    bees (4)\n"
        );
    }

    #[test]
    fn command_stats_can_be_limited() {
        let history = History {
            connection: Connection::open_in_memory().unwrap(),
            network: Network::random(),
        };
        let stats_generator = crate::stats_generator::StatsGenerator::new(&history);
        let lines = stats_generator.generate_command_stats(
            2,
            None,
            Vec::from([
                StatItem {
                    cmd: "a".to_string(),
                    count: 1,
                },
                StatItem {
                    cmd: "be".to_string(),
                    count: 2,
                },
            ]),
        );
        assert_eq!(
            lines,
            "  - 2 first commands, sorted by occurrences:\n    a (1)\n    be (2)\n"
        );
    }
}
