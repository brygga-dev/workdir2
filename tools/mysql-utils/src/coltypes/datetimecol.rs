use super::*;

pub struct DateTimeCol;
impl ColType for DateTimeCol {
    fn col_value(&self, row: &mut mysql::Row, idx: usize) -> Result<Option<ColValue>> {
        match row.take_opt::<Option<String>, _>(idx) {
            Some(Ok(v)) => Ok(v.map(ColValue::DateTime)),
            Some(Err(e)) => conv_err(Some(e)),
            None => conv_err(None),
        }
    }
    fn write(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::DateTime(v) = value {
            //write!(w, "{}", v.format("%Y-%m-%d %H:%M:%S"))?;
            write!(w, "{}", v)?;
            Ok(())
        } else {
            write_err()
        }
    }
    fn json(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::DateTime(v) = value {
            //write!(w, "\"{}\"", v.format("%Y-%m-%d %H:%M:%S"))?;
            write!(w, "\"{}\"", v)?;
            Ok(())
        } else {
            write_err()
        }
    }
    fn sexpr(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::DateTime(v) = value {
            //write!(w, "\"{}\"", v.format("%Y-%m-%d %H:%M:%S"))?;
            write!(w, "\"{}\"", v)?;
            Ok(())
        } else {
            write_err()
        }
    }
}
impl std::fmt::Debug for DateTimeCol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DateTimeCol")
    }
}
