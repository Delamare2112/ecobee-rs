use chrono::Datelike;
use ecobee::{Ecobee, GetRuntimeReport, Selection, SelectionInclude, SelectionType};
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
            include: SelectionInclude::includeDevice,
        });
        let new_revision = &summary.revisionList[0].runtime_revision;
        if runtime_revision != *new_revision {
            runtime_revision = new_revision.clone();

            let today = chrono::Local::today();
            let today = format!("{}-{}-{}", today.year(), today.month(), today.day());

            let thermostat_id = summary.revisionList[0].thermostat_identifier.clone();
            let request = GetRuntimeReport {
                selection: Selection {
                    selectionType: SelectionType::thermostats,
                    selectionMatch: thermostat_id,
                    include: SelectionInclude::includeDevice,
                },
                includeSensors: true,
                startDate: today.clone(),
                endDate: today,
                columns: "zoneHvacMode,zoneCalendarEvent".to_string(),
                ..Default::default()
            };
            dbg!(bee.get_runtime_report(request));
        }
        sleep(Duration::from_secs(15 * 60));
    }
}
