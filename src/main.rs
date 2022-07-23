mod mail;

use std::array;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::fmt;
use std::io::LineWriter;
// use std::error::Error;
// use eml_parser::errors::EmlError;
use eml_parser::parser::EmlParser;
use std::str::from_utf8;
use std::collections::hash_map;

use base64;

fn get_files(dir: &str) -> Vec<String> {
    let d = fs::read_dir(dir).unwrap();
    let mut input_files: Vec<String> = Vec::new();

    for file_path in d {
        let p = file_path.as_ref().unwrap().path().display().to_string();

        input_files.push(p);
    }

    input_files
}

fn clean_string(s: &str) -> String {
    let s_copy = String::from(s);
    let data = String::from(
        from_utf8(&(s_copy.replace("\n", "").replace("\r", "").into_bytes())).unwrap(),
    );
    data
}

/*
 * the input is a str slice, and I want to reference each line in the str slice
 * and save them into a vector
 * */
fn break_string(s: &str) -> Vec<&str> {
    let mut res: Vec<&str> = Vec::new();
    let it = s.char_indices();
    let mut start: usize = 0;
    for (i, v) in it {
        if v == '\n' {
            res.push(&s[start..i]);
            start = i + 1;
        }
    }
    res
}

struct Data <'b>{
    name: &'b str,
    electricity: u32,
    electricity_after: u32,
    water: u32,
    water_after: u32,
    gas: u32,
    gas_after: u32,
    notes: String,
}


fn make_row(v: &Vec<String>) -> String {
    let mut row: String = String::new();

    for s in &v[0..v.len()-1] {
        row += &format!("{},",s).to_string();
    }
    row+=&format!("{}",v[v.len()-1]).to_string();

    row
}

/* take the stores and get the useful info */
fn summarize(stores: &Vec::<EmailData>) -> Vec<String>{
    let mut db: HashMap<String, Vec::<u32>> = HashMap::<String, Vec<u32>>::new();
    let mut res = Vec::<String>::new();
    let ncols = 7;
    let key = "施設名：";

    for email in stores {
        let name: String;
        match email.name.find(key) {
            Some(i) => name = email.name[i+key.len()..].replace("\r", ""),
            None => name="".to_string(),
        }

        let cols: &mut Vec::<u32>; 
        if db.contains_key(&name) {
            cols = db.get_mut(&name).unwrap();
        } 
        else {
            let c = vec![0;ncols];
            db.insert(name.to_string(), c);
            cols = db.get_mut(&name).unwrap();
        }
         
        for (issue,_v) in &email.issue {
            if issue.is_some() {
                match issue.as_ref().unwrap() {
                    Usage::Electricity =>       cols[1]+=1,
                    Usage::Water =>             cols[2]+=1,
                    Usage::Gas =>               cols[3]+=1,
                    Usage::WaterAfter =>        cols[4]+=1,
                    Usage::ElectricityAfter =>  cols[5]+=1,
                    Usage::GasAfter =>          cols[6]+=1,
                }

                cols[0] += 1;
            }
        }
    }
    let header = vec!["No", "店舗名", "延べ回数", "運用(電気)", "運用(水道)",
    "運用(ガス)","閉店時(電気)","閉店時(水道)","閉店時(ガス)","備考"];
    let s = make_row(&header.iter().map(|x| x.to_string()).collect());

    res.push(s);
    

    let mut data: Vec<(&String, &Vec<u32>)> = db.iter().collect();
    data.sort_by_key(|pair| pair.0);

    let mut num = 1;
    for (name, usage) in data {
        let mut x: Vec<String> = usage.iter().map(|x| format!("{}",x)).collect();

        x.insert(0, num.to_string());
        x.insert(1, name.to_string());
        x.push(String::from(""));

        let row = make_row(&x);
        println!("{}", row);
        res.push(row);
        num+=1;

    }

    res
}

#[derive(Debug)]
enum Usage {
    Water,
    WaterAfter,
    Electricity,
    ElectricityAfter,
    Gas,
    GasAfter,
}

