use crate::record::Record;

// Sheet
//
// name - name of sheet
// records - list (Vec<Record>) with records to store informations about money flow
// fraction - how much of your incomes you wanna commit to this type of money flow,
// tho save as with Record::value it's i64 and just getting divided by 100 for 
// calculations, so for example if for essential things you wanna spend up to 50%
// of your income it's fraction 50 (50% = 50 (= fraction) / 100)

pub struct Sheet {
    pub name: String,
    pub records: Vec<Record>,
    pub fraction: i64,
}
impl Default for Sheet {
    fn default() -> Self {
        Self {
            name: String::new(),
            records: Vec::new(),
            fraction: 0,
        }
    }
}
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
