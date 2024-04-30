use std::{fs::File, io::Write, process::ExitCode};

use reqwest;


pub fn main() -> ExitCode{
    // let url = "https://checkerproxy.net/archive/2024-04-16";
    // let url = "https://checkerproxy.net/dist/app.57112776e884f15d8518.js";
    let url = "https://checkerproxy.net/dist/vendor.f77040baef66ee31ae0d.js";
    let get_res = reqwest::blocking::get(url).unwrap();

    let mut file = File::create("vendor.js").unwrap();

    file.write_all(get_res.text().unwrap().as_bytes()).unwrap();

    // println!("{:#?}", get_res.text().unwrap());

    ExitCode::SUCCESS
}
