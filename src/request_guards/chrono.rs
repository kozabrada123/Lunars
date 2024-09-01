use chrono::{DateTime, Utc};

/// Tries to somehow get a chrono utc timestamp
/// from a string.
///
/// Attempts to parse from rfc3339 (ISO) and also unix milliseconds.
pub fn chrono_timestamp_from_string(string: &str) -> Option<DateTime<Utc>> {
    if let Ok(datetime) = DateTime::parse_from_rfc3339(string) {
        return Some(datetime.with_timezone(&Utc));
    }

    if let Ok(millis) = string.parse::<i64>() {
        if let Some(datetime) = DateTime::<Utc>::from_timestamp_millis(millis) {
            return Some(datetime);
        }
    }

    None
}

// Screw all this code.
//
/*
// See https://stackoverflow.com/questions/55029850/how-to-use-a-date-in-the-url-with-rocket-rs

use chrono::DateTime;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;

use chrono::ParseError;
use chrono::TimeZone;
use chrono::Utc;
use rocket::form::FromForm;
use rocket::request::FromParam;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::OpenApiFromRequest;

// https://stackoverflow.com/questions/25413201/how-do-i-implement-a-trait-i-dont-own-for-a-type-i-dont-own
// https://github.com/SergioBenitez/Rocket/issues/602#issuecomment-380497269
pub struct NaiveDateForm(NaiveDate);
pub struct NaiveTimeForm(NaiveTime);
pub struct NaiveDateTimeForm(NaiveDateTime);

impl<'v> FromParam<'v> for NaiveDateForm {
    type Error = ParseError;

    fn from_param(param: &'v str) -> Result<Self, Self::Error> {
        match NaiveDate::parse_from_str(&param, "%Y-%m-%d") {
            Ok(date) => Ok(NaiveDateForm(date)),
            Err(e) => Err(e),
        }
    }
}

impl<'v> FromParam<'v> for NaiveTimeForm {
    type Error = ParseError;

    fn from_param(param: &'v str) -> Result<Self, Self::Error> {
        if let Ok(time) = NaiveTime::parse_from_str(&param, "%H:%M:%S%.3f") {
            return Ok(NaiveTimeForm(time));
        }
        if let Ok(time) = NaiveTime::parse_from_str(&param, "%H:%M") {
            return Ok(NaiveTimeForm(time));
        }

          Err(NaiveTime::parse_from_str(&param, "%H:%M:%S%.3f").err().unwrap())
    }
}

impl<'v> FromParam<'v> for NaiveDateTimeForm {
    type Error = &'v str;

    fn from_param(param: &'v str) -> Result<NaiveDateTimeForm, Self::Error> {
        if param.len() < "0000-00-00T00:00".len() {
            return Err(param);
        }
        let date = NaiveDateForm::from_param(&param[.."0000-00-00".len()])
            .map_err(|_| param)?;
        let time =
            NaiveTimeForm::from_param(&param["0000-00-00T".len()..])
                .map_err(|_| param)?;
        Ok(NaiveDateTimeForm(NaiveDateTime::new(*date, *time)))
    }
}

impl std::ops::Deref for NaiveDateForm {
    type Target = NaiveDate;
    fn deref(&self) -> &NaiveDate {
        &self.0
    }
}

impl std::ops::Deref for NaiveTimeForm {
    type Target = NaiveTime;
    fn deref(&self) -> &NaiveTime {
        &self.0
    }
}

impl std::ops::Deref for NaiveDateTimeForm {
    type Target = NaiveDateTime;
    fn deref(&self) -> &NaiveDateTime {
        &self.0
    }
}

pub struct UtcDateTimeForm(DateTime<Utc>);

impl<'v> FromParam<'v> for UtcDateTimeForm {
    type Error = ParseError;

    fn from_param(param: &'v str) -> Result<UtcDateTimeForm, Self::Error> {
         if let Ok(datetime) = DateTime::parse_from_rfc3339(param) {
             return Ok(UtcDateTimeForm(datetime.with_timezone(&Utc)));
         }

         if let Ok(naive_datetime) = NaiveDateTimeForm::from_param(param) {
             return Ok(UtcDateTimeForm(Utc.from_utc_datetime(&naive_datetime)));
         }

         if let Ok(millis) = param.parse::<i64>() {
            if let Some(datetime) = DateTime::<Utc>::from_timestamp_millis(millis) {
                return Ok(UtcDateTimeForm(datetime));
            }
         }

         Err(DateTime::parse_from_rfc3339(param).err().unwrap())
    }
}

impl std::ops::Deref for UtcDateTimeForm {
    type Target = DateTime<Utc>;
    fn deref(&self) -> &DateTime<Utc> {
        &self.0
    }
}
*/
