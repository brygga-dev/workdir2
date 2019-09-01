use super::*;

pub struct BinCol;
impl ColType for BinCol {
    fn col_value(&self, row: &mut mysql::Row, idx: usize) -> Result<Option<ColValue>> {
        match row.take_opt::<Option<Vec<u8>>, _>(idx) {
            Some(Ok(v)) => Ok(v.map(ColValue::Bin)),
            Some(Err(e)) => conv_err(Some(e)),
            None => conv_err(None),
        }
    }
    fn write(&self, w: &mut Box<dyn Write>, _value: ColValue) -> Result<()> {
        write!(w, "[binary]")?;
        Ok(())
    }
    fn json(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::Bin(v) = value {
            write!(w, "{}", base64::encode(&v))?;
            Ok(())
        } else {
            write_err()
        }
    }
    fn sexpr(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::Bin(v) = value {
            write!(w, "{}", base64::encode(&v))?;
            Ok(())
        } else {
            write_err()
        }
    }
}
impl std::fmt::Debug for BinCol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "BinCol")
    }
}
