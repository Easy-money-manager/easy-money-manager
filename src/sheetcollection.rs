use crate::sheet::{Sheet, SheetError};

struct SheetCollection {
    name: String,
    sheets: Vec<Sheet>,
}

impl Default for SheetCollection {
    fn default() -> Self {
        Self {
            name: String::new(),
            sheets: Vec::new(),
        }
    }
}
