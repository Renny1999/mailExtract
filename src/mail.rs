use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufRead};

pub struct Mail {
    from: String,
    to: String,
    contents: String,
}

impl std::fmt::Display for Mail {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n{}\n{}", self.from, self.to, self.contents)
    }
}

#[allow(unused)]
impl Mail {
    pub fn get_from(&self) -> &str {
        &(self.from)
    }
    pub fn get_to(&self) -> &str {
        &(self.to)
    }
    pub fn get_contents(&self) -> &str {
        &(self.to)
    }
}

#[allow(unused)]
pub fn parse_mbox(path: &str) -> Vec<Mail> {
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
        let mut char_iter = l.char_indices();
        
        let mut first_four: String = String::with_capacity(20);
        for _i in 0..4 {
            if let Some(character) = char_iter.next() {
                first_four += &(String::try_from(character.1).unwrap());
            }
        }

        if first_four.len() >= 4 && &first_four == "From" {
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

