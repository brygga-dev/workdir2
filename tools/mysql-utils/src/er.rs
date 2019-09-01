use std::fmt;

pub type Result<T> = std::result::Result<T, MyLibError>;

pub enum MyLibError {
    MySql(mysql::error::Error),
    Io(std::io::Error),
    Convert(mysql::FromValueError),
    Msg(String),
}
impl std::error::Error for MyLibError {

}

pub fn err_msg<T>(msg: impl Into<String>) -> Result<T> {
    Err(MyLibError::Msg(msg.into()))
}

pub fn error_msg(msg: impl Into<String>) -> MyLibError {
    MyLibError::Msg(msg.into())
}

impl fmt::Debug for MyLibError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Msg(msg) => write!(f, "{}", msg),
            Self::MySql(e) => write!(f, "Mysql: {:?}", e),
            Self::Io(e) => write!(f, "Io: {:?}", e),
            Self::Convert(e) => write!(f, "Convert: {:?}", e),
        }
    }
}
impl fmt::Display for MyLibError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<&str> for MyLibError {
    fn from(e: &str) -> Self {
        MyLibError::Msg(e.to_string())
    }
}

impl From<mysql::FromValueError> for MyLibError {
    fn from(e: mysql::FromValueError) -> Self {
        MyLibError::Convert(e)
    }
}

impl From<mysql::error::Error> for MyLibError {
    fn from(e: mysql::error::Error) -> Self {
        MyLibError::MySql(e)
    }
}
impl From<std::io::Error> for MyLibError {
    fn from(e: std::io::Error) -> Self {
        MyLibError::Io(e)
    }
}
