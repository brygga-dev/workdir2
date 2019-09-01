use super::*;

pub struct UIntCol;
impl ColType for UIntCol {
    fn col_value(&self, row: &mut mysql::Row, idx: usize) -> Result<Option<ColValue>> {
        match row.take_opt::<Option<u32>, _>(idx) {
            Some(Ok(v)) => Ok(v.map(ColValue::U32)),
            Some(Err(e)) => conv_err(Some(e)),
            None => conv_err(None),
        }
    }
    fn write(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::U32(v) = value {
            write!(w, "{}", v)?;
            Ok(())
        } else {
            write_err()
        }
    }

    fn json(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::U32(v) = value {
            write!(w, "{}", v)?;
            Ok(())
        } else {
            write_err()
        }
    }

    fn sexpr(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::U32(v) = value {
            write!(w, "{}", v)?;
            Ok(())
        } else {
            write_err()
        }
    }
}
impl std::fmt::Debug for UIntCol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "UIntCol")
    }
}
