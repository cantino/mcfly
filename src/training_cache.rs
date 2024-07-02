use crate::history::Features;
use csv::Reader;
use csv::Writer;
use std::fs::File;
use std::path::Path;

pub fn write(data_set: &[(Features, bool)], cache_path: &Path) {
    let mut writer = Writer::from_path(cache_path)
        .unwrap_or_else(|err| panic!("McFly error: Expected to be able to write a CSV ({err})"));
    output_header(&mut writer);

    for (features, correct) in data_set {
        output_row(&mut writer, features, *correct);
    }
}

#[must_use]
pub fn read(cache_path: &Path) -> Vec<(Features, bool)> {
    let mut data_set: Vec<(Features, bool)> = Vec::new();

    let mut reader = Reader::from_path(cache_path)
        .unwrap_or_else(|err| panic!("McFly error: Expected to be able to read from CSV ({err})"));

    for result in reader.records() {
        let record = result.unwrap_or_else(|err| {
            panic!("McFly error: Expected to be able to unwrap cached result ({err})")
        });

        let features = Features {
            age_factor: record[0].parse().unwrap(),
            length_factor: record[1].parse().unwrap(),
            exit_factor: record[2].parse().unwrap(),
            recent_failure_factor: record[3].parse().unwrap(),
            selected_dir_factor: record[4].parse().unwrap(),
            dir_factor: record[5].parse().unwrap(),
            overlap_factor: record[6].parse().unwrap(),
            immediate_overlap_factor: record[7].parse().unwrap(),
            selected_occurrences_factor: record[8].parse().unwrap(),
            occurrences_factor: record[9].parse().unwrap(),
        };

        data_set.push((features, record[10].eq("t")));
    }

    data_set
}

fn output_header(writer: &mut Writer<File>) {
    writer
        .write_record([
            "age_factor",
            "length_factor",
            "exit_factor",
            "recent_failure_factor",
            "selected_dir_factor",
            "dir_factor",
            "overlap_factor",
            "immediate_overlap_factor",
            "selected_occurrences_factor",
            "occurrences_factor",
            "correct",
        ])
        .unwrap_or_else(|err| panic!("McFly error: Expected to write to CSV ({err})"));
    writer
        .flush()
        .unwrap_or_else(|err| panic!("McFly error: Expected to flush CSV ({err})"));
}

fn output_row(writer: &mut Writer<File>, features: &Features, correct: bool) {
    writer
        .write_record(&[
            format!("{}", features.age_factor),
            format!("{}", features.length_factor),
            format!("{}", features.exit_factor),
            format!("{}", features.recent_failure_factor),
            format!("{}", features.selected_dir_factor),
            format!("{}", features.dir_factor),
            format!("{}", features.overlap_factor),
            format!("{}", features.immediate_overlap_factor),
            format!("{}", features.selected_occurrences_factor),
            format!("{}", features.occurrences_factor),
            if correct {
                String::from("t")
            } else {
                String::from("f")
            },
        ])
        .unwrap_or_else(|err| panic!("McFly error: Expected to write to CSV ({err})"));
    writer
        .flush()
        .unwrap_or_else(|err| panic!("McFly error: Expected to flush CSV ({err})"));
}
