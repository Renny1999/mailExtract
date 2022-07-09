extern crate mailbox;

use std::fs::File;
use std::io::{BufReader, BufRead};

struct Mail {
    from: String,
    to: String,
    contents: String,
}

fn main() -> std::io::Result<()>{
    let file_name : &str = "res/test.mbox";
    println!("{}",file_name);

    let mut file : File = match File::open(file_name) {
        Ok(f) => f,
        Err(e) => panic!("Error {}",e),
    };

    let reader = BufReader::new(file);
    let mut mail_unit: Mail = Mail{
                                from: String::with_capacity(10),
                                to: String::with_capacity(10),
                                contents: String::new()
    };
    for line in reader.lines() {
        let l = line.unwrap();
        if (l.len() >= 4 && &l[0..4] == "From") {
            mail_unit.from = l;
        }
        mail_unit += &l;
        // mail_unit += "\n";
    }

    // println!("{}", mail_unit);
    


    Ok(())
}
