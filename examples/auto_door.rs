use chrono::Datelike;
use ecobee::{
    Ecobee, GetRuntimeReport, Selection, SelectionInclude, SelectionType, Settings, Thermostat,
    UpdateThermostat,
};
use std::cmp::Ordering;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let bee = Ecobee {
        auth: std::env::var("ECOBEE_AUTH").expect("ECOBEE_AUTH must be set to run tests"),
        refresh: std::env::var("ECOBEE_REFRESH").unwrap_or_default(),
    };
    let mut runtime_revision = String::new();
    loop {
        let summary = bee.get_thermostat_summary(Selection {
            selectionType: SelectionType::registered,
            selectionMatch: "".to_string(),
            include: Some(SelectionInclude::includeDevice),
        });
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
            let catio_id = runtime_report.sensorList[0]
                .sensors
                .as_ref()
                .expect("No sensors found!")
                .iter()
                .filter(|sensor| sensor.sensorName.is_some())
                .find(|sensor| sensor.sensorName.as_ref().unwrap() == "Catio Door")
                .expect("Failed to find Catio Door sensor!")
                .sensorId
                .as_ref()
                .expect("Somehow the Catio Door did not have a sensor Id!");
            let catio_index = runtime_report.sensorList[0]
                .columns
                .as_ref()
                .expect("No sensor columns found!")
                .iter()
                .enumerate()
                .find(|(_i, id)| id == &catio_id)
                .expect("Failed")
                .0;
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
            let mut valid_data = runtime_report.sensorList[0]
                .data
                .as_ref()
                .expect("Failed to find sensor data in the runtime report!")
                .iter()
                .map(|data| {
                    (
                        data.split(',')
                            .skip(date_index)
                            .next()
                            .expect("A data entry had too few entries!"),
                        data.split(',')
                            .skip(time_index) // This is in UTC
                            .next()
                            .expect("A data entry had too few entries!"),
                        data.split(',')
                            .skip(catio_index)
                            .next()
                            .expect("A data entry had too few entries!"),
                    )
                })
                .filter(|(_date_entry, _time_entry, catio_entry)| !catio_entry.is_empty())
                .collect::<Vec<_>>();
            valid_data.sort_by(
                |(date_entry_a, time_entry_a, _), (date_entry_b, time_entry_b, _)| {
                    let d = date_entry_b.cmp(date_entry_a);
                    if d == Ordering::Equal {
                        time_entry_b.cmp(time_entry_a)
                    } else {
                        d
                    }
                },
            );
            dbg!(&valid_data);
            if let Some((_, _, sensor_status)) = valid_data.iter().next() {
                let mode = if sensor_status == &"1" {
                    println!("Door is now closed.");
                    "auto".to_string()
                } else {
                    println!("Door is now open!");
                    "off".to_string()
                };
                bee.update_thermostat(UpdateThermostat {
                    selection: Selection {
                        selectionType: SelectionType::registered,
                        selectionMatch: "".to_string(),
                        include: None,
                    },
                    thermostat: Some(Thermostat {
                        identifier: thermostat_id,
                        settings: Some(Settings {
                            hvacMode: Some(mode),
                        }),
                    }),
                    // functions: None,
                });
            } else {
                eprintln!("No sensor data!");
            }
        }
        sleep(Duration::from_secs(15 * 60));
    }
}
