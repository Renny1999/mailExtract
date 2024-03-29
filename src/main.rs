mod mail;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::fmt;
// use std::error::Error;
// use eml_parser::errors::EmlError;
use eml_parser::parser::EmlParser;
use std::str::from_utf8;
use chrono::NaiveDateTime;
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

fn extract_body(s: &str) -> String {

  let mut mystr = String::from("");

  let lines: Vec<&str> = break_string(s);


  for l in lines {
    if l.contains("--==") { continue; }
    if l.contains("Content-Type: text/plain; charset=\"utf-8\"") { continue; }
    if l.contains("MIME-Version:") { continue; }
    if l.contains("Content-Transfer-Encoding: base64") { continue; }
    if l == "\n" { continue; }
    if l == "\r" { continue; }

    let newstr = String::from(l).replace("\r", "");

    mystr = mystr + &newstr;
  }

  mystr
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

#[allow(unused)]
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
fn summarize(stores: &Vec::<EmailData>) 
    -> (Vec<String>, HashMap<String, Vec::<u32>>){
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
                    Usage::ElectricityAfter =>  cols[4]+=1,
                    Usage::WaterAfter =>        cols[5]+=1,
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
    for (name, usage) in &data {
        let mut x: Vec<String> = usage.iter().map(|x| format!("{}",x)).collect();

        x.insert(0, num.to_string());
        x.insert(1, name.to_string());
        x.push(String::from(""));

        let row = make_row(&x);
        println!("{}", row);
        res.push(row);
        num+=1;

    }

    let mut sums = vec![0; 7];

    for (_key,vals) in &data {
        sums[0] += vals[0];
        sums[1] += vals[1];
        sums[2] += vals[2];
        sums[3] += vals[3];
        sums[4] += vals[4];
        sums[5] += vals[5];
        sums[6] += vals[6];
    }

    let footer = 
        vec!["".to_string(), "総計".to_string(),
        sums[0].to_string(),
        sums[1].to_string(),
        sums[2].to_string(),
        sums[3].to_string(),
        sums[4].to_string(),
        sums[5].to_string(),
        sums[6].to_string(),
        "".to_string()];

    res.push(make_row(&footer));

    (res,db)
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
    parsed_date: Option<NaiveDateTime>,
    usage_text: Vec<String>,
    issue: Vec<(Option<Usage>, String)>,
}


impl std::fmt::Display for EmailData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut issues: String = String::new();
        for (u,_v) in &self.issue{
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

fn parse_date(date_string: &str) -> Option<chrono::NaiveDateTime> {
    let mut cleaned = String::new();
    let mut pause: bool = false;
    for c in date_string.chars() {
        if c == '(' {
            pause = true;
            continue;
        }
        if c == ')' {
            pause = false;
            continue;
        }
        if pause == true {
            continue;
        }
        cleaned += &(c.to_string());
    }

    cleaned = cleaned + "0:0:0";

    match chrono::NaiveDateTime:: parse_from_str(
            &cleaned.to_string(),
            "日時：%Y年%m月%d日 %H:%M:%S") {
        Ok(res) => Some(res),
        Err(_) => None,
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 2);
    let src_dir = &args[1];

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

    let mut output = fs::File::create(format!("output.csv"))?;

    let mut stores :Vec<EmailData> = Vec::<EmailData>::new();

    /* works through eml files */
    for f in &eml_files {
        let mut eml;
        match EmlParser::from_file(f) {
            Ok(res) => eml = res,
            Err(e) => panic!("{} error parsing file", e),
        }

        /* parse the email */
        let eml = eml.parse().unwrap();

        let extracted_body = extract_body(&(eml.body).as_ref().unwrap());

        /* get the text body to a workable state */
        let encoded_body: String = clean_string(&extracted_body);

        let decoded_bytes: Vec<u8>;


        // println!("encoded_body: {}", &encoded_body);

        base64::decode(&encoded_body).unwrap();
        if let Ok(v) = base64::decode(encoded_body) {
          decoded_bytes = v;
        }
        else {
          println!("Error parsing {}", &f);
          continue;
        }
        let decoded_body = from_utf8(&decoded_bytes).unwrap().to_string();

        let line_by_line = break_string(&decoded_body);

        let mut store = EmailData {
            company:  line_by_line[0].to_string().replace("\r", ""),
            store:    line_by_line[1].to_string().replace("\r", ""),
            id:       line_by_line[2].to_string().replace("\r", ""),
            name:     line_by_line[8].to_string().replace("\r", ""),
            date:     line_by_line[9].to_string().replace("\r", ""),
            parsed_date: parse_date(&line_by_line[9].to_string().replace("\r", "")),
            usage_text: Vec::new(),
            issue: Vec::<(Option<Usage>, String)>::new(),
        };

        // parse_date(&store.date);

        let mut starti = 0;
        let mut counter = 0;
        /* get to the first line of the issues block*/
        for line in &line_by_line {
            if line.find("症状：").is_some() {
                store.usage_text.push(String::from(*line));
                starti = counter+1;
                break;
            }
            counter+=1;
        }

        let it = &mut line_by_line[starti..].iter();

        // sort the issues into corresponding bins
        while let Some(line) = it.next() {
            let mut usage_type: Option<Usage> = None;
            if line.len() < 3{
                break;
            }

            store.usage_text.push(String::from(*line));

            /* after close */
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

            /* get the usage percentage */
            if usage_type.is_some() {
                match it.next() {
                    Some(line) => {
                        /* this is the usage info */
                        store.issue.push((usage_type, (line.to_string())));
                        store.usage_text.push(String::from(*line));
                    },
                    None => {
                        break;
                    }
                }
            }
        }
        stores.push(store);
    }
    
    // sort the stores by name, then date
    stores.sort_by(
        |x, y|{
            if &x.name == &y.name {
                if x.parsed_date.is_some() && y.parsed_date.is_none() {
                    // x > y
                    Ordering::Greater
                } 
                else if x.parsed_date.is_none() && y.parsed_date.is_some() {
                    Ordering::Less
                }
                else if x.parsed_date.is_none() && y.parsed_date.is_none() {
                    Ordering::Equal
                }else {
                    (&x.parsed_date).partial_cmp(&y.parsed_date).unwrap()
                }
            }else{
                (&x.name).partial_cmp(&y.name).unwrap()
            }
        });


    for store in &stores {
        let formatted:String;

        // put usage text for a store into columns
        formatted = make_row(&store.usage_text).replace("\r", "");
        let res = format!("{},{},{}\n", store.name, store.date,formatted);

        output.write(res.as_bytes())?;
    }

    let (summary, _db) = summarize(&stores);
    output.write("\n".as_bytes())?;

    // write out summary
    for line in summary {
        output.write((line+"\n").as_bytes())?;
    }

    

    Ok(())
}