#[derive(Debug)]
struct EmailData{
    company: String,
    store: String,
    id: String,
    name: String,
    date: String,
    // issue: Vec<&'b str>,
    issue: Vec<(Option<Usage>, String)>,
}

impl std::fmt::Display for EmailData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut issues: String = String::new();
        for (u,v) in &self.issue{
            if let Some(usage) = u {
                match *usage{
                    Usage::Water => issues += "Water",
                    Usage::WaterAfter => issues += "WaterAfter",
                    Usage::Electricity => issues += "Electricity",
                    Usage::ElectricityAfter => issues += "ElectricityAfter",
                    Usage::Gas => issues += "Gas",
                    Usage::GasAfter => issues += "GasAfter",
                }
                issues += "\n";
            }
        }
            
        write!(f, 
               "company: {}\n\
                store: {}\n\
                Id: {} \n\
                name: {}\n\
                date: {}\n\
                issue: {}\n", 
                self.company, self.store, self.id, self.name, self.date, issues)
    }
}


fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 3);
    let src_dir = &args[1];
    let dest_dir = &args[2];

    let input_files = get_files(src_dir);

    let eml_files: Vec<String> = input_files
        .into_iter()
        .filter(|x| {
            let ext = Path::new(x).extension();
            if (ext.is_some()) && (ext.unwrap() == "eml") {
                return true;
            } else {
                return false;
            }
        })
        .collect();

    println!("found {} .eml files", eml_files.len());

    let dest_dir = fs::read_dir(dest_dir).unwrap();
    let mut output = fs::File::create(format!("output.csv"))?;

    let mut stores :Vec<EmailData> = Vec::<EmailData>::new();

    /* works through eml files */
    for f in eml_files {
        let mut eml;
        match EmlParser::from_file(f) {
            Ok(res) => eml = res,
            Err(e) => panic!("{} error parsing file", e),
        }

        /* parse the email */
        let eml = eml.parse().unwrap();

        /* get the text body to a workable state */
        let encoded_body: String = clean_string(&(eml.body).unwrap());
        let decoded_bytes: Vec<u8> = base64::decode(encoded_body).unwrap();
        let decoded_body = from_utf8(&decoded_bytes).unwrap().to_string();

        let line_by_line = break_string(&decoded_body);

        let mut store = EmailData {
            company: line_by_line[0].to_string(),
            store: line_by_line[1].to_string(),
            id: line_by_line[2].to_string(),
            name: line_by_line[8].to_string(),
            date: line_by_line[9].to_string(),
            issue: Vec::<(Option<Usage>, String)>::new(),
        };

        let mut starti = 0;
        let mut counter = 0;
        /* get the issues */
        for line in &line_by_line {
            if line.find("症状：").is_some() {
                starti = counter+1;
                break;
            }
            counter+=1;
        }

        let it = &mut line_by_line[starti..].iter();


        while let Some(line) = it.next() {
            let mut usage_type: Option<Usage> = None;
            if line.len() < 3{
                break;
            }
            /* after close */
            let t:Vec<char>= line.char_indices().map(|x| x.1).collect();
            if let Some(_) = &line.find("閉店後"){
                if line.find("水道").is_some() {
                    usage_type = Some(Usage::WaterAfter);
                }
                if line.find("電気").is_some() {
                    usage_type = Some(Usage::ElectricityAfter);
                }
                if line.find("ガス").is_some() {
                    usage_type = Some(Usage::Gas);
                }
            } 

            /* before close */
            else {
                if line.find("水道").is_some() {
                    usage_type = Some(Usage::Water);
                }
                if line.find("電気").is_some() {
                    usage_type = Some(Usage::Electricity);
                }
                if line.find("ガス").is_some() {
                    usage_type = Some(Usage::GasAfter);
                }

            }
            if usage_type.is_some() {
                match it.next() {
                    Some(line) => {
                        /* this is the usage info */
                        store.issue.push((usage_type, (line.to_string())));
                    },
                    None => {
                        break;
                    }
                }
            }
        }
        stores.push(store);
    }

    let summary = summarize(&stores);
    for line in summary {
        output.write((line+"\n").as_bytes())?;
    }


    Ok(())
}
