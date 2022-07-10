extern crate mailbox;

use core::fmt;
use std::fs::File;
use std::io::{BufReader, BufRead};

struct Mail {
    from: String,
    to: String,
    contents: String,
}

impl fmt::Display for Mail {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n{}\n{}", self.from, self.to, self.contents)
    }
}

impl Mail {
    fn get_from(&self) -> &str {
        &(self.from)
    }
    fn get_to(&self) -> &str {
        &(self.to)
    }
    fn get_contents(&self) -> &str {
        &(self.to)
    }
}

fn parse_mbox(path: &str) -> Vec<Mail> {
    let file : File = match File::open(path) {
        Ok(f) => f,
        Err(e) => panic!("Error {}",e),
    };

    let reader = BufReader::new(file);
    let mut mail_unit: Mail = Mail {
                                from: String::with_capacity(10),
                                to: String::with_capacity(10),
                                contents: String::new()
    };
    

    let mut mails: Vec<Mail> = Vec::<Mail>::new();

    let mut lines = reader.lines();

    while let Some(line) = lines.next() {
        let l = line.unwrap();

        if l.len() >= 4 && &l[0..4] == "From" {
            if mail_unit.get_from().len() > 0 {
                mails.push(mail_unit);
                mail_unit = Mail {
                    from: String::with_capacity(10),
                    to: String::with_capacity(10),
                    contents: String::new()
                };
            }

            mail_unit.from = l;
            match lines.next() {
                Some(res) => mail_unit.to = res.unwrap(),
                None => continue,
            }
        }else{
            mail_unit.contents += &l;
            mail_unit.contents += "\n";
        }
    }
    if mail_unit.get_from().len() > 0 {
        mails.push(mail_unit);
    }

    mails
}

fn main() -> std::io::Result<()>{
    let file_name : &str = "res/test.mbox";
    println!("{}",file_name);
    
    let mails: Vec<Mail> = parse_mbox(file_name);

    for mail in mails {
        println!("{}", mail.get_from());
    }

    Ok(())
}
