use crate::coltypes::*;
use crate::er::{err_msg, MyLibError, Result};

use indexmap::IndexMap;

#[derive(Debug)]
pub struct TableDef {
    pub name: String,
    pub fields: IndexMap<String, ColumnDef>,
    // The keys are registered on or
    // moved to the ColumnDef unless the key consist of
    // several cols
    // List of <Vec<Col name>>
    pub unique_keys: Vec<Vec<String>>,
    // Option<Vec<Col name>>
    pub primary_keys: Option<Vec<String>>,
    // <Key name, Vec<Col name>>
    pub index_keys: IndexMap<String, Vec<String>>,
}
#[derive(Debug)]
pub struct ColumnDef {
    pub name: String,
    pub sql_type: nom_sql::SqlType,
    pub renderer: Box<dyn ColType>,
    pub auto_increment: bool,
    pub primary_key: bool,
    pub not_null: bool,
    pub unique: bool,
    pub index: bool,
    pub collate: Option<String>,
    pub charset: Option<String>,
    pub default: Option<nom_sql::Literal>,
}
impl TableDef {
    pub fn select_sql(&self) -> String {
        let fields = self.fields.keys().map(|s| s.as_str()).collect::<Vec<_>>();
        format!("select {} from {}", fields.as_slice().join(", "), self.name)
    }

    pub fn to_sexpr(&self) -> String {
        String::new()
    }
}
use std::convert::TryFrom;
impl TryFrom<&[u8]> for TableDef {
    type Error = MyLibError;
    fn try_from(b: &[u8]) -> Result<TableDef> {
        //println!("Before parse {}", String::from_utf8_lossy(b));
        let parsed = nom_sql::parse_query_bytes(b)?;
        //println!("{:#?}", parsed);
        let stm = match parsed {
            nom_sql::SqlQuery::CreateTable(stm) => stm,
            _ => return err_msg("Expected create table"),
        };
        let mut fields = stm
            .fields
            .into_iter()
            .map(|c: nom_sql::ColumnSpecification| {
                use nom_sql::SqlType;
                let renderer: Box<dyn ColType> = match c.sql_type {
                    SqlType::Bigint(_)
                    | SqlType::Int(_)
                    | SqlType::Tinyint(_) => Box::new(IntCol),
                    SqlType::UnsignedInt(_)
                    | SqlType::UnsignedTinyint(_)
                    | SqlType::UnsignedBigint(_) => Box::new(UIntCol),
                    SqlType::Varchar(_)
                    | SqlType::Text
                    | SqlType::Tinytext
                    | SqlType::Mediumtext
                    | SqlType::Longtext
                    // Not sure how this is encoded coming from server
                    | SqlType::Enum(_)
                    => Box::new(StrCol),
                    SqlType::Date => Box::new(DateCol),
                    SqlType::DateTime(_)
                    | SqlType::Timestamp => Box::new(DateTimeCol),
                    SqlType::Bool => Box::new(BoolCol),
                    SqlType::Decimal(_, _)
                    | SqlType::Float
                    | SqlType::Real
                    | SqlType::Double
                    => Box::new(FloatCol),
                    SqlType::Char(_) => Box::new(StrCol),
                    SqlType::Binary(_)
                    | SqlType::Blob
                    | SqlType::Longblob
                    | SqlType::Mediumblob
                    | SqlType::Tinyblob
                    | SqlType::Varbinary(_)
                    => Box::new(BinCol)

                };
                let mut def = ColumnDef {
                    name: c.column.name,
                    sql_type: c.sql_type,
                    renderer,
                    auto_increment: false,
                    primary_key: false,
                    not_null: false,
                    unique: false,
                    index: false,
                    collate: None,
                    charset: None,
                    default: None,
                };
                for constraint in c.constraints {
                    use nom_sql::ColumnConstraint::*;
                    match constraint {
                        NotNull => def.not_null = true,
                        DefaultValue(literal) => def.default = Some(literal),
                        Collation(collation) => def.collate = Some(collation),
                        AutoIncrement => def.auto_increment = true,
                        PrimaryKey => def.primary_key = true,
                        Unique => def.unique = true,
                        CharacterSet(charset) => def.charset = Some(charset),
                    }
                }
                (def.name.clone(), def)
            })
            .collect::<IndexMap<_, _>>();
        let mut unique_keys = Vec::new();
        let mut primary_keys = None;
        let mut index_keys = IndexMap::new();
        if let Some(keys) = stm.keys {
            for key in keys {
                use indexmap::map::Entry;
                use nom_sql::TableKey::*;
                // These are a bit repetitive, but there was also complexity
                // when I tried to use closures.
                match key {
                    //
                    Key(name, cols) => {
                        if cols.len() == 1 {
                            // Single col
                            if let Some(Entry::Occupied(mut e)) =
                                cols.first().map(|c| fields.entry(c.name.clone()))
                            {
                                e.get_mut().index = true;
                            } else {
                                return err_msg(format!("Failed to get col from: {}", name));
                            }
                        } else {
                            // Multiple cols
                            index_keys.insert(name, cols.into_iter().map(|c| c.name).collect());
                        }
                    }
                    PrimaryKey(cols) => {
                        if cols.len() == 1 {
                            // Single col
                            if let Some(Entry::Occupied(mut e)) =
                                cols.first().map(|c| fields.entry(c.name.clone()))
                            {
                                e.get_mut().primary_key = true;
                            } else {
                                return err_msg("Failed to get primary col");
                            }
                        } else {
                            // Multiple cols
                            primary_keys =
                                Some(cols.into_iter().map(|c| c.name).collect::<Vec<_>>());
                        }
                    }
                    UniqueKey(name, cols) => {
                        if cols.len() == 1 {
                            // Single col
                            if let Some(Entry::Occupied(mut e)) =
                                cols.first().map(|c| fields.entry(c.name.clone()))
                            {
                                e.get_mut().unique = true;
                            } else {
                                return err_msg(format!("Failed to get col from: {:?}", name));
                            }
                        } else {
                            // Multiple cols
                            unique_keys.push(cols.into_iter().map(|c| c.name).collect());
                        }
                    }
                    FulltextKey(_name, _cols) => {
                        // Not implemented
                    }
                }
            }
        }
        let def = TableDef {
            name: stm.table.name,
            fields,
            unique_keys,
            primary_keys,
            index_keys,
        };
        Ok(def)
    }
}
