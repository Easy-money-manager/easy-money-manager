use chrono::{NaiveDate, Local};

#[derive(Debug)]
pub enum RecordError {
    EmptyDescription,
    InvalidYear,
    InvalidMonth,
    InvalidDay,
    ValueError(ValueError),
}

#[derive(Debug)]
pub enum ValueError {
    InvalidZl,
    InvalidGr,
}
impl From<ValueError> for RecordError {
    fn from(error: ValueError) -> Self {
        RecordError::ValueError(error)
    }
}


// Record
//
// description - description of specific money flow
// date - date of that money flow
// value - value of that flow; i64 because computers have problem with calculating 
// decimal fractions so to display it's just gonna be display value / 100

pub struct Record {
    pub description: String,
    pub date: chrono::NaiveDate,
    pub value: i64,
}
impl Default for Record {
    fn default() -> Self {
        Self {
            description: String::new(),
            date: Local::now().date_naive(),
            value: 0,
        }
    }
}
impl Clone for Record {
    fn clone(&self) -> Record {
        Record {
            description: self.description.clone(),
            date: self.date.clone(),
            value: self.value.clone(),
        }
    }
}
impl Record {
    pub fn parse_value(value_zl: &str, value_gr: &str) -> Result<i64, ValueError> {
        let zl = if value_zl.is_empty() { 0 } else {
            value_zl.parse::<i64>().map_err(|_| ValueError::InvalidZl)?
        };
        let gr = if value_gr.is_empty() { 0 } else {
            value_gr.parse::<i64>().map_err(|_| ValueError::InvalidGr)?
        };
        Ok(zl * 100 + gr)
    }
    pub fn from_input(description: &str, year: &i32, month: &u32, day: &u32, value_zl: &str, value_gr: &str) -> Result<Self, RecordError> {
        if description.trim().is_empty() {
            return Err(RecordError::EmptyDescription);
        }
        if !(1900..=2100).contains(year) {
            return Err(RecordError::InvalidYear);
        }
        if !(1..=12).contains(month) {
            return Err(RecordError::InvalidMonth);
        }
        if *day < 1 ||
            *day > 30 + ((*month % 2 != 0) ^ (*month > 7)) as u32 ||
                2 == *month && *day > 28 + (*year % 4 == 0 && *year % 100 != 0 || year % 400 == 0) as u32 {
            return Err(RecordError::InvalidDay);
        }
        Ok(Self {
            description: description.to_string(),
            date: NaiveDate::from_ymd_opt(*year, *month, *day).unwrap(),
            value: Self::parse_value(value_zl, value_gr)?,
        })
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn value_display(&self) -> String {
        (self.value as f64 / 100.0).to_string()
    }
    pub fn date_display(&self) -> String {
        self.date.format("%d.%m.%Y").to_string()
    }
    pub fn date_set(&mut self, year: &i32, month: &u32, day: &u32) {
        self.date = NaiveDate::from_ymd_opt(*year, *month, *day).unwrap();
    }
    pub fn description_set(&mut self, description: &String) {
        self.description = description.clone();
    }
    pub fn value_set(&mut self, value_zl: &String, value_gr: &String) {
        self.value = match value_zl.is_empty() {
            false => (match value_zl.parse::<i64>() {
                Ok(value) => value,
                Err(_)    => 0,
            }) * 100,
            true  => 0,
        } + match value_gr.is_empty() {
            false => match value_gr.parse::<i64>() {
                Ok(value) => value,
                Err(_)    => 0,
            },
            true  => 0,
        };
    }
}
