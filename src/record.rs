use chrono::Local;

// Record {{{
// Definition {{{
pub struct Record {
    pub description: String,
    pub date: chrono::NaiveDate,
    pub value: i64,
}
// }}}

// Initialization {{{
impl Default for Record {
    fn default() -> Self {
        Self {
            description: String::new(),
            date: Local::now().date_naive(),
            value: 0,
        }
    }
}
// }}}

// Clone {{{
impl Clone for Record {
    fn clone(&self) -> Record {
        Record {
            description: self.description.clone(),
            date: self.date.clone(),
            value: self.value.clone(),
        }
    }
}
// }}}

// Display Value {{{
impl Record {
    pub fn value_display(&self) -> String {
        (self.value as f64 / 100.0).to_string()
    }
}
// }}}
// }}}
