use chrono::Local;

// Record {{{
//
// description - description of specific money flow
// date - date of that money flow
// value - value of that flow; i64 because computers have problem with calculating 
// decimal fractions so to display it's just gonna be display value / 100
//
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
