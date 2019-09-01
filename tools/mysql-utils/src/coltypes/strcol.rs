use super::*;

pub struct StrCol;
impl ColType for StrCol {
    // &str?
    fn col_value(&self, row: &mut mysql::Row, idx: usize) -> Result<Option<ColValue>> {
        match row.take_opt::<Option<String>, _>(idx) {
            Some(Ok(v)) => Ok(v.map(ColValue::Str)),
            Some(Err(e)) => conv_err(Some(e)),
            None => conv_err(None),
        }
    }
    fn write(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::Str(mut v) = value {
            if v.len() > 20 {
                v.truncate(20); // temp
                w.write_all(v.as_bytes())?;
            } else {
                w.write_all(v.as_bytes())?;
            }
            Ok(())
        } else {
            write_err()
        }
    }
    fn json(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::Str(v) = value {
            // There should be an option for this,
            // but now attempting to parse php serialized string
            match ser_utils::php::deserialize(v.as_bytes()) {
                Ok(ser) => {
                    w.write_all(
                        ser_utils::php_json::serialize_json_lines(String::with_capacity(256), ser, 3).as_bytes(),
                    )?;
                }
                Err(_) => {
                    // Regular string
                    // Hard to predict how much is needed
                    // ideally we should share a buffer
                    let mut b = String::with_capacity(v.len() + 5);
                    /*
                    if v.starts_with("a:4:{s:5:") {
                        let mut tmp = std::fs::File::create("temp.txt")?;
                        write!(tmp, "{}", v)?;
                    }*/
                    b = ser_utils::json::json_escape_string(&v, b);
                    w.write_all(b.as_bytes())?;
                }
            }
            Ok(())
        } else {
            write_err()
        }
    }

    fn sexpr(&self, w: &mut Box<dyn Write>, value: ColValue) -> Result<()> {
        if let ColValue::Str(v) = value {
            // There should be an option for this,
            // but now attempting to parse php serialized string
            match ser_utils::php::deserialize(v.as_bytes()) {
                Ok(ser) => {
                    w.write_all(
                        ser_utils::sexpr::serialize_php(String::with_capacity(256), ser, 3).as_bytes(),
                    )?;
                }
                Err(_) => {
                    // Regular string
                    // Hard to predict how much is needed
                    // ideally we should share a buffer
                    let mut b = String::with_capacity(v.len() + 5);
                    /*
                    if v.starts_with("a:4:{s:5:") {
                        let mut tmp = std::fs::File::create("temp.txt")?;
                        write!(tmp, "{}", v)?;
                    }*/
                    b = ser_utils::sexpr::escape_string(&v, b);
                    w.write_all(b.as_bytes())?;
                }
            }
            Ok(())
        } else {
            write_err()
        }
    }
}
impl std::fmt::Debug for StrCol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "StrCol")
    }
}
