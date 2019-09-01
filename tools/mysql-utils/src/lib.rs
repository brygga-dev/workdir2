mod coltypes;
pub mod er;
pub mod table_def;
use er::{err_msg, error_msg, Result};
use indexmap::IndexMap;
use mysql::Value;
use std::convert::TryFrom;
use table_def::TableDef;

type TableDefMap = IndexMap<String, TableDef>;

pub struct Db {
    conn: mysql::Conn,
    pub table_defs: TableDefMap,
}
impl Db {
    pub fn new(host: &str, port: u16, db_name: &str, user: &str, pass: &str) -> Result<Self> {
        let mut conn_builder = mysql::OptsBuilder::new();
        conn_builder
            .ip_or_hostname(Some(host))
            .tcp_port(port)
            .db_name(Some(db_name))
            .user(Some(user))
            .pass(Some(pass));
        let conn_opts = mysql::Opts::from(conn_builder);
        // Gives overflow error when error connecting?
        let conn = mysql::Conn::new(conn_opts)?;
        Ok(Db {
            conn,
            table_defs: IndexMap::new(),
        })
    }

    pub fn tablenames(&mut self) -> Result<Vec<String>> {
        let tables = Self::query(&mut self.conn, "show tables")?;
        let mut tablenames = Vec::new();
        for row in tables {
            let mut row = row?;
            let value: String = match row.take_opt(0) {
                Some(v) => v?,
                None => return err_msg("Could not convert"),
            };
            tablenames.push(value);
        }
        Ok(tablenames)
    }

    pub fn init_table_defs(&mut self) -> Result<()> {
        let tablenames = self.tablenames()?;
        let from = std::time::Instant::now();
        for tablename in tablenames {
            let create_tbl_query =
                Self::query(&mut self.conn, &format!("show create table {}", tablename))?;
            for row in create_tbl_query {
                let mut row = row?;
                let value: Vec<u8> = match row.take_opt(1) {
                    Some(v) => v?,
                    None => return err_msg("Could not convert"),
                };
                //println!("{}", String::from_utf8_lossy(&value));
                let td = TableDef::try_from(value.as_slice())?;
                self.table_defs.insert(td.name.clone(), td);
            }
        }
        println!("Time: {:?}", from.elapsed().as_millis());
        Ok(())
    }

