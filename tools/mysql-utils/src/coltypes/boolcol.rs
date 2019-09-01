use super::*;

pub struct BoolCol;
impl ColType for BoolCol {
    fn col_value(&self, row: &mut mysql::Row, idx: usize) -> Result<Option<ColValue>> {
        match row.take_opt::<Option<bool>, _>(idx) {
            Some(Ok(v)) => Ok(v.map(ColValue::Bool)),
            Some(Err(e)) => conv_err(Some(e)),
            None => conv_err(None),
        }
    }
    fn write(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::Bool(b) = value {
            if b {
                w.write_all(b"true")?;
                Ok(())
            } else {
                w.write_all(b"false")?;
                Ok(())
            }
        } else {
            write_err()
        }
    }
    fn json(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::Bool(b) = value {
            if b {
                w.write_all(b"true")?;
                Ok(())
            } else {
                w.write_all(b"false")?;
                Ok(())
            }
        } else {
            write_err()
        }
    }
    fn sexpr(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::Bool(b) = value {
            if b {
                w.write_all(b"T")?;
                Ok(())
            } else {
                w.write_all(b"F")?;
                Ok(())
            }
        } else {
            write_err()
        }
    }
}
impl std::fmt::Debug for BoolCol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "BoolCol")
    }
}
