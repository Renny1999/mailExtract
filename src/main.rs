mod mail;

use std::env;
use std::error::Error;
use eml_parser::errors::EmlError;
use eml_parser::parser::EmlParser;

fn main() -> std::io::Result<()>{
    // let args: Vec<String> = env::args().collect();
    //
    // let file_name : &str = &args[1];

    // let file_name : &str = "res/test.mbox";
    // println!("{}",file_name);
    // 
    // let mails: Vec<mail::Mail> = mail::parse_mbox(file_name);
    //
    // // for mail in mails {
    // //     println!("{}", mail.get_from());
    // // }
    //
    //
    // println!("{}", mails[0]);
    // println!("______________________");
    // println!("{}", mails[1]);
    //
    let file_name = "res/eml/■重要【ECOIS】水光熱運用異常 のお知らせ.eml";

    let mut eml;
    
    match EmlParser::from_file(file_name) {
        Ok(res) => eml = res,
        Err(e) => panic!("{} error parsing file", e),
    }

    if let Ok(parsed) = eml.parse() {
        println!("{:?}", parsed.body);
    } else {
        println!("Failed to parse");
    }


    Ok(())
}
