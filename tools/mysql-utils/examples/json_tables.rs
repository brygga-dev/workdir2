use mysql_utils::{er::Result, Db};

fn times(msg: &str, d: std::time::Duration) {
    println!(
        "{}: {}",
        msg,
        (d.as_secs() as f32) + (d.as_nanos() as f32) / (1_000_000_000 as f32)
    );
}

fn main() -> Result<()> {
    let mut db = Db::new("127.0.0.1", 3307, "wordpress", "wordpress", "wordpress")?;
    db.print_query("show tables")?;
    db.init_table_defs()?;
    //println!("{:?}", db.table_defs);
    //db.table_to_json("wp_options")?;
    let before = std::time::Instant::now();
    db.tables_to_json_files()?;
    times("Json data", before.elapsed());
    Ok(())
}
