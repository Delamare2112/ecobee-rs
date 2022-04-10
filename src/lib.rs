#![feature(in_band_lifetimes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize)]
pub enum SelectionType {
    registered,
    thermostats,
    managementSet,
}

#[derive(Debug, Serialize)]
pub enum SelectionInclude {
    includeRuntime,
    includeExtendedRuntime,
    includeElectricity,
    includeSettings,
    includeLocation,
    includeProgram,
    includeEvents,
    includeDevice,
    includeTechnician,
    includeUtility,
    includeManagement,
    includeAlerts,
    includeReminders,
    includeWeather,
    includeHouseDetails,
    includeOemCfg,
    includeEquipmentStatus,
    includeNotificationSettings,
    includePrivacy,
    includeVersion,
    includeSecuritySettings,
    includeSensors,
    includeAudio,
    includeEnergy,
    includeCapabilities,
}

#[derive(Debug)]
pub struct Selection {
    pub selectionType: SelectionType,
    pub selectionMatch: String,
    pub include: SelectionInclude,
}

impl Selection {
    fn to_json(&self) -> String {
        let selection_type = format!("{:?}", self.selectionType);
        let selection_match = &self.selectionMatch;
        let selection_include = format!("{:?}", self.include);
        format!(
            "{{\"selectionType\":\"{selection_type}\",\"selectionMatch\":\"{selection_match}\",\"{selection_include}\":true}}"
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct Status {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct GetThermostatSummaryResponseJson {
    pub revisionList: Vec<String>,
    pub thermostatCount: i32,
    pub statusList: Vec<String>,
    pub status: Status,
}

#[derive(Debug)]
pub struct GetThermostatSummaryResponse {
    pub revisionList: Vec<CSVRevisionValues>,
    pub thermostatCount: i32,
    pub statusList: Vec<String>,
    pub status: Status,
}

impl Into<GetThermostatSummaryResponse> for GetThermostatSummaryResponseJson {
    fn into(self) -> GetThermostatSummaryResponse {
        GetThermostatSummaryResponse {
            revisionList: self
                .revisionList
                .iter()
                .map(|s| CSVRevisionValues::from_str(s).expect("Failed to make a CSV thing!"))
                .collect(),
            thermostatCount: self.thermostatCount,
            statusList: self.statusList,
            status: self.status,
        }
    }
}

#[derive(Debug)]
pub struct CSVRevisionValues {
    pub thermostat_identifier: String,
    pub thermostat_name: String,
    pub connected: bool,
    pub thermostat_revision: String,
    pub alerts_revision: String,
    pub runtime_revision: String,
    pub interval_revision: String,
}

impl FromStr for CSVRevisionValues {
    type Err = ();

    /// Eg: 522697894617:My ecobee:true:220115212500:220103232041:220115222447:220115222000
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splits = s.split(':');
        Ok(Self {
            thermostat_identifier: splits.next().ok_or(())?.to_string(),
            thermostat_name: splits.next().ok_or(())?.to_string(),
            connected: match splits.next().ok_or(())? {
                "true" => true,
                "false" => false,
                _ => return Err(()),
            },
            thermostat_revision: splits.next().ok_or(())?.to_string(),
            alerts_revision: splits.next().ok_or(())?.to_string(),
            runtime_revision: splits.next().ok_or(())?.to_string(),
            interval_revision: splits.next().ok_or(())?.to_string(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct GetRuntimeReportJson {
    pub selection: String,
    pub startDate: String,
    pub startInterval: i32,
    pub endDate: String,
    pub endInterval: i32,
    pub columns: String,
    pub includeSensors: bool,
}

#[derive(Debug)]
pub struct GetRuntimeReport {
    pub selection: Selection,
    pub startDate: String,
    pub startInterval: i32,
    pub endDate: String,
    pub endInterval: i32,
    pub columns: String,
    pub includeSensors: bool,
}

impl Into<GetRuntimeReportJson> for GetRuntimeReport {
    fn into(self) -> GetRuntimeReportJson {
        GetRuntimeReportJson {
            selection: self.selection.to_json(),
            startDate: self.startDate,
            startInterval: self.startInterval,
            endDate: self.endDate,
            endInterval: self.endInterval,
            columns: self.columns,
            includeSensors: self.includeSensors,
        }
    }
}

impl Default for GetRuntimeReport {
    fn default() -> Self {
        Self {
            selection: Selection {
                selectionType: SelectionType::thermostats,
                selectionMatch: "".to_string(),
                include: SelectionInclude::includeRuntime,
            },
            startDate: "".to_string(),
            startInterval: 0,
            endDate: "".to_string(),
            endInterval: 287,
            columns: "".to_string(),
            includeSensors: false,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RuntimeReport {
    pub thermostatIdentifier: Option<String>,
    pub rowCount: Option<i32>,
    pub rowList: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct RuntimeSensorMetadata {
    pub sensorId: Option<String>,
    pub sensorName: Option<String>,
    pub sensorType: Option<String>,
    pub sensorUsage: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RuntimeSensorReport {
    pub thermostatIdentifier: Option<String>,
    pub sensors: Option<Vec<RuntimeSensorMetadata>>,
    pub columns: Option<Vec<String>>,
    pub data: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct GetRuntimeReportResponse {
    pub startDate: String,
    pub startInterval: i32,
    pub endDate: String,
    pub endInterval: i32,
    pub columns: String,
    pub reportList: Vec<RuntimeReport>,
    pub sensorList: Vec<RuntimeSensorReport>,
}

pub struct Ecobee {
    pub auth: String,
    pub refresh: String,
}

impl Ecobee {
    pub fn get_thermostat_summary(&self, selection: Selection) -> GetThermostatSummaryResponse {
        let auth = &self.auth;
        let selection_type = format!("{:?}", selection.selectionType);
        let selection_match = selection.selectionMatch;
        let selection_include = format!("{:?}", selection.include);
        let selection = format!(
            "{{\"selectionType\":\"{selection_type}\",\"selectionMatch\":\"{selection_match}\",\"{selection_include}\":true}}"
        );
        let request = ureq::get(&format!("https://api.ecobee.com/1/thermostatSummary?format=json&body={{\"selection\":{selection}}}"))
            .set("Content-Type", "text/json")
            .set("Authorization", &format!("Bearer {auth}"))
            .call()
            .expect("Failed to build get_thermostat_summary request!");
        let j: GetThermostatSummaryResponseJson = serde_json::from_str(
            &request
                .into_string()
                .expect("Failed to get body from get_thermostat_summary request"),
        )
        .expect("Failed to deserialize body from get_thermostat_summary request");
        j.into()
    }
    pub fn get_runtime_report(&self, data: GetRuntimeReport) -> GetRuntimeReportResponse {
        let auth = &self.auth;
        let data: GetRuntimeReportJson = data.into();
        let data = serde_json::to_string(&data)
            .expect("Failed to serialize GetRuntimeReport object!")
            .replace("\\\"", "\"")
            .replace(r#""selection":"{""#, r#""selection":{""#)
            .replace(r#"":true}""#, r#"":true}"#);

        let request = ureq::get(&format!(
            "https://api.ecobee.com/1/runtimeReport?format=json&body={data}"
        ))
        .set("Content-Type", "text/json")
        .set("Authorization", &format!("Bearer {auth}"))
        .call()
        .expect("Failed to build get_runtime_report request!");
        serde_json::from_str(
            &request
                .into_string()
                .expect("Failed to get body from get_runtime_report request"),
        )
        .expect("Failed to deserialize body from get_runtime_report request")
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn thermostat_summary() {
        let bee = Ecobee {
            auth: std::env::var("ECOBEE_AUTH").expect("ECOBEE_AUTH must be set to run tests"),
            refresh: "".to_string(),
        };
        let ret = bee.get_thermostat_summary(Selection {
            selectionType: SelectionType::registered,
            selectionMatch: "".to_string(),
            include: SelectionInclude::includeDevice,
        });
        dbg!(ret);
    }
}
