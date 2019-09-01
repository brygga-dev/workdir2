mod bincol;
mod boolcol;
mod datecol;
mod datetimecol;
mod floatcol;
mod intcol;
mod strcol;
mod timecol;
mod uintcol;

pub use bincol::BinCol;
pub use boolcol::BoolCol;
pub use datecol::DateCol;
pub use datetimecol::DateTimeCol;
pub use floatcol::FloatCol;
pub use intcol::IntCol;
pub use strcol::StrCol;
pub use timecol::TimeCol;
pub use uintcol::UIntCol;

pub use crate::er::{MyLibError, Result};
pub use std::io::Write;

// Not sure of performance impact vs more gnarly
// direct approaches
// Better solutions is something to look for
pub enum ColValue {
    I32(i32),
    U32(u32),
    F32(f32),
    Str(String),
    Bool(bool),
    // Strings for now to get 0000-00.. working
    Date(String),
    DateTime(String),
    Time(String),
    Bin(Vec<u8>),
}

struct DateVal {}

pub fn conv_err(e: Option<mysql::FromValueError>) -> Result<Option<ColValue>> {
    Err(MyLibError::Msg(format!("Failed to convert: {:?}", e)))
}

pub fn write_err() -> Result<()> {
    Err(MyLibError::Msg("Failed to write".into()))
}

// I took a bit much for my liking with these trait object,
// feels like it may not be rusts best way
// it may however work/perform well anyway. Trying some time
pub trait ColType: std::fmt::Debug {
    // Option takes nullable into account
    // There could be two variants to specifically handle null/non null
    fn col_value(&self, row: &mut mysql::Row, idx: usize) -> Result<Option<ColValue>>;
    fn write(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()>;
    fn json(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()>;
    fn sexpr(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()>;

    fn opt_write(&self, w: &mut Box<dyn Write>, value: Option<ColValue>) -> Result<()> {
        match value {
            Some(value) => self.write(w, value),
            None => {
                write!(w, "null")?;
                Ok(())
            }
        }
    }

    fn opt_json(&self, w: &mut Box<dyn Write>, value: Option<ColValue>) -> Result<()> {
        match value {
            Some(value) => self.json(w, value),
            None => {
                write!(w, "null")?;
                Ok(())
            }
        }
    }

    fn opt_sexpr(&self, w: &mut Box<dyn Write>, value: Option<ColValue>) -> Result<()> {
        match value {
            Some(value) => self.sexpr(w, value),
            None => {
                write!(w, "N")?;
                Ok(())
            }
        }
    }
}
