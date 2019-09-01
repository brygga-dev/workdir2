use msql_srv::*;
use std::io;
use std::net;
use std::thread;

struct Backend {
    conn: mysql::Conn,
}
fn times(msg: &str, d: std::time::Duration) {
    println!("PRX: {}: {}", msg, (d.as_secs() as f32) + (d.as_nanos() as f32) / (1_000_000_000 as f32));
}
impl<W: io::Write> MysqlShim<W> for Backend {
    type Error = io::Error;

    fn on_prepare(&mut self, query: &str, info: StatementMetaWriter<W>) -> io::Result<()> {
        println!("Prepare query: {}", query);
        info.reply(10, &[], &[])
    }

    fn on_execute(
        &mut self,
        id: u32,
        parser: ParamParser,
        results: QueryResultWriter<W>,
    ) -> io::Result<()> {
        println!("On execute");
        results.completed(0, 0)
    }

    fn on_close(&mut self, _: u32) {
        println!("On close");
    }

    fn on_query(&mut self, query: &str, results: QueryResultWriter<W>) -> io::Result<()> {
        let result = self.conn.query(query).unwrap();
        let cols = result.columns_ref();
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
        row_writer.finish()
    }

    // Client selects db
    fn on_init(&mut self, schema: &str, init_writer: InitWriter<W>) -> Result<(), Self::Error> {
        if self.conn.select_db(schema) {
            init_writer.ok()
        } else {
            init_writer.error(msql_srv::ErrorKind::ER_BAD_DB_ERROR, b"Error selecting db")
        }
    }
}


fn main() -> io::Result<()> {
    let listener = net::TcpListener::bind("0.0.0.0:3308")?;
    let mut threads = Vec::new();
    let mut conn_builder = mysql::OptsBuilder::new();
    conn_builder
        .ip_or_hostname(Some("db"))
        .tcp_port(3306)
        .db_name(Some("wordpress"))
        .user(Some("wordpress"))
        .pass(Some("wordpress"));
    let conn_opts = mysql::Opts::from(conn_builder);
    while let Ok((s, _)) = listener.accept() {
        s.set_nodelay(true).unwrap();
        let conn_opts = conn_opts.clone();
        threads.push(thread::spawn(move || {
            let conn = mysql::Conn::new(conn_opts).unwrap();
            MysqlIntermediary::run_on_tcp(Backend { conn }, s).unwrap();
        }));
    }
    for t in threads {
        t.join().unwrap();
    }
    Ok(())
}
