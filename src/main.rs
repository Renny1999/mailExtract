use std::fs::File;

fn main() => Result{
    let file_name : &str = "res/test.mbox";
    let fstream : File = File::open(file_name)?;

}
