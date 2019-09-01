use super::*;

pub struct FloatCol;
impl ColType for FloatCol {
    fn col_value(&self, row: &mut mysql::Row, idx: usize) -> Result<Option<ColValue>> {
        match row.take_opt::<Option<f32>, _>(idx) {
            Some(Ok(v)) => Ok(v.map(ColValue::F32)),
            Some(Err(e)) => conv_err(Some(e)),
            None => conv_err(None),
        }
    }
    fn write(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::F32(v) = value {
            write!(w, "{}", v)?;
            Ok(())
        } else {
            write_err()
        }
    }

    fn json(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::F32(v) = value {
            write!(w, "{}", v)?;
            Ok(())
        } else {
            write_err()
        }
    }

    fn sexpr(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::F32(v) = value {
            write!(w, "{}", v)?;
            Ok(())
        } else {
            write_err()
        }
    }
}
impl std::fmt::Debug for FloatCol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "FloatCol")
    }
}
