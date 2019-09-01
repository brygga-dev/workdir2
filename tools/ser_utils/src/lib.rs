pub mod php;
pub mod json;
pub mod php_json;
pub mod sexpr;

#[cfg(test)]
mod tests {
    use crate::php;

    fn test_round(orig_serialized: &[u8]) {
        let t = php::deserialize(orig_serialized);
        println!("Deserialize original:");
        println!("{:#?}", &t);
        let t = t.unwrap();
        let s = php::serialize(t.clone(), String::with_capacity(orig_serialized.len()));
        /*
        use std::fs::File;
        let mut tmp = File::create("temp2.txt").unwrap();
        use std::io::Write;
        tmp.write_all(s.as_bytes()).unwrap();*/
        let orig_utf8 = String::from_utf8_lossy(orig_serialized);
        //println!("Then serialized to:");
        //println!("{:#?}", s);
        assert!(s == orig_utf8);
        let t2 = php::deserialize(s.as_bytes());
        //println!("Now deserialized again");
        //println!("{:#?}", &t2);
        assert!(t == t2.unwrap());
    }
    #[test]
    fn test_round1() {
        // select * from wp_options where option_name='active_plugins'
        let orig_serialized = b"a:2:{i:0;s:17:\"brygga/brygga.php\";i:1;s:13:\"pods/init.php\";}";
        test_round(orig_serialized);
        let test2 = r#"a:3:{s:9:"sandboxed";b:0;s:8:"location";a:1:{s:2:"ip";s:12:"192.168.33.0";}s:6:"events";a:2:{i:0;a:7:{s:4:"type";s:8:"wordcamp";s:5:"title";s:11:"WordCamp US";s:3:"url";s:29:"https://2019.us.wordcamp.org/";s:6:"meetup";s:0:"";s:10:"meetup_url";s:0:"";s:4:"date";s:19:"2019-11-01 00:00:00";s:8:"location";a:4:{s:8:"location";s:18:"St. Louis, MO, USA";s:7:"country";s:2:"US";s:8:"latitude";d:38.6532135;s:9:"longitude";d:-90.3136733;}}i:1;a:7:{s:4:"type";s:6:"meetup";s:5:"title";s:39:"Oslo WordPress Meetup September edition";s:3:"url";s:68:"https://www.meetup.com/Oslo-WordPress-Meetup-Group/events/263803514/";s:6:"meetup";s:21:"Oslo WordPress Meetup";s:10:"meetup_url";s:51:"https://www.meetup.com/Oslo-WordPress-Meetup-Group/";s:4:"date";s:19:"2019-09-10 17:00:00";s:8:"location";a:4:{s:8:"location";s:12:"Oslo, Norway";s:7:"country";s:2:"no";s:8:"latitude";d:59.910057067871;s:9:"longitude";d:10.746315956116;}}}}"#;
        let _t =  php::deserialize(test2.as_bytes()).unwrap();
        //println!("{:#?}", t);
        test_round(test2.as_bytes());
    }
    /*
    #[test]
    fn test_round2() {
        use std::io::prelude::*;
        use std::fs::File;
        let mut tmp = File::open("temp.txt").unwrap();
        let mut s = String::new();
        tmp.read_to_string(&mut s).unwrap();
        test_round(s.as_bytes());
    }
    */
}