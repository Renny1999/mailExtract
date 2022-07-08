extern crate mailbox;

use std::fs::File;
use std::io::{BufReader, Read};

use std::env;

fn main() -> std::io::Result<()>{
    let file_name : &str = "res/test.mbox";
    println!("{}",file_name);

    let mut file : File = match File::open(file_name) {
        Ok(f) => f,
        Err(e) => panic!("Error {}",e),
    };

    let mbox = mailbox::read(file);

    // for mail in mbox {
    //     println!("{:?}", mail);
    // }

    //
    // let mut buf_reader = BufReader::new(file);
    // let mut contents : String = String::new();
    //
    // buf_reader.read_to_string(&mut contents)?;

    // for line in &contents {
    //     println!(line);
    // }
    // println!("{}",contents);


    Ok(())
}
