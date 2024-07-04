use std::cmp::min;
use std::collections::HashMap;

use serde::Serialize;

use crate::history::History;
use crate::settings::Settings;

#[derive(Debug)]
pub struct StatsGenerator<'a> {
    history: &'a History,
}

#[derive(Debug, Clone, Serialize)]
struct StatItem {
    cmd_tpl: String,
    count: i64,
    dir: Option<String>,
}

impl<'a> StatsGenerator<'a> {
    #[must_use]
    pub fn generate_stats(&self, settings: &Settings) -> String {
        let mut lines = String::new();
        let count_history = Self::count_commands_from_db_history(self, &None);
        if count_history == 0 {
            return "No history found in the database".to_string();
        }
        lines.push_str("📊 Quick stats:\n");
        if settings.stats_only_dir.is_some() {
            lines.push_str(
                format!(
                    "  - your history database contains {:?} items total and {:?} in {:?}\n",
                    count_history,
                    Self::count_commands_from_db_history(self, &settings.stats_only_dir),
                    &settings.stats_only_dir.as_ref().unwrap()
                )
                .as_mut_str(),
            );
        } else {
            lines.push_str(
                format!("  - your history database contains {count_history:?} items\n")
                    .as_mut_str(),
            );
        }
        let most_used_commands = self.most_used_commands(
            settings.stats_cmds,
            settings.stats_min_cmd_length,
            settings.stats_dirs,
            settings.stats_global_commands_to_ignore,
            &settings.stats_only_dir,
        );
        lines.push_str(&Self::generate_command_stats(
            self,
            settings.stats_cmds,
            most_used_commands,
        ));
        lines
    }

    fn generate_command_stats(&self, cmds: i16, stats: Vec<StatItem>) -> String {
        let mut lines = String::new();
        let mut directory_map: HashMap<Option<String>, Vec<&StatItem>> = HashMap::new();

        // Group stats by directory
        for item in &stats {
            directory_map
                .entry(item.dir.clone())
                .or_default()
                .push(item);
        }

        for (dir, items) in &directory_map {
            if let Some(dir_name) = dir {
                lines.push_str(&format!(
                    "  - top {:?} matching commands in directory {:?}, sorted by occurrence:\n",
                    min(cmds, items.len() as i16),
                    dir_name
                ));
            } else {
                lines.push_str(&format!(
                    "  - top {:?} matching commands, sorted by occurrence:\n",
                    min(cmds, items.len() as i16)
                ));
            }

            for item in &items[..min(cmds as usize, items.len())] {
                lines.push_str(&format!("    {} ({})\n", item.cmd_tpl, item.count));
            }
        }

        if lines.contains("QUOTED") || lines.contains("PATH") {
            lines.push_str("  - (QUOTED and PATH indicate portions of a command that were removed for grouping)\n");
        }

        lines
    }

    fn most_used_commands(
        &self,
        cmds: i16,
        min_cmd_length: i16,
        dirs: i16,
        global_commands_to_ignore: i16,
        only_dir: &Option<String>,
    ) -> Vec<StatItem> {
        if dirs > 0 || only_dir.is_some() {
            let query = "
            WITH DirectoryCounts AS (
                SELECT
                    dir,
                    COUNT(*) AS cmd_count
                FROM
                    commands
                WHERE
                    length(cmd_tpl) >= :min_cmd_length
                    AND (:dir_filter_off OR dir = :only_dir)
                GROUP BY
                    dir
                ORDER BY
                    cmd_count DESC
                LIMIT
                    MAX(1, :dirs)
            ),
            TopGlobalCommands AS (
                SELECT
                    cmd_tpl,
                    COUNT(*) AS cmd_occurrence
                FROM
                    commands
                WHERE
                    length(cmd_tpl) >= :min_cmd_length
                GROUP BY
                    cmd_tpl
                ORDER BY
                    cmd_occurrence DESC
                LIMIT
                    :global_commands_to_ignore
            ),
            TopCommands AS (
                SELECT
                    dir,
                    cmd_tpl,
                    COUNT(*) AS cmd_occurrence,
                    ROW_NUMBER() OVER (PARTITION BY dir ORDER BY COUNT(*) DESC) AS row_num
                FROM
                    commands
                WHERE
                    dir IN (SELECT dir FROM DirectoryCounts)
                    AND cmd_tpl NOT IN (SELECT cmd_tpl FROM TopGlobalCommands)
                    AND length(cmd_tpl) >= :min_cmd_length
                GROUP BY
                    dir, cmd_tpl
            )
            SELECT
                dir,
                cmd_tpl,
                cmd_occurrence
            FROM
                TopCommands
            WHERE
                row_num <= :cmds
            ORDER BY
                dir, cmd_occurrence DESC
        ";

            self.history.run_query(
                query,
                &[
                    (":dir_filter_off", &only_dir.is_none()),
                    (":only_dir", &only_dir.as_ref().unwrap_or(&String::new())),
                    (":min_cmd_length", &min_cmd_length.to_owned()),
                    (":cmds", &cmds.to_owned()),
                    (
                        ":global_commands_to_ignore",
                        &global_commands_to_ignore.to_owned(),
                    ),
                    (":dirs", &dirs.to_owned()),
                ],
                |row| {
                    Ok(StatItem {
                        dir: Some(row.get(0)?),
                        cmd_tpl: row.get(1)?,
                        count: row.get(2)?,
                    })
                },
            )
        } else {
            let query = "
                SELECT cmd_tpl, COUNT(1) AS n
                FROM commands
                WHERE length(cmd_tpl) >= :min_cmd_length
                GROUP BY 1
                ORDER BY 2 DESC
                LIMIT :cmds
            ";

            self.history.run_query(
                query,
                &[
                    (":min_cmd_length", &min_cmd_length.to_owned()),
                    (":cmds", &cmds.to_owned()),
                ],
                |row| {
                    Ok(StatItem {
                        cmd_tpl: row.get(0)?,
                        count: row.get(1)?,
                        dir: None,
                    })
                },
            )
        }
    }

    #[inline]
    pub fn new(history: &'a History) -> Self {
        Self { history }
    }
    fn count_commands_from_db_history(&self, dir: &Option<String>) -> i32 {
        struct Count {
            count: i32,
        }
        let vec = self.history.run_query(
            "SELECT count(1) AS n FROM commands WHERE (:dir_filter_off OR dir = :directory)",
            &[
                (":dir_filter_off", &dir.is_none()),
                (":directory", &dir.as_ref().unwrap_or(&String::new())),
            ],
            |row| Ok(Count { count: row.get(0)? }),
        );
        vec.first().unwrap().count
    }
}
