use chrono::Datelike;
use ecobee::{
    Ecobee, GetRuntimeReport, Selection, SelectionInclude, SelectionType, Settings, Thermostat,
    UpdateThermostat,
};
use std::cmp::Ordering;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut bee = Ecobee {
        api_key: std::env::var("ECOBEE_KEY").expect("ECOBEE_KEY must be est to run tests"),
        auth: std::env::var("ECOBEE_AUTH").expect("ECOBEE_AUTH must be set to run tests"),
        refresh: std::env::var("ECOBEE_REFRESH").expect("ECOBEE_REFRESH must be est to run tests"),
    };
    let mut runtime_revision = String::new();
    loop {
        let summary = bee.get_thermostat_summary(Selection {
            selectionType: SelectionType::registered,
            selectionMatch: "".to_string(),
            include: Some(SelectionInclude::includeDevice),
        });
        std::env::set_var("ECOBEE_AUTH", &bee.auth);
        std::env::set_var("ECOBEE_REFRESH", &bee.refresh);
        let new_revision = &summary.revisionList[0].runtime_revision;
        if runtime_revision != *new_revision {
            runtime_revision = new_revision.clone();

            let today = chrono::Utc::today();
            let today = format!("{}-{:0>2}-{:0>2}", today.year(), today.month(), today.day());
            // dbg!(&today);

            let thermostat_id = summary.revisionList[0].thermostat_identifier.clone();
            let request = GetRuntimeReport {
                selection: Selection {
                    selectionType: SelectionType::thermostats,
                    selectionMatch: thermostat_id.clone(),
                    include: Some(SelectionInclude::includeDevice),
                },
                includeSensors: true,
                startDate: today.clone(),
                endDate: today.clone(),
                columns: "zoneHvacMode,zoneCalendarEvent".to_string(),
                ..Default::default() // TODO: I don't have to grab all data from the start of the UTC day
            };
            let runtime_report = bee.get_runtime_report(request);
            dbg!(&runtime_report);
            let date_index = runtime_report.sensorList[0]
                .columns
                .as_ref()
                .expect("No sensor columns found!")
                .iter()
                .enumerate()
                .find(|(_i, id)| id == &"date")
                .expect("Failed")
                .0;
            let time_index = runtime_report.sensorList[0]
                .columns
                .as_ref()
                .expect("No sensor columns found!")
                .iter()
                .enumerate()
                .find(|(_i, id)| id == &"time")
                .expect("Failed")
                .0;
            let mut sorted_data = runtime_report.sensorList[0]
                .data
                .as_ref()
                .expect("Failed to find sensor data in the runtime report!")
                .clone();
            sorted_data.sort_by(|line_a, line_b| {
                let date_entry_a = line_a
                    .split(',')
                    .skip(date_index)
                    .next()
                    .expect("A data entry had too few entries!");
                let date_entry_b = line_b
                    .split(',')
                    .skip(date_index)
                    .next()
                    .expect("A data entry had too few entries!");
                let time_entry_a = line_a
                    .split(',')
                    .skip(time_index)
                    .next()
                    .expect("A data entry had too few entries!");
                let time_entry_b = line_b
                    .split(',')
                    .skip(time_index)
                    .next()
                    .expect("A data entry had too few entries!");
                let d = date_entry_b.cmp(date_entry_a);
                if d == Ordering::Equal {
                    time_entry_b.cmp(time_entry_a)
                } else {
                    d
                }
            });
            let something_open = runtime_report.sensorList[0]
                .sensors
                .as_ref()
                .expect("No sensors found!")
                .iter()
                .filter(|sensor| sensor.sensorType.is_some())
                .filter(|sensor| sensor.sensorType.as_ref().unwrap() == "dryContact")
                .map(|sensor| sensor.sensorId.as_ref().expect("A dryContact did not have an ID!"))
                .any(|dry_sensor_id| {
                    let index = runtime_report.sensorList[0]
                        .columns
                        .as_ref()
                        .expect("No sensor columns found!")
                        .iter()
                        .enumerate()
                        .find(|(_i, id)| id == &dry_sensor_id)
                        .expect(
                            "A dryContact had an ID, but the idea was not a column in available sensor data!",
                        )
                        .0;
                    let entry = sorted_data
                        .iter()
                        .map(|line| {
                            line.split(',')
                                .skip(index)
                                .next()
                                .expect("A data entry had too few entries!")
                        })
                        .filter(|entry| !entry.is_empty())
                        .next();
                    if let Some(entry) = entry {
                        if entry == "0" {
                            println!("{dry_sensor_id} most recently reported that it's open!");
                            return true;
                        } else {
                            println!("{dry_sensor_id} most recently reported that it's closed!");
                        }
                    } else {
                        println!("{dry_sensor_id} has no recent data.  Assuming it's closed!");
                    }
                    return false;
                });
            let mode = if something_open { "off" } else { "auto" };
            bee.update_thermostat(UpdateThermostat {
                selection: Selection {
                    selectionType: SelectionType::registered,
                    selectionMatch: "".to_string(),
                    include: None,
                },
                thermostat: Some(Thermostat {
                    identifier: thermostat_id,
                    settings: Some(Settings {
                        hvacMode: Some(mode.to_string()),
                    }),
                }),
                // functions: None,
            });
        }
        sleep(Duration::from_secs(15 * 60));
    }
}
