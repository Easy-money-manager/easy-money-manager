use emm_shared::record::ParsedImportRecord;
use chrono::NaiveDate;
use serde::{ Deserialize, Serialize };

#[derive(Debug, Serialize, Deserialize)]
pub struct CsvImportRecord {
    description: String,
    date: String,
    value: String
}

pub struct Import;

impl Import {
    pub fn parse_csv_import(csv_data: &str) -> Result<Vec<ParsedImportRecord>, Box<dyn std::error::Error>> {
        let mut reader = csv::Reader::from_reader(csv_data.as_bytes());
        let mut records: Vec<ParsedImportRecord> = Vec::new();

        for result in reader.deserialize::<CsvImportRecord>() {
            let row = result?;

            let date  = Self::parse_date(&row.date)?;
            let value = Self::parse_money(&row.value)?;
            records.push(ParsedImportRecord {
                description: row.description,
                date: date,
                value: value,
            });
        }
        Ok(records)
    }
    fn parse_date(date: &str) -> Result<NaiveDate, chrono::ParseError> {
        let date = date.trim();
        NaiveDate::parse_from_str(date, "%d.%m.%Y")
            .or_else(|_| NaiveDate::parse_from_str(date, "%d/%m/%Y"))
            .or_else(|_| NaiveDate::parse_from_str(date, "%d-%m-%Y"))
    }
    fn parse_money(value: &str) -> Result<i64, Box<dyn std::error::Error>> {
        let value: String = value.trim().replace(',', ".");
        let negative: bool = value.starts_with('-');
        let value: &str = value.trim_start_matches('-');

        let mut parts = value.split('.');
        let zl: i64 = parts.next().unwrap_or("0").parse()?;
        let fraction: &str = parts.next().unwrap_or("0");
        let gr = match fraction.len() {
            0 => 0,
            1 => fraction.parse::<i64>()? * 10,
            _ => fraction[..2].parse::<i64>()?,
        };
        let result = zl * 100 + gr;
        Ok(if negative { -result } else { result })
    }
}
