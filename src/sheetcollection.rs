use crate::sheet::/*{*/Sheet;//, SheetError};

pub struct SheetCollection {
    pub name: String,
    pub sheets: Vec<Sheet>,
}

impl Default for SheetCollection {
    fn default() -> Self {
        Self {
            name: String::new(),
            sheets: Vec::new(),
        }
    }
}

impl SheetCollection {
    pub fn len(&self) -> usize {
        self.sheets.len()
    }
    pub fn create(name: &str) -> Self {
        Self {
            name: name.to_string(),
            sheets: Vec::new(),
        }
    }
    pub fn push(&mut self, name: &str, fraction: i64) {
        self.sheets.push(Sheet::create(name, fraction));
    }
}