    pub fn get_table<'a>(table_defs: &'a TableDefMap, table_name: &str) -> Result<&'a TableDef> {
        table_defs
            .get(table_name)
            .ok_or_else(|| error_msg("Failed to get table"))
    }

    pub fn tables_to_json_files(&mut self) -> Result<()> {
        use std::fs;
        use std::path::PathBuf;
        let base_dir = PathBuf::from("tables_json");
        if !base_dir.exists() {
            fs::create_dir(&base_dir)?;
        }
        let tablenames = self.tablenames()?;
        // One folder for each table
        for tablename in tablenames {
            let mut table_dir = base_dir.clone();
            table_dir.push(&tablename);
            if !table_dir.exists() {
                fs::create_dir(&table_dir)?;
            }
            // Inside, one file for definition
            // For now, one file with data for each table
            let data_file = fs::File::create(table_dir.join("data.json"))?;
            self.table_to_json(&tablename, Box::new(data_file))?;
        }
        // Todo: I think, records should have one file
        // each and be partitioned with a
        // folder for each around 1000s records so
        // the filesystem handles it better
        // '1' (1..=1000), '1001' (1001..=2000) etc
        // Each file should get a name encoding it's
        // primary key
        // vs big file, this will be more resource
        // efficient, and probably easier to inspect
        // (hashes and weird folder names for non-auto-increment
        // keys possible exceptions)
        Ok(())
    }

    pub fn table_to_json(&mut self, table: &str, mut out: Box<dyn std::io::Write>) -> Result<()> {
        let table_def = Self::get_table(&self.table_defs, table)?;
        let sql = table_def.select_sql();
        let result = Self::query(&mut self.conn, &sql)?;
        let mut row_idx = 0;
        write!(out, "[\n")?;
        for row in result {
            let mut row = row?;
            let mut idx = 0;
            if row_idx == 0 {
                write!(out, "\t[\n")?;
            } else {
                write!(out, ",\n\t[\n")?;
            }
            for (_name, field) in &table_def.fields {
                if idx == 0 {
                    write!(out, "\t\t")?;
                } else {
                    write!(out, ",\n\t\t")?;
                }
                field
                    .renderer
                    .opt_json(&mut out, field.renderer.col_value(&mut row, idx)?)?;
                idx += 1;
            }
            write!(out, "\n\t]")?;
            row_idx += 1;
        }
        write!(out, "\n]")?;
        Ok(())
    }

    pub fn tables_to_sexpr_files(&mut self) -> Result<()> {
        use std::fs;
        use std::path::PathBuf;
        let base_dir = PathBuf::from("tables_sexpr");
        if !base_dir.exists() {
            fs::create_dir(&base_dir)?;
        }
        let tablenames = self.tablenames()?;
        // One folder for each table
        for tablename in tablenames {
            let mut table_dir = base_dir.clone();
            table_dir.push(&tablename);
            if !table_dir.exists() {
                fs::create_dir(&table_dir)?;
            }
            // Inside, one file for definition
            // For now, one file with data for each table
            let data_file = fs::File::create(table_dir.join("data.scm"))?;
            self.table_to_sexpr(&tablename, Box::new(data_file))?;
        }
        Ok(())
    }

    pub fn table_to_sexpr(&mut self, table: &str, mut out: Box<dyn std::io::Write>) -> Result<()> {
        let table_def = Self::get_table(&self.table_defs, table)?;
        let sql = table_def.select_sql();
        let result = Self::query(&mut self.conn, &sql)?;
        write!(out, "(")?;
        for row in result {
            let mut row = row?;
            write!(out, "\n\t(\n")?;
            let mut idx = 0;
            for (_name, field) in &table_def.fields {
                if idx == 0 {
                    write!(out, "\t\t")?;
                } else {
                    write!(out, "\n\t\t")?;
                }
                field
                    .renderer
                    .opt_sexpr(&mut out, field.renderer.col_value(&mut row, idx)?)?;
                idx += 1;
            }
            write!(out, "\n\t)")?;
        }
        write!(out, "\n)")?;
        Ok(())
    }

    pub fn print_all(&mut self, table: &str) -> Result<()> {
        let table_def = Self::get_table(&self.table_defs, table)?;
        let sql = table_def.select_sql();
        let result = Self::query(&mut self.conn, &sql)?;
        let stdout = std::io::stdout();
        let mut o: Box<dyn std::io::Write> = Box::new(stdout);
        for row in result {
            let mut row = row?;
            let mut idx = 0;
            for (_name, field) in &table_def.fields {
                if idx > 0 {
                    write!(o, " | ")?;
                }
                field
                    .renderer
                    .opt_write(&mut o, field.renderer.col_value(&mut row, idx)?)?;
                idx += 1;
            }
            writeln!(o, "")?;
        }
        /*
        let stdout = std::io::stdout();
        let mut o = stdout.lock();
        for row in result {
            let row = row?;
            let values = row.unwrap();
            for value in values {
                match value {
                    Value::Int(v) => write!(o, "{}", v)?,
                    Value::UInt(v) => write!(o, "{}", v)?,
                    Value::Bytes(v) => write!(o, "{}", String::from_utf8_lossy(&v))?,
                    Value::NULL => write!(o, "NULL")?,
                    Value::Float(v) => write!(o, "{}", v)?,
                    Value::Time(neg, d, h, m, s, ms) => {
                        write!(o, "{} {} {}:{}:{}:{}", neg, d, h, m, s, ms)?;
                    }
                    Value::Date(y, month, d, h, m, s, ms) => {
                        write!(o, "{}-{}-{} {}:{}:{}:{}", y, month, d, h, m, s, ms)?;
                    }
                }
            }
            write!(o, "\n")?;
        }
        */
        Ok(())
    }

    pub fn print_query(&mut self, q: &str) -> Result<()> {
        let r = Self::query(&mut self.conn, q)?;
        Self::print_result(r)
    }

    pub fn parse_create_table(&mut self, tbl: &str) -> Result<()> {
        let result = Self::query(&mut self.conn, &format!("show create table {}", tbl))?;
        for row in result {
            let mut row = row?;
            let value: Vec<u8> = match row.take_opt(1) {
                Some(v) => v?,
                None => return err_msg("Could not convert"),
            };
            println!("Before parse {}", String::from_utf8_lossy(&value));
            let parsed = nom_sql::parse_query_bytes(&value)?;
            println!("after parse");
            println!("{:#?}", parsed);
            let td = TableDef::try_from(value.as_slice());
            println!("{:#?}", td);
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
        Ok(())
    }

    pub fn print_result(result: mysql::QueryResult<'_>) -> Result<()> {
        use std::io::Write;
        let stdout = std::io::stdout();
        let mut o = stdout.lock();
        // Todo: Determine column widths
        let cols = result.columns_ref();
        let mut i = 0;
        for col in cols {
            if i > 0 {
                write!(o, " | ")?;
            }
            write!(o, "{}", col.name_str())?;
            i += 1;
        }
        writeln!(o, "")?;
        for row in result {
            let row = row?;
            let values = row.unwrap();
            // Reusing i
            i = 0;
            for value in values {
                if i > 0 {
                    write!(o, " | ")?;
                }
                match value {
                    Value::Int(v) => write!(o, "{}", v)?,
                    Value::UInt(v) => write!(o, "{}", v)?,
                    Value::Bytes(v) => write!(o, "{}", String::from_utf8_lossy(&v))?,
                    Value::NULL => write!(o, "NULL")?,
                    Value::Float(v) => write!(o, "{}", v)?,
                    Value::Time(neg, d, h, m, s, ms) => {
                        write!(o, "{} {} {}:{}:{}:{}", neg, d, h, m, s, ms)?;
                    }
                    Value::Date(y, month, d, h, m, s, ms) => {
                        write!(o, "{}-{}-{} {}:{}:{}:{}", y, month, d, h, m, s, ms)?;
                    }
                }
                i += 1;
            }
            writeln!(o, "")?;
        }
        Ok(())
    }

    pub fn query<'a>(conn: &'a mut mysql::Conn, q: &str) -> Result<mysql::QueryResult<'a>> {
        println!("Query: {}", q);
        let result = conn.query(q)?;
        Ok(result)
        /*
        let srv_cols = cols
            .iter()
            .map(|c| msql_srv::Column {
                table: c.table_str().to_string(),
                column: c.name_str().to_string(),
                coltype: c.column_type(),
                colflags: c.flags(),
            })
            .collect::<Vec<_>>();
        let mut row_writer = results.start(&srv_cols).unwrap();
        for row in result {
            let row = row.unwrap();
            let values = row.unwrap();
            row_writer.write_row(values)?;
        }
        Rows {}
        */
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
