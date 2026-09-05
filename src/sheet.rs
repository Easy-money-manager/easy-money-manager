use crate::record::Record;

// Sheet {{{
// Definition {{{
pub struct Sheet {
    pub name: String,
    pub records: Vec<Record>,
    pub fraction: i64,
}
// }}}

// Initialization {{{
impl Default for Sheet {
    fn default() -> Self {
        Self {
            name: String::new(),
            records: Vec::new(),
            fraction: 0,
        }
    }
}
// }}}

// Sum{{{
impl Sheet {
    pub fn sum(&self) -> i64 {
        let mut sum = 0;
        for record in &self.records {
            sum += record.value;
        }
        sum
    }
    pub fn sum_display(&self) -> String {
        (self.sum() as f64 / 100.0).to_string()
    }
}
// }}}
// }}}
