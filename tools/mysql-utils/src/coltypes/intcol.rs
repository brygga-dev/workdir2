use super::*;

pub struct IntCol;
impl ColType for IntCol {
    fn col_value(&self, row: &mut mysql::Row, idx: usize) -> Result<Option<ColValue>> {
        match row.take_opt::<Option<i32>, _>(idx) {
            Some(Ok(v)) => Ok(v.map(ColValue::I32)),
            Some(Err(e)) => conv_err(Some(e)),
            None => conv_err(None),
        }
    }
    fn write(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::I32(v) = value {
            write!(w, "{}", v)?;
            Ok(())
        } else {
            write_err()
        }
    }

    fn json(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::I32(v) = value {
            write!(w, "{}", v)?;
            Ok(())
        } else {
            write_err()
        }
    }
    fn sexpr(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::I32(v) = value {
            write!(w, "{}", v)?;
            Ok(())
        } else {
            write_err()
        }
    }
}
impl std::fmt::Debug for IntCol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "IntCol")
    }
}
