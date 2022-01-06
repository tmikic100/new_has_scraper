#[macro_use] extern crate lazy_static;
extern crate regex;
use std::env;
use regex::Regex;
use std::fs;
use std::io;
use std::path::Path;
use scraper::{Selector, Html};
use term_table::table_cell::TableCell;
use term_table::row::Row;
use indicatif::{MultiProgress, ProgressBar};
use std::collections::HashSet;
use std::hash::Hash;
use std::cmp::Eq;
use std::net::SocketAddr;
use std::time::Instant;
fn calculate_result(result:f64) -> String{
	let h = (result/(60.00*60.00));
	let h = h.floor();
	let min = ((result%(60.00*60.00))/60.00).floor();
	let sec = result%60.00;
	let sec = ((sec*100.00).round())/100.00;
	let mut sresult = "".to_string();
	if h == 0.00{
		if min != 0.00{
			if sec != 0.00{
				if sec >= 10.00{
					sresult = format!("{}:{:.2}",min,sec);
				}else{
					sresult = format!("{}:0{:.2}",min,sec);
				}
			}else{
				sresult = format!("{}:00,00",min);
			}
		}else{
			sresult = format!("{:.2}",sec);
		}
	}else{
		if min != 0.00{
			if min >= 10.00{
				if sec != 0.00{
					if sec >= 10.00{
						sresult = format!("{}:{}:{:.2}",h,min,sec);
					}else{
						sresult = format!("{}:{}:0{:.2}",h,min,sec);
					}
				}else{
					sresult = format!("{}:{}:00,00",h,min);
				}
			}else{
				if sec != 0.00{
					if sec >= 10.00{
						sresult = format!("{}:0{}:{:.2}",h,min,sec);
					}else{
						sresult = format!("{}:0{}:0{:.2}",h,min,sec);
					}
				}else{
					sresult = format!("{}:0{}:00,00",h,min);
				}
			}
		}else{
			if sec >= 10.00{
				sresult = format!("{}:00:{:.2}",h,sec);
			}else{
				sresult = format!("{}:00:0{:.2}",h,sec);
			}
		}
	}
	sresult.replace(".",",")	
}
fn get_result(text:String) -> f64{
	let text = text.replace(";",":");
	let minutes = text.split(":").collect::<Vec<_>>();
	let mut result:f64 = 0.0;
	if minutes.len() == 1{
		result = minutes[0].replace(",",".").parse::<f64>().unwrap();
	}else if minutes.len() == 2{
		result = minutes[0].parse::<f64>().unwrap()*60.0+minutes[1].replace(",",".").parse::<f64>().unwrap();
	}else{
		result = minutes[0].parse::<f64>().unwrap()*60.0*60.0+minutes[1].parse::<f64>().unwrap()*60.0+minutes[2].replace(",",".").parse::<f64>().unwrap();
	}
	result
}
fn format_result(text:String) -> String{
	let minutes = text.split(":").collect::<Vec<_>>();
	let mut result:f64 = 0.0;
	if minutes.len() == 1{
		result = minutes[0].replace(",",".").parse::<f64>().unwrap();
	}else if minutes.len() == 2{
		result = minutes[0].parse::<f64>().unwrap()*60.0+minutes[1].replace(",",".").parse::<f64>().unwrap();
	}else{
		result = minutes[0].parse::<f64>().unwrap()*60.0*60.0+minutes[1].parse::<f64>().unwrap()*60.0+minutes[2].replace(",",".").parse::<f64>().unwrap();
	}
	calculate_result(result)
}

//https://github.com/GlaivePro/IaafPoints/blob/master/LICENSE.md
//uzimanje magicnih brojeva iz csv-a tako da se moze izracunati IAAF-ovi (WA) bodovi
fn get_magic_numbers(mut search: String) -> Vec<String>{
	let contents = fs::read_to_string("magicnumbers.csv")
    .expect("Something went wrong reading the file");
	let mut array = Vec::new();
	if search.contains("x") || search.contains("0"){
		search = search + &"m".to_string();
		search = search.replace("mm","m").replace("000mWm","kmW").replace("000Wm","kmW").replace("kmWm","kmW").replace("mHm","mH").replace("Hm","mH").replace("mSCm","mSC").replace("SCm","mSC");
	}
	for x in contents.split("\n"){
		array.push(x.split(",").collect::<Vec<_>>());
	}
	let mut res = Vec::new();
	for x in array{
		if search.to_lowercase() == x[0].to_lowercase(){
			for y in x{
				res.push(y.trim().to_string());
			}
			break;
		}
	}
	res
}
//izracunjavanje IAAF-ovih bodova
fn calculate_points(result: f64, discip: String, age: String, year: String)-> String{
	let mut search = get_age_data(age,year) + &get_discipline_alias(discip);
	let magic = get_magic_numbers(search);
	let mut points:f64 = 0.0;
	if magic.len() == 4{
		let resultShift:f64 = magic[1].parse().unwrap();
		let conversionFactor:f64 = magic[2].parse().unwrap();
		let pointShift:f64 = magic[3].parse().unwrap();
		points = conversionFactor * (result + resultShift).powf(2.0) + pointShift;
		points = points.floor();
	}
	points.to_string()
}
fn calculate_points_manual(result: String, search: String)-> String{
	let magic = get_magic_numbers(search);
	let mut points:f64 = 0.0;
	let result = get_result(result);
	if magic.len() == 4{
		let resultShift:f64 = magic[1].parse().unwrap();
		let conversionFactor:f64 = magic[2].parse().unwrap();
		let pointShift:f64 = magic[3].parse().unwrap();
		points = conversionFactor * (result + resultShift).powf(2.0) + pointShift;
		points = points.floor();
	}
	points.to_string()
}
fn display(data: Vec<Vec<Vec<String>>>){
	println!("");
	for x in 0..data.len(){
		let mut table = term_table::Table::new();
		if data[x].len() > 1{
			if data[x][1][0] == "Discipline" 	{
				table.max_column_width = 60;
				table.style = term_table::TableStyle::extended();
				table.add_row(Row::new(vec![
					TableCell::new_with_alignment(data[x][0][2].clone() + &" ".to_owned() + &data[x][0][1].clone()   , 5, term_table::table_cell::Alignment::Center)
				]));
				for y in 1..data[x].len(){
					let mut rows = Vec::new();
					let mut added = false;
					if data[x].len() != 1{
						for z in 0..data[x][y].len() {
							rows.push(TableCell::new(data[x][y][z].clone()));
							added = true;
						}
					}else{	
						table.add_row(Row::new(vec![
							TableCell::new_with_alignment(data[x][y][0].clone() , 5, term_table::table_cell::Alignment::Center)
						]));
					}
					if added == true{
						table.add_row(Row::new(rows));
					}
				}
			}else if data[x][0][data[x][0].len()-2] == "PB" || data[x][0][1]== "Discipline"{
				table.max_column_width = 60;
				table.style = term_table::TableStyle::extended();
				for y in 0..data[x].len(){
					let mut rows = Vec::new();
					let mut added = false;
					for z in 0..data[x][y].len() {
						rows.push(TableCell::new(data[x][y][z].clone()));
						added = true;
					}
					if added == true{
						table.add_row(Row::new(rows));
					}
				}
			}else{
				table.max_column_width = 60;
				table.style = term_table::TableStyle::extended();
				if data[x][0][3] == "single" || data[x][0][3] == "wind"{
					table.add_row(Row::new(vec!("Rank".to_string(),"Mark".to_string(),"Wind".to_string(),"Name".to_string(),"Birthday".to_string(),"Club".to_string(),"City".to_string(),"Date".to_string())));
				}else if data[x][0][3] == "multi"{
					table.add_row(Row::new(vec!("Rank".to_string(),"Mark".to_string(),"Name".to_string(),"Birthday".to_string(),"Club".to_string(),"City".to_string(),"Date".to_string(),"Results".to_string())));				
				}else if data[x][0][3] == "relay"{
					table.add_row(Row::new(vec!("Rank".to_string(),"Mark".to_string(),"Name".to_string(),"City".to_string(),"Date".to_string(),"Runner 1".to_string(),"Runner 2".to_string(),"Runner 3".to_string(),"Runner 4".to_string())));					
				}
				for y in 1..data[x].len(){
					let mut rows = Vec::new();
					let mut added = false;
					if !data[x][y][0].contains("/"){
						for z in 0..data[x][y].len() {
							rows.push(TableCell::new(data[x][y][z].clone()));
							added = true;
						}
					}else{	
						table.add_row(Row::new(vec![
							TableCell::new_with_alignment(data[x][y][0].clone() , 8, term_table::table_cell::Alignment::Center)
						]));
					}
					if added == true{
						table.add_row(Row::new(rows));
					}
				}					
			}
		println!("{}", table.render());
		}
	}
}
fn save_csv(data:Vec<Vec<Vec<String>>>){
	let mut csv:String = String::new();
	let mut name:String = String::new();
	csv += "sep=~\n";
	for x in 0..data.len(){
		if data[x].len() > 1{
			if data[x][1][0] == "Discipline"{
				name = data[x][0][2].clone();
				csv += &(data[x][0][2].to_string() +" "+&data[x][0][1] + "\n");
				for y in 1..data[x].len(){
					for z in 0..data[x][y].len()-1{
						csv += &(data[x][y][z].to_string() + "~");
					}
					csv += &(data[x][y][data[x][y].len()-1].to_string() + "\n");
				}
			}else if data[x][0][data[x][0].len()-2] == "PB"{
				name = data[x][0][0].clone();
				for y in 0..data[x].len(){
					for z in 0..data[x][y].len()-1{
						csv += &(data[x][y][z].to_string() + "~");
					}
					csv += &(data[x][y][data[x][y].len()-1].to_string() + "\n");
				}
			}else{
				if data[x][0][3] == "single"{
					csv += &("Rank~Mark~Wind~Name~Birthday~Club~City~Date\n");
				}else if data[x][0][3] == "multi"{
					csv += &("Rank~Mark~Name~Birthday~Club~City~Date~Results\n");
				}else if data[x][0][3] == "relay"{
					csv += &("Rank~Mark~Name~City~Club~Runner 1~Runner 2~Runner 3~Runner 4");
					
				}
				for y in 1..data[x].len(){
					for z in 0..data[x][y].len()-1{
						csv += &(data[x][y][z].to_string() + "~");
					}
					csv += &(data[x][y][data[x][y].len()-1].to_string() + "\n");
				}			
			}
			csv += &"\n";
		}
	}
	fs::create_dir_all("/has/export/".to_owned() + &data[0][0][0].clone()+&*"/");
	fs::write("/has/export/".to_owned() + &data[0][0][0].clone()+&*"/"+&name.clone() + &*".csv", csv.clone()).expect("Unable to write file");
}
fn save_stats(data:Vec<Vec<Vec<String>>>, key:String){
	let mut csv:String = String::new();
	let mut name:String = String::new();
	for x in 0..data.len(){
		for y in 0..data[x].len(){
			for z in 0..data[x][y].len()-1{
				csv += &(data[x][y][z].to_string() + "~");
			}
			csv += &(data[x][y][data[x][y].len()-1].to_string() + "\n");
		}
		csv += &"\n";
	}
	fs::create_dir_all("/has/export/statistics/");
	fs::write("/has/export/statistics/".to_owned() + &key.clone() + &*".csv", csv.clone()).expect("Unable to write file");
}
//funkcija za CLI za unos podataka

pub fn CLI_input(question:&str) -> String {
	let mut ans:String= "".to_string();
	println!("{}", question);
	io::stdin()
		.read_line(&mut ans)
		.expect("Failed to read line");
	let ans = ans.replace("\n","").replace("\r","");
	ans
}
//funkcija za CLI pitanja
pub fn CLI_question(question:&str, mut answers:Vec<String>, all: bool) -> String{
	let mut ans:String= "".to_string();
	println!("{}", question);
	if answers.len() == 2{
		println!("{} or {}",answers[0],answers[1]);
	}else{
		for x in 0..answers.len(){
			println!("{}",answers[x]);
		}
	}
	if all == true{
		println!("all");
		answers.push("all".to_string());
	}
	let mut response = "".to_string();
	loop{
		let mut ans = "".to_string();
		io::stdin()
			.read_line(&mut ans)
			.expect("Failed to read line");

		let ans:String = ans.to_lowercase().split_whitespace().collect();
		for x in 0..answers.len(){
			let comp:String = answers[x].to_lowercase().split_whitespace().collect();
			if ans == comp{
				let ans = answers[x].split_whitespace().collect();
				return ans;
			}
		}
		println!("Invaild answer");
	}
}
//funkcija za CLI pitanja ali maskiranje izlaza s drugim imenima
fn CLI_question_alias(question:&str, answers:Vec<String>, alias:Vec<Vec<String>>) -> String{
	let mut ans= "".to_string();
	let mut answer = "".to_string();
	println!("{}", question);
	for x in 0..alias.len() {
		let mut pat = String::new();
		for y in 0..alias[x].len()-1{
			pat.push_str(&*format!("{} or ", alias[x][y]));
		}
		pat.push_str(&*format!("{}", alias[x][alias[x].len()-1]));
		println!("{}", pat);
	}
	println!("all");
	let mut res = 0;
	'main:loop{
		let mut ans= "".to_string();
		io::stdin()
			.read_line(&mut ans)
			.expect("Failed to read line");

		let mut ans:String = ans.to_lowercase().split_whitespace().collect();
		for x in 0..alias.len(){
			for y in 0..alias[x].len() {
				let comp:String = alias[x][y].to_lowercase().split_whitespace().collect();
				if ans == comp {
					res = x;
					break 'main;
				}
				if ans == "all"{
					res = answers.len()+1;
					break 'main;
				}
			}
		}
		println!("Invaild answer");
	}
	if res != answers.len()+1{
		answers[res].clone()
	}else{
		"all".to_string()
	}
}
//omogucuje uzimanje magicnih brojeva pomocu kategorije i godine
pub fn get_age_data(age:String, year:String) -> String{
	let data = age.replace(".csv", "")
		.replace("ddw", "f")
		.replace("ddm", "m")
		.replace("jjm", "m")
		.replace("jjw", "f")
		.replace("mdm", "m")
		.replace("mdw", "f")
		.replace("mjm", "m")
		.replace("mjw", "f")
		.replace("msm", "m")
		.replace("msw", "f")
		.replace("ssm", "m")
		.replace("ssw", "f")
		.replace(&year[2..4], "&")
		.replace("&d", "i")
		.replace("&", "o")
		.replace(".html","").replace("1","").replace("2","").replace("3","").replace("4","").replace("5","").replace("6","");
	data
}
pub fn get_discipline_alias(discip:String) -> String{
	discip.to_lowercase().replace(" ","").replace(".","")
	.replace("milja","1mile")
	.replace("milje","miles")
	.replace("desetoboj","decathlon")
	.replace("sedmoboj","heptathlon")
	.replace("petoboj","pentathlon")
	.replace("vis",       "HJ" )
	.replace("dalj",       "LJ" )
	.replace("troskok",     "TJ" )
	.replace("motka",      "PV" )
	.replace("kugla",       "SP" )
	.replace("disk",    "DT" )
	.replace("koplje",   "JT" )
	.replace("kladivo",    "HT" )
	.replace("prepone",        "H" )
	.replace("steeplechase",  "SC")
	.replace("zapreke", "SC")
	.replace("zapr", "SC")
	.replace("polumaraton",   "HM" )
	.replace("maraton","Marathon") 
	.replace("hodanje",  "W")
	.replace("(cesta)","")
	.replace("cesta","")
	.replace("000W","kmW").replace("Relay","").replace("\r","")
	
}
pub fn cleanup_discipline(discip:String) -> String{
	discip.replace(".","")
	.replace("(cesta)","")
	.replace("zapr", "zapreke")
}
//radi nadimke za godine tako da je lakse izabrat kategoriju
pub fn get_age_alias(age:String, year:String) -> Vec<String>{
	let mut res:Vec<String> = Vec::new();
	res.push(age.replace(".html", "")
		.replace(".htm", "")
		.replace("ddw", "Kadetkinje")
		.replace("ddm", "Kadeti")
		.replace("jjm", "Juniori")
		.replace("jjw", "Juniorke")
		.replace("mdm", "Mlađi kadeti")
		.replace("mdw", "Mlađe kadetkinje")
		.replace("mjm", "Mlađi juniori")
		.replace("mjw", "Mlađe juniorke")
		.replace("msm", "Mlađi seniori")
		.replace("msw", "Mlađe seniorke")
		.replace("ssm", "Seniori")
		.replace("ssw", "Seniorke")
		.replace(&year[2..4], "&")
		.replace("&d", " dvorana")
		.replace("&", ""));
	res.push(age.replace(".html", "")
		.replace(".htm", "")
		.replace("ddw", "kw")
		.replace("ddm", "km")
		.replace("jjm", "jm")
		.replace("jjw", "jw")
		.replace("mdm", "mkm")
		.replace("mdw", "mkw")
		.replace("mjm", "mjm")
		.replace("mjw", "mjw")
		.replace("msm", "msm")
		.replace("msw", "msw")
		.replace("ssm", "sm")
		.replace("ssw", "sw")
		.replace(&year[2..4], "&")
		.replace("&d", "d")
		.replace("&", ""));
	res.push(age.replace(".html", "")
		.replace(".htm","")
		.replace(&year[2..4], "&")
		.replace("&d", "d")
		.replace("&", "")
		.replace("ddw", "U16w")
		.replace("ddm", "U16m")
		.replace("jjm", "U20m")
		.replace("jjw", "U20w")
		.replace("mdm", "U14m")
		.replace("mdw", "U14w")
		.replace("mjm", "U18m")
		.replace("mjw", "U18w")
		.replace("msm", "U23m")
		.replace("msw", "U23w")
		.replace("ssm", "M35")
		.replace("ssw", "W35"));
	res
}
pub fn fetchWebsite(url:&str) -> Result<String, reqwest::Error>{
	let body = reqwest::blocking::get(url)?
    .text_with_charset("windows-1250")?;

	Ok(body)
}
pub fn cache(){
	let website = fetchWebsite("https://www.has.hr/images/stories/HAS/tabsez/");
	let mut years = Vec::new();
	let _website = match website{
		Ok(year) => {
			let fragment = Html::parse_fragment(&year.to_string());
			let selector = Selector::parse("a").unwrap();
			for y in fragment.select(&selector) {
				let txt = y.text().collect::<Vec<_>>();
				if txt[0].contains("/"){
					years.push(txt[0].to_string().replace("/",""));
				}
			}
			fs::create_dir_all("/has/cache/".to_owned() + &years[years.len()-1].clone());
			let year_website = fetchWebsite(&("https://www.has.hr/images/stories/HAS/tabsez/".to_owned()+&years[years.len()-1].clone()));
			let _year_website = match year_website{
				Ok(age) => {
					let fragment = Html::parse_fragment(&age.to_string());
					let selector = Selector::parse("a").unwrap();
					for y in fragment.select(&selector) {
						let txt = y.text().collect::<Vec<_>>();
						if txt[0].contains(".htm"){
							let data_website = fetchWebsite(&("https://www.has.hr/images/stories/HAS/tabsez/".to_owned()+&years[years.len()-1].clone()+"/"+&txt[0]));
							let _data_website = match data_website{
								Ok(data) => {
									fs::write(&*("/has/cache/".to_owned() + &years[years.len()-1].clone() + "/" + &txt[0].clone().replace("html","htm").replace("htm","html")),data.clone()).expect("Unable to write file");

								},
								Err(o) => println!("Failed! using cache! / data")
							};
						}
					}
				},
				Err(a) => println!("Failed! using cache! / age")
			};
		},
		Err(e) => println!("Failed! using cache! / year")
	};
}
pub fn get_years() -> Vec<String> {
	let mut years = Vec::new();
    let paths = fs::read_dir("/has/cache/").unwrap();
    for path in paths {
        years.push(path.unwrap().path().to_string_lossy().replace("/has/cache/",""));
    }
	years
}
//funkcija za uzimanje kategorija omogucuje uzimanje podataka s hasove stranice
pub fn get_ages(year:String) -> Vec<String>{
	let mut ages = Vec::new();
    let paths = fs::read_dir(&("/has/cache/".to_owned()+&year.clone())).unwrap();
    for path in paths {
        ages.push(path.unwrap().path().to_string_lossy().replace(&("/has/cache/".to_owned()+&year.clone()+"\\"),""));
    }
	ages
}
pub fn get_categories(data: Vec<Vec<Vec<String>>>) -> Vec<String>{
	let mut res:Vec<String> = Vec::new();
	for x in data{
		if x.len() > 0{
			if x[0].len() > 1{
				res.push(x[0][2].clone());
			}
		}
	}
	res
}
pub fn get_category(data: Vec<Vec<Vec<String>>>,category: String) -> Vec<Vec<String>>{
	let mut res:Vec<Vec<String>> = Vec::new();
	for x in data{
		if x.len() > 0{
			if x[0][2].to_lowercase().replace(" ","") == category.to_lowercase().replace(" ",""){
				res = x;
				break;
			}
		}	
	}
	res
}
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::fs::File;
/*pub fn //append_error(text: String) -> std::io::Result<()> {
	let mut error = OpenOptions::new()
            .read(true)
            .write(true)
			.append(true)
            .create(true)
            .open("error.log")?;
	writeln!(error,"{}\n\n",text);
	Ok(())
}*/
/*
extracts the date from HTML file for whole age group

*/
pub fn get_data(path: &str) -> Vec<Vec<Vec<String>>>{
	let data: String = fs::read_to_string(path).unwrap().replace("\r\n","\n");
	let remove_find = Regex::new(r"</?font.*?>").unwrap();
	//append_error(path.to_string());
	let cleaned_data = remove_find.replace_all(&data,"");
		lazy_static! {
		//regexi za uzimanje podataka i sortiranje njih
		//vjetar
    	static ref WIND_DATA: Regex = Regex::new(r" ([+-]?\d,\d) ").unwrap();	
		static ref EXTRACT_DISCIPLINE: Regex = Regex::new(r"(?u)<b>([^\n]*) / [^\n]* - ([\p{L}\d (),.=-]*)(?: / [\p{L}\d ,.()=]*)?</b>\n([\p{L}(-; \n]*)").unwrap();
		static ref SINGLE_EXTRACT: Regex = Regex::new(r"(\d{0,2}[;:]?\d{0,2}:?\d+,\d{1,2}|\d*[;:]*\d*:\d+) +([+-]{0,1}\d,\d)? +([-\p{L} -]+?) +(\d{2}.\d{2}.\d{4}|\d{4})? ([\p{L}\d-]+?)? +([\p{L},/ .-]+?) +(\d{2}.\d{2}.\d{4})").unwrap();
		static ref RELAY_EXTRACT: Regex = Regex::new(r"(\d*:*\d+,\d{1,2}) +([\p{L} ,-]+?) {2,}([\p{L},/ ]+?) +(\d{2}.\d{2}.\d{4})\s+([\p{L} -]+?) \(\d{4}\), ([\p{L} -]+?) \(\d{4}\), ([\p{L} -]+?) \(\d{4}\), ([\p{L} -]+?) \(\d{4}\)").unwrap();
		static ref MULTI_EXTRACT: Regex = Regex::new(r"(\d{3,}) +([\d\p{L} \.-]+?) +(\d{2}.\d{2}.\d{4}|\d{4}) ([\p{L}-]{0,}) +([\p{L},/ ]+?) +((?:\d{2}[/-])?\d{2}.\d{2}.\d{4})\s+([\p{L}0-9/:+, -]+)").unwrap();
	}
	let mut data:Vec<Vec<Vec<String>>> = Vec::new();
	for caps in EXTRACT_DISCIPLINE.captures_iter(&cleaned_data) {	
		let mut res:Vec<Vec<String>> = Vec::new();
		let mut rank = 1;
		let v = &caps[3].split("\n").collect::<Vec<&str>>();
		let extra_info = true; //Služi da nam kad nije bilo informacija o vjetru rečemo korisniku da nije bilo informacija s NO_WIND
		let mut extra_mod = false; //Jer smo naisli na nema informacija o vjetru ili uz pomoć vjetra
		let mut extra_pos = 0; //Ak jesmo na kojoj je poziciji(Prepostavka je da nema informacija o vjetru je prije nego uz pomoć vjetra tako da možemo kasnije uzimat laške ove podatke
		let mut pos = 2;
		
		if WIND_DATA.is_match(&caps[3]){
			
			for x in v{
				let mut temp:Vec<String> = Vec::new();		
				if SINGLE_EXTRACT.is_match(x){
					pos = pos + 1;
					if !extra_mod{
						temp.push(rank.to_string());
						rank = rank +1;
					}else{
						temp.push("INVD".to_string());
					}
					for y in SINGLE_EXTRACT.captures_iter(x) {
						let mut count = 0;
						for z in y.iter(){
							match z {
								Some(i) => {
									if count != 0{
										temp.push(i.as_str().to_string());
									}else if count == 1{
										if !(caps[2].to_lowercase().contains("kladivo") || caps[2].to_lowercase().contains("disk") || caps[2].to_lowercase().contains("koplje") || caps[2].to_lowercase().contains("vortex")){
											temp.push(format_result(i.as_str().to_string()));
										}
									}
								},
								None => {
									temp.push("NO_INFO".to_string())
								}
							}
							count = count + 1;
						}
					}
					res.push(temp.clone());
					temp.clear();
				}else{
					if x.contains("/"){
						if extra_mod == true{
							pos = pos + 1;
						}
						temp.push(x.to_string());
						res.push(temp.clone());
						temp.clear();
						extra_mod = true;
						extra_pos = pos;
					}else{
						if x != &"" && (!x.contains("(") || !x.contains(")")){
							//append_error(x.to_string());
						}
					}
				}
			}
			if res.len() > 0 {
				if res[0].len() > 1{
					res.insert(0,vec!(res[0][7][res[0][7].len()-4..res[0][7].len()].to_string(),path.to_string().replace(&("/has/cache/".to_owned()+&res[0][7][res[0][7].len()-4..res[0][7].len()]+"/"),""),(&caps[2]).to_string(),"wind".to_string(),extra_pos.to_string()));
				}
			}
		}else if !(MULTI_EXTRACT.is_match(&caps[3]) || RELAY_EXTRACT.is_match(&caps[3])){
			for x in v{
				let mut temp:Vec<String> = Vec::new();		
				if SINGLE_EXTRACT.is_match(x){
					pos = pos + 1;
					if !extra_mod{
						temp.push(rank.to_string());
						rank = rank +1;
					}else{
						temp.push("INVD".to_string());
					}
					for y in SINGLE_EXTRACT.captures_iter(x) {
						let mut first = false;
						for z in y.iter(){
							if first{
								match z {
									Some(i) => {
										temp.push(i.as_str().to_string());
									},
									None => (temp.push("NO_INFO".to_string()))
								}
							}else{
								first = true;
							}
						}
					}
					res.push(temp.clone());
					temp.clear();
				}else{
					if x.contains("/"){
						if extra_mod == true{
							pos = pos + 1;
						}
						temp.push(x.to_string());
						res.push(temp.clone());
						temp.clear();
						extra_mod = true;
						extra_pos = pos;
					}else{
						if x != &"" && (!x.contains("(") || !x.contains(")")){
							//append_error(x.to_string());
						}
					}
				}
			}
			if res.len() > 0{
				if res[0].len() > 1{
					res.insert(0,vec!(res[0][7][res[0][7].len()-4..res[0][7].len()].to_string(),path.to_string().replace(&("/has/cache/".to_owned()+&res[0][7][res[0][7].len()-4..res[0][7].len()]+"/"),""),(&caps[2]).to_string(),"single".to_string(),extra_pos.to_string()));
				}
			}
		}else if MULTI_EXTRACT.is_match(&caps[3]) || RELAY_EXTRACT.is_match(&caps[3]){
			//println!("ooooooooooooooooooooooooooooooooooooo");
			let multi_line = Regex::new(r"(?um)(?:\d*:*\d+,\d{1,2}|\d+).+\n.+").unwrap();
			for t in multi_line.captures_iter(&caps[3]) {
				let mut temp:Vec<String> = Vec::new();		
				for x in t.iter(){
					let x = x.unwrap();
					//println!("{}", x.as_str());
					//println!("_____________________________________");
					if MULTI_EXTRACT.is_match(x.as_str()){
						for y in MULTI_EXTRACT.captures_iter(x.as_str()) {
							temp.push(rank.to_string());
							rank = rank +1;
							let mut first = false;
							for z in y.iter(){
								if first{
									match z {
										Some(i) => {
											temp.push(i.as_str().to_string());
										},
										None => (temp.push("NO_INFO".to_string()))
									}
								}else{
									first = true;
								}
							}
						}
						res.push(temp.clone());
						temp.clear();
					}else if RELAY_EXTRACT.is_match(x.as_str()){
						for y in RELAY_EXTRACT.captures_iter(x.as_str()) {
							temp.push(rank.to_string());
							rank = rank +1;
							let mut first = false;
							for z in y.iter(){
								if first{
									match z {
										Some(i) => {
											temp.push(i.as_str().to_string());
										},
										None => (temp.push("NO_INFO".to_string()))
									}
								}else{
									first = true;
								}
							}
							res.push(temp.clone());
							temp.clear();
						}
					}else{
						if x.as_str() != "" && (!x.as_str().contains("(") || !x.as_str().contains(")")){
							//append_error(x.as_str().to_string());
						}
					}			
				}
			}
			if RELAY_EXTRACT.is_match(&caps[3]){
				if !&caps[1].contains("mješovita"){
					res.insert(0,vec!(res[0][4][res[0][4].len()-4..res[0][4].len()].to_string(),path.to_string().replace(&("/has/cache/".to_owned()+&res[0][4][res[0][4].len()-4..res[0][4].len()]+"/"),""),(&caps[2]).to_string(),"relay".to_string(),"".to_string()));
				}else{
					res.insert(0,vec!(res[0][4][res[0][4].len()-4..res[0][4].len()].to_string(),path.to_string().replace(&("/has/cache/".to_owned()+&res[0][4][res[0][4].len()-4..res[0][4].len()]+"/"),""),"X".to_string()+&(&caps[2]).to_string(),"relay".to_string(),"".to_string()));
				}
			}else if MULTI_EXTRACT.is_match(&caps[3]){
				res.insert(0,vec!(res[0][6][res[0][6].len()-4..res[0][6].len()].to_string(),path.to_string().replace(&("/has/cache/".to_owned()+&res[0][6][res[0][6].len()-4..res[0][6].len()]+"/"),""),(&caps[2]).to_string(),"multi".to_string(),extra_pos.to_string()));
			}
		}else{
			//append_error(caps[3].to_string());
		}
		data.push(res.clone());
		res.clear();
	}
	data
}
/*
gets results for single year for athlete
*/
pub fn get_profile(data:Vec<Vec<Vec<String>>>, search: String) -> Vec<Vec<String>>{
	let mut res:Vec<Vec<String>> = Vec::new();
	let mut extra_mod = false;
	let mut bday = "".to_string();
	for discip in data.clone(){
		let mut extra_mod = false;
		for mut x in 1..discip.len(){
			if discip[x].len() > 1 && discip[0].len() > 1{
				let discipline_text = cleanup_discipline(discip[0][2].clone());
				if discip[0][3] == "wind" || discip[0][3] == "single"{
					if discip[x][3].replace(" ","").to_lowercase() == search.replace(" ","").to_lowercase(){
						bday = discip[x][4][discip[x][4].len()-4..discip[x][4].len()].to_string().clone();
						if !extra_mod{
							if data[0][0][1].clone().contains("d.html"){
								res.push(vec!(discipline_text.clone()+" D",discip[x][1].clone(),discip[x][2].clone(),discip[x][0].clone(),calculate_points(get_result(discip[x][1].clone()),discip[0][2].clone(),data[0][0][1].clone(),data[0][0][0].clone())));
							}else{						
								res.push(vec!(discipline_text.clone(),discip[x][1].clone(),discip[x][2].clone(),discip[x][0].clone(),calculate_points(get_result(discip[x][1].clone()),discip[0][2].clone(),data[0][0][1].clone(),data[0][0][0].clone())));
							}
							if discip[0][4].parse::<usize>().unwrap() != 0{
								x = discip[0][4].parse::<usize>().unwrap();
								extra_mod = true;
							}else{
								break;
							}
						}else{
							res.push(vec!(discipline_text.clone()+" IR",discip[x][1].clone(),discip[x][2].clone(),discip[x][0].clone(),"0".to_string()));
							break;
							
						}
					}
				}else if discip[0][3] == "relay"{
					if discip[x][5].replace(" ","").to_lowercase() == search.replace(" ","").to_lowercase() || discip[x][6].replace(" ","").to_lowercase() == search.replace(" ","").to_lowercase() || discip[x][7].replace(" ","").to_lowercase() == search.replace(" ","").to_lowercase() || discip[x][8].replace(" ","").to_lowercase() == search.replace(" ","").to_lowercase(){
						if data[0][0].len() > 1{
							if data[0][0][1].clone().contains("d.html"){
								res.push(vec!(discipline_text.clone()+" D",discip[x][1].clone(),"NO_INFO".to_string(),discip[x][0].clone(),calculate_points(get_result(discip[x][1].clone()),discip[0][2].clone(),data[0][0][1].clone(),data[0][0][0].clone())));
							}else{
								res.push(vec!(discipline_text.clone(),discip[x][1].clone(),"NO_INFO".to_string(),discip[x][0].clone(),calculate_points(get_result(discip[x][1].clone()),discip[0][2].clone(),data[0][0][1].clone(),data[0][0][0].clone())));
							}
						}
					}
				}else if discip[0][3] == "multi"{
					if discip[x][2].replace(" ","").to_lowercase() == search.replace(" ","").to_lowercase(){
						if data[0][0][1].clone().contains("d.html"){
							res.push(vec!(discipline_text.clone()+" D",discip[x][1].clone(),"NO_INFO".to_string(),discip[x][0].clone(),calculate_points(get_result(discip[x][1].clone()),discip[0][2].clone(),data[0][0][1].clone(),data[0][0][0].clone())));
						}else{
							res.push(vec!(discipline_text.clone(),discip[x][1].clone(),"NO_INFO".to_string(),discip[x][0].clone(),calculate_points(get_result(discip[x][1].clone()),discip[0][2].clone(),data[0][0][1].clone(),data[0][0][0].clone())));
						}
					}
				}
			}
		}
	}
	if res.len() >= 1 {
		if res[0].len() > 1{
			res.insert(0,vec!("Discipline".to_string(),"Mark".to_string(),"Wind".to_string(),"Rank".to_string(),"WA points".to_string()));
			res.insert(0,vec!(data[0][0][0].clone() ,data[0][0][1].clone(),search,bday));
			return res;
		}else{
			vec!(vec!("empty".to_string()))	
		}
	}else{
		vec!(vec!("empty".to_string()))
	}
}
/*
gets whole carrer of a athlete
*/
fn get_carrer(search: String) -> Vec<Vec<String>>{
	let years = get_years();
	let barYear = ProgressBar::new(years.len() as u64);
	/*
	TODO optimzation to not process other gender and only revelant ages
	*/
	let _gender = "".to_string();
	let _bday = 0;
	let mut res:Vec<Vec<Vec<String>>>= Vec::new();
	let mut discip_global:Vec<String> = Vec::new();
	for x in 0..years.len(){
		barYear.inc(1);		
		let mut data:Vec<Vec<String>>;
		let mut temp:Vec<Vec<String>> = Vec::new();
		let mut discip:Vec<String> = Vec::new();
		for age in get_ages(years[x].clone()){
			data = get_profile(get_data(&("/has/cache/".to_owned() + &years[x].clone() + "/" + &age.clone())), search.clone());
			if data.len() != 1{
				if data[0][0] != "empty"{
					for d in 2..data.len(){
						let mut found = false;
						for dis in discip.clone(){
							if dis == data[d][0]{
								found = true;
								break;
							}
							
						}
						if !found{
							discip.push(data[d][0].clone());
							temp.push(vec!(data[d][0].clone(),data[d][1].clone(),data[d][4].clone()));
						}else{
							for x in 0..temp.len(){
								if temp[x][0] == data[d][0]{
									if !(data[d][0].to_lowercase().contains("kladivo") || data[d][0].to_lowercase().contains("disk") || data[d][0].to_lowercase().contains("koplje") || data[d][0].to_lowercase().contains("vortex") || data[d][0].to_lowercase().contains("boj") || data[d][0].to_lowercase().contains("athlon")){
										if get_result(temp[x][1].clone()) > get_result(data[d][1].clone()){
											temp.remove(x);
											temp.insert(0,vec!(data[d][0].clone(),data[d][1].clone(),data[d][4].clone()));
										}
									}else{
										if get_result(temp[x][1].clone()) <	 get_result(data[d][1].clone()){
											temp.remove(x);
											temp.insert(0,vec!(data[d][0].clone(),data[d][1].clone(),data[d][4].clone()));
										}
									}
								}									
							}
						}
					}
				}
			}
		}	
		if temp.len() > 0{
			for d in discip {
				let mut found = false;
				for t in &discip_global{
					if t == &d{
						found = true;
					}
				}
				if !found{
					discip_global.push(d.clone());
				}
			} 
			temp.insert(0,vec!(years[x].clone()));
			res.push(temp);
		}
	}			
	barYear.finish();
	let mut carrer:Vec<Vec<Vec<String>>> = Vec::new();
	for x in res.clone(){
		let mut temp:Vec<Vec<String>> = Vec::new();
		for d in discip_global.clone(){
			let mut found = false;
			let mut result = "".to_string();
			let mut points = "".to_string();
			for y in x.clone(){
				if d == y[0]{
					found = true;
					result = y[1].clone();
					points = y[2].clone();
					break;
				}
			}
			if found{
				temp.push(vec!(d.clone(),result,points));
			}else{
				temp.push(vec!(d.clone(),"x".to_string(),"0".to_lowercase()));
			}
		}
		temp.insert(0,vec!(x[0][0].clone()));
		carrer.push(temp.clone());
	}
	let mut results:Vec<f64> = Vec::new();
	let mut res:Vec<Vec<String>> = Vec::new();
	for d in discip_global{
		let mut temp:Vec<String> = Vec::new();
		temp.push(d.clone());
		let mut pb = 0.0;
		let mut points = "".to_string();
		for x in carrer.clone(){
			for y in x{
				if y[0] == d{
					temp.push(y[1].clone());
					if y[1] != "x".to_string(){
						let result = get_result(y[1].clone());
						if y[0].clone().contains("m") || y[0].clone().contains("x") || y[0].clone().contains("0") || y[0].clone().contains("Ma"){
							if pb > result{
								pb = result;
								points = y[2].clone();
							}else if pb == 0.0{
								pb = result;
								points = y[2].clone();
							}
						}else{
							if pb < result{
								pb = result;
								points = y[2].clone();
							}					
						}
					}
					break;
					
				}
			}
		}
		results.push(pb);
		if !(d.to_lowercase().contains("kladivo") || d.to_lowercase().contains("disk") || d.to_lowercase().contains("koplje") || d.to_lowercase().contains("vortex") || d.to_lowercase().contains("boj")){
			temp.push(calculate_result(pb));
		}else{
			temp.push(pb.to_string().replace(".",","));
		}
		temp.push(points);
		res.push(temp);
	}
	let mut removed = 0; 
	let mut technical:Vec<Vec<String>> = Vec::new();
	for x in 0..res.len(){
		if (!res[x-removed][0].contains("m") && !res[x-removed][0].contains("x")&& !res[x-removed][0].contains("0")&& !res[x-removed][0].contains("Ma") || res[x-removed][0].contains("g")) || res[x-removed][0].contains("boj"){
			technical.push(res[x-removed].clone());
			res.remove(x-removed);
			removed = removed + 1;
		}
	}
	let mut min = 0;
	if res.len() > 1{
		for x in 0..res.len()-1{
			min = x;
			for y in (x+1)..res.len(){
				if get_result(res[y][res[y].len()-2].clone()) < get_result(res[min][res[min].len()-2].clone()) {
					min = y;
				}
			}	
			let temp = res[min].clone();
			res[min] = res[x].clone();
			res[x] = temp;		
		}
	}
	for x in 0..technical.len(){
		min = x;
		for y in (x+1)..technical.len(){
			if get_result(technical[y][technical[y].len()-2].clone()) < get_result(technical[min][technical[min].len()-2].clone()) {
				min = y;
			}
		}	
		let temp = technical[min].clone();
		technical[min] = technical[x].clone();
		technical[x] = temp;
	}
	for x in 0..technical.len(){
		res.push(technical[x].clone());
	}
	let mut temp:Vec<String> = Vec::new(); 
	temp.push(search.clone());
	for x in carrer{
		temp.push(x[0][0].clone());
	}
	temp.push("PB".to_string());
	temp.push("WA points".to_string());
	res.insert(0,temp.clone());
	res
}
/*
Function for getting clubs statistics
gets every result and calculates the points for it and sorts it from highest to lowest
*/
fn get_stats(year: String) -> Vec<Vec<String>>{
	let mut stats: Vec<Vec<String>> = Vec::new();
	let mut relay: Vec<Vec<String>> = Vec::new();
	let mut relayCount: i32 = 0;
	let mut bests: Vec<i32> = Vec::new();
	let barYear = ProgressBar::new(get_ages(year.clone()).len() as u64);
	for age in get_ages(year.clone()){
		let data = get_data(&("/has/cache/".to_owned() + &year.clone() + "/" + &age.clone()));
		let mut sufix = "";
		if age.clone().contains("d.html"){
			sufix = " D";
		}
		for discip in data.clone(){
			for x in 1..discip.len(){
				if discip[x].len() > 1 && discip[0].len() > 1 {
					if discip[0][3] == "wind"{
						if x == discip[0][4].parse::<usize>().unwrap(){
							break;
						}
					}
					if discip[x][5] == "AGR" || discip[x][2] == "AGRAM, ZAGREB" || discip[x][4] == "AGR"{
						let points = calculate_points(get_result(discip[x][1].clone()),discip[0][2].clone(),data[0][0][1].clone(),data[0][0][0].clone()).parse::<i32>().unwrap();
						if points != 0{
							if discip[0][3] == "wind" || discip[0][3] == "single"{
								if stats.len() != 0{
									let mut find = false;
									let mut pos = 0;
									for stat in 0..stats.len(){
										if stats[stat][1] == discip[x][3]{
											if stats[stat][0].clone() == discip[0][2].clone()+sufix.clone(){
												find = true;
												pos = stat;
												break;
											}
										}
									}
									if find == true{
										if points > bests[pos]{
											stats.remove(pos);
											bests.remove(pos);
											stats.push(vec!(discip[0][2].clone()+sufix.clone(),discip[x][3].clone(),discip[x][1].clone(),discip[x][7].clone(),discip[x][6].clone()));
											bests.push(points);
										}
									}else{
										stats.push(vec!(discip[0][2].clone()+sufix.clone(),discip[x][3].clone(),discip[x][1].clone(),discip[x][7].clone(),discip[x][6].clone()));
										bests.push(points);
									}
								}else{
									stats.push(vec!(discip[0][2].clone()+sufix.clone(),discip[x][3].clone(),discip[x][1].clone(),discip[x][7].clone(),discip[x][6].clone()));
									bests.push(points);
								}									
							}else if discip[0][3] == "relay"{
								if relay.len() != 0{
									let mut pos = 0;
									let mut team = 0;
									let mut teamCount = 0;
									for stat in 0..stats.len(){
										team = 0;
										if stats[stat][1].contains("%"){
											let i = stats[stat][1].replace("%","").parse::<usize>().unwrap();
											for e in 0..relay[i].len(){
												let mut found = false;
												for j in 5..8{
													if discip[x][j] == relay[i][e]{
														found = true;
														team = team+1;
														break;
													}
												}			
												if found == false{
													break;
												}
											}
											if team == 3{
												pos = stat;
												teamCount = i;
												break;
											}
										}
									}									
									if team == 3{
										if points > bests[pos]{
											stats.remove(pos);
											bests.remove(pos);
											stats.insert(pos, vec!(discip[0][2].clone()+sufix.clone(),teamCount.to_string()+"%",discip[x][1].clone(),discip[x][4].clone(),discip[x][3].clone()));
											bests.insert(pos,points);
										}
									}else{
										relay.push(vec!(discip[x][5].clone(),discip[x][6].clone(),discip[x][7].clone(),discip[x][8].clone()));
										stats.push(vec!(discip[0][2].clone()+sufix.clone(),relayCount.clone().to_string()+"%",discip[x][1].clone(),discip[x][4].clone(),discip[x][3].clone()));
										bests.push(points);
										relayCount = relayCount+1;
									}
								}else{
									relay.push(vec!(discip[x][5].clone(),discip[x][6].clone(),discip[x][7].clone(),discip[x][8].clone()));
									stats.push(vec!(discip[0][2].clone()+sufix.clone(),relayCount.clone().to_string()+"%",discip[x][1].clone(),discip[x][4].clone(),discip[x][3].clone()));
									bests.push(points);
									relayCount = relayCount+1;
								}
							}else if discip[0][3] == "multi"{
								if stats.len() != 0{
									let mut find = false;
									let mut pos = 0;
									for stat in 0..stats.len(){
										if stats[stat][1] == discip[x][3]{
											if stats[stat][0].clone() == discip[0][2].clone()+sufix.clone(){
												find = true;
												pos = stat;
												break;
											}
										}
									}
									if find == true{
										if points > bests[pos]{
											stats.remove(pos);
											bests.remove(pos);
											stats.push(vec!(discip[0][2].clone()+sufix.clone(),discip[x][2].clone(),discip[x][1].clone(),discip[x][6].clone(),discip[x][5].clone()));
											bests.push(points);
										}
									}else{
										stats.push(vec!(discip[0][2].clone()+sufix.clone(),discip[x][2].clone(),discip[x][1].clone(),discip[x][6].clone(),discip[x][5].clone()));
										bests.push(points);
									}
								}else{
									stats.push(vec!(discip[0][2].clone()+sufix.clone(),discip[x][2].clone(),discip[x][1].clone(),discip[x][6].clone(),discip[x][5].clone()));
									bests.push(points);
								}	
							}
						}
					}
				}
			}
		}
		barYear.inc(1);		
	}
	barYear.finish();
	let mut min = 0;
	for x in 0..stats.len()-1{
		min = x;
		for y in (x+1)..stats.len(){
			if bests[y] > bests[min] {
				min = y;
			}
		}	
		let temp = stats[min].clone();
		stats[min] = stats[x].clone();
		stats[x] = temp;
		let temp = bests[min].clone();
		bests[min] = bests[x].clone();
		bests[x] = temp;			
	}
	let mut last = 0;
	for x in relay.clone(){	
		println!("{:?}",x);
	}
	for x in 0..stats.len(){
		if stats[x][1].contains("%"){
			println!("{:?}",stats[x].clone());
			let i = stats[x][1].replace("%","").parse::<usize>().unwrap();
			let mut name = "".to_string();
			for e in 0..relay[i].len()-1{
				name = name + &relay[i][e].clone() + ", ";
			}
			name = name + &relay[i][3].clone();
			stats[x].remove(1);
			stats[x].insert(1, name);
		}
		stats[x].push(bests[x].clone().to_string());
		if bests[x] != last{
			stats[x].insert(0,(x+1).to_string());
		}else{
			stats[x].insert(0,"".to_string());
		}
		last = bests[x];
	}
	stats.insert(0,vec!("Rank".to_string(),"Discipline".to_string(),"Name".to_string(),"Mark".to_string(),"Date".to_string(),"City".to_string(),"WA Points".to_string()));
	stats
}
fn get_club_record() -> Vec<Vec<Vec<String>>>{
	let years = get_years();
	let barYear = ProgressBar::new(years.len() as u64);
	let gender = "".to_string();
	let bday = 0;
	let mut res:Vec<Vec<Vec<String>>>= Vec::new();
	let mut discip:Vec<Vec<String>> = Vec::new();
	let mut best:Vec<f64> = Vec::new();
	
	for x in 5..years.len(){
		let mut data:Vec<Vec<Vec<String>>>;
		for age in get_ages(years[x].clone()){
			data = get_data(&("/has/cache/".to_owned() + &years[x].clone() + "/" + &age.clone()));
			println!("{} {}",get_age_alias(age.clone(),years[x].clone())[2],years[x].clone());
			for discipline in data{
				if discipline.len() != 0{
					for person in 1..discipline.len(){
						if discipline[person].len() > 1{
							if discipline[0].len() > 1{
								if discipline[0][3] == "single" || discipline[0][3] == "wind"{
									if discipline[person][5] == "AGR"{
										if discip.len() != 0{
											let mut found = false;
											for d in 0..discip.len(){
												if discip[d][0] == discipline[0][2] && get_age_alias(age.clone(),years[x].clone())[2] == discip[d][1]{
													if discip[d][0].clone().contains("m") || discip[d][0].clone().contains("x") || discip[d][0].clone().contains("0") || discip[d][0].clone().contains("Ma"){
														if get_result(discipline[person][1].clone()) < best[d]{
															discip.remove(d);
															discip.insert(d,vec!(discipline[0][2].clone(),get_age_alias(age.clone(),years[x].clone())[2].clone(),discipline[person][3].clone(),discipline[person][1].clone(),discipline[person][6].clone(),discipline[person][7].clone()));
															best.remove(d);
															best.insert(d,get_result(discipline[person][1].clone()));
														}
													}else{
														if get_result(discipline[person][1].clone()) > best[d]{
															discip.remove(d);
															discip.insert(d,vec!(discipline[0][2].clone(),get_age_alias(age.clone(),years[x].clone())[2].clone(),discipline[person][3].clone(),discipline[person][1].clone(),discipline[person][6].clone(),discipline[person][7].clone()));
															best.remove(d);
															best.insert(d,get_result(discipline[person][1].clone()));
														}
													}
													found = true;
													break;
												}
											}
											if found == false{
												discip.push(vec!(discipline[0][2].clone(),get_age_alias(age.clone(),years[x].clone())[2].clone(),discipline[person][3].clone(),discipline[person][1].clone(),discipline[person][6].clone(),discipline[person][7].clone()));
												best.push(get_result(discipline[person][1].clone()));
											}
										}else{
											discip.push(vec!(discipline[0][2].clone(),get_age_alias(age.clone(),years[x].clone())[2].clone(),discipline[person][3].clone(),discipline[person][1].clone(),discipline[person][6].clone(),discipline[person][7].clone()));
											best.push(get_result(discipline[person][1].clone()));
										}
										break;
									}
								}else if discipline[0][3] == "multi"{
									if discipline[person][4] == "AGR"{
										if discip.len() != 0{
											let mut found = false;
											for d in 0..discip.len(){
												if discip[d][0] == discipline[0][2] && get_age_alias(age.clone(),years[x].clone())[2] == discip[d][1]{
													if discip[d][0].clone().contains("m") || discip[d][0].clone().contains("x") || discip[d][0].clone().contains("0") || discip[d][0].clone().contains("Ma"){
														if get_result(discipline[person][1].clone()) < best[d]{
															discip.remove(d);
															discip.insert(d,vec!(discipline[0][2].clone(),get_age_alias(age.clone(),years[x].clone())[2].clone(),discipline[person][2].clone(),discipline[person][1].clone(),discipline[person][5].clone(),discipline[person][6].clone()));
															best.remove(d);
															best.insert(d,get_result(discipline[person][1].clone()));
														}
													}else{
														if get_result(discipline[person][1].clone()) > best[d]{
															discip.remove(d);
															discip.insert(d,vec!(discipline[0][2].clone(),get_age_alias(age.clone(),years[x].clone())[2].clone(),discipline[person][2].clone(),discipline[person][1].clone(),discipline[person][5].clone(),discipline[person][6].clone()));
															best.remove(d);
															best.insert(d,get_result(discipline[person][1].clone()));
														}
													}
													found = true;
													break;
												}
											}
											if found == false{
												discip.push(vec!(discipline[0][2].clone(),get_age_alias(age.clone(),years[x].clone())[2].clone(),discipline[person][2].clone(),discipline[person][1].clone(),discipline[person][5].clone(),discipline[person][6].clone()));
												best.push(get_result(discipline[person][1].clone()));
											}
										}else{
											discip.push(vec!(discipline[0][2].clone(),get_age_alias(age.clone(),years[x].clone())[2].clone(),discipline[person][2].clone(),discipline[person][1].clone(),discipline[person][5].clone(),discipline[person][6].clone()));
											best.push(get_result(discipline[person][1].clone()));
										}
										break;
									}									
								}else if discipline[0][3] == "relay"{
									if discipline[person][2] == "AGRAM, ZAGREB"{
										if discip.len() != 0{
											let mut found = false;
											for d in 0..discip.len(){
												if discip[d][0] == discipline[0][2] && get_age_alias(age.clone(),years[x].clone())[2] == discip[d][1]{
													if discip[d][0].clone().contains("m") || discip[d][0].clone().contains("x") || discip[d][0].clone().contains("0") || discip[d][0].clone().contains("Ma"){
														if get_result(discipline[person][1].clone()) < best[d]{
															discip.remove(d);
															discip.insert(d,vec!(discipline[0][2].clone(),get_age_alias(age.clone(),years[x].clone())[2].clone(),	discipline[person][5].clone() + &", ".to_owned() +&discipline[person][6].clone() + &", ".to_owned()+&discipline[person][7].clone() + &", ".to_owned()+&discipline[person][8].clone(),discipline[person][1].clone(),discipline[person][3].clone(),discipline[person][4].clone()));
															best.remove(d);
															best.insert(d,get_result(discipline[person][1].clone()));
														}
													}else{
														if get_result(discipline[person][1].clone()) > best[d]{
															discip.remove(d);
															discip.insert(d,vec!(discipline[0][2].clone(),get_age_alias(age.clone(),years[x].clone())[2].clone(),	discipline[person][5].clone() + &", ".to_owned() +&discipline[person][6].clone() + &", ".to_owned()+&discipline[person][7].clone() + &", ".to_owned()+&discipline[person][8].clone(),discipline[person][1].clone(),discipline[person][3].clone(),discipline[person][4].clone()));
															best.remove(d);
															best.insert(d,get_result(discipline[person][1].clone()));
														}
													}
													found = true;
													break;
												}
											}
											if found == false{
												discip.push(vec!(discipline[0][2].clone(),get_age_alias(age.clone(),years[x].clone())[2].clone(),	discipline[person][5].clone() + &", ".to_owned() +&discipline[person][6].clone() + &", ".to_owned()+&discipline[person][7].clone() + &", ".to_owned()+&discipline[person][8].clone(),discipline[person][1].clone(),discipline[person][3].clone(),discipline[person][4].clone()));
												best.push(get_result(discipline[person][1].clone()));
											}
										}else{
											discip.push(vec!(discipline[0][2].clone(),get_age_alias(age.clone(),years[x].clone())[2].clone(),	discipline[person][5].clone() + &", ".to_owned() +&discipline[person][6].clone() + &", ".to_owned()+&discipline[person][7].clone() + &", ".to_owned()+&discipline[person][8].clone(),discipline[person][1].clone(),discipline[person][3].clone(),discipline[person][4].clone()));
											best.push(get_result(discipline[person][1].clone()));
										}
										break;
									}									
								}
							}
						}
					}
				}
			}
		}
	}
	let mut min = 0;
	let mut age = "".to_string();
	let mut finished = false;
	let mut sep:Vec<Vec<Vec<String>>> = Vec::new();
	let mut temp:Vec<Vec<String>> = Vec::new();
	while discip.len() != 0{
		let age = discip[0][1].clone();
		let mut temp:Vec<Vec<String>> = Vec::new();
		let mut removed = 0;
		for y in 0..discip.len(){
			if age == discip[y-removed][1]{
				temp.push(discip[y-removed].clone());
				discip.remove(y-removed);
				removed = removed + 1;
			}
		}
		sep.push(temp);
	}
	let mut export:Vec<Vec<Vec<String>>>= Vec::new();
	for mut res in sep{
		let mut removed = 0; 
		let mut technical:Vec<Vec<String>> = Vec::new();
		for x in 0..res.len(){
			if (!res[x-removed][0].contains("m") && !res[x-removed][0].contains("x")&& !res[x-removed][0].contains("0")&& !res[x-removed][0].contains("Ma") || res[x-removed][0].contains("g")) || res[x-removed][0].contains("boj"){
				technical.push(res[x-removed].clone());
				res.remove(x-removed);
				removed = removed + 1;
			}
		}
		let mut min = 0;
		if res.len() > 1{
			for x in 0..res.len()-1{
				min = x;
				for y in (x+1)..res.len(){
					if get_result(res[y][3].clone()) < get_result(res[min][3].clone()) {
						min = y;
					}
				}	
				let temp = res[min].clone();
				res[min] = res[x].clone();
				res[x] = temp;		
			}
		}
		for x in 0..technical.len(){
			min = x;
			for y in (x+1)..technical.len(){
				if get_result(technical[y][3].clone()) < get_result(technical[min][3].clone()) {
					min = y;
				}
			}	
			let temp = technical[min].clone();
			technical[min] = technical[x].clone();
			technical[x] = temp;
		}
		for x in 0..technical.len(){
			res.push(technical[x].clone());
		}
		let mut row:Vec<Vec<String>> = Vec::new();
		for d in res{
			row.push(d.clone());
		}
		
		export.push(row.clone());
		row.clear();
	}
	export
	/*for d in 0..res.len(){
		for x in 0..res[d].len()-1{
			min = x;
			for y in (x+1)..res[d].len(){
				if get_result(res[d][y][3].clone()) > get_result(res[d][min][3].clone()) {
					min = y;
				}
			}	
			let temp = res[d][min].clone();
			res[d][min] = res[d][x].clone();
			res[d][x] = temp;		
		}
	}*/
}

fn main(){
	let data = vec!(get_stats("2021".to_string()));
	display(data.clone());
	save_stats(data.clone(), "2021".to_string());
	/*let years = get_years();
	for x in 0..years.len(){
		for age in get_ages(years[x].clone()){
			for disp in get_categories(get_data(&("/has/cache/".to_owned() + &years[x].clone() + "/" + &age.clone()))){
				if disp.contains("0") == true && disp.contains("m") == false{	
					println!("{} {}", age, disp);
				}
			}
		}
	}*/
	/*let args: Vec<String> = env::args().collect();
	cache();
	if args.len() == 1{
		while true{
			//save_csv(vec!(get_profile(get_data("/has/cache/2021/ssm21.html"), "Fran bonifačić".to_string())));
			let mut alias:Vec<Vec<String>> = vec![];
			let year = CLI_question("Choose a year",get_years(),true);
			if year != "all".to_string(){
				for x in get_ages(year.clone()){
					alias.push(get_age_alias(x, year.clone()));
				}
				let age = CLI_question_alias("Choose a age",get_ages(year.clone()), alias);
				if age != "all".to_string(){
					let data = get_data(&*("/has/cache/".to_owned() + &year.clone() + "/" + &age.clone()));
					let discipline = CLI_question("Choose a discipline",get_categories(data.clone()),true);
					if discipline != "all".to_string(){
						display(vec!(get_category(data,discipline)));
					}else{
						let key = CLI_input("Who do you want to search for");
						let data = get_profile(data,key);
						display(vec!(data.clone()));
						save_csv(vec!(data));
					}
				}else{
					let key = CLI_input("Who do you want to search for");
					let mut data:Vec<Vec<Vec<String>>> = Vec::new();
					data.push(vec!(vec!(year.clone() + &" profiles".to_string())));
					let barYear = ProgressBar::new(get_ages(year.clone()).len() as u64);
					for x in get_ages(year.clone()){
						data.push(get_profile(get_data(&*("/has/cache/".to_owned() + &year.clone() + "/" + &x.clone())),key.clone()));		
						barYear.inc(1);		
					}
					barYear.finish();	
					display(data.clone());
					save_csv(data);
				}
			}else{
				let key = CLI_input("Who do you want to search for");
				let data = get_carrer(key);
				display(vec!(data.clone()));
				save_csv(vec!(data));
			}
		}
	}else if args[1] == "s"{
		while true{
			let key = CLI_input("What year do you want statistics for?");
			let data = get_stats(key.clone());
			display(vec!(data.clone()));
			save_stats(vec!(data),key);	
		}		
	}else if args[1] == "c"{
		while true{
			let discip = CLI_input("What discipline do you want to calculate?");
			let result = CLI_input("What result do you want to calculate?");
			println!("Number of points they would get is {}\n", calculate_points_manual(result,discip));
		}		
	}else if args[1] == "r"{
		let data = get_club_record();
		display(data.clone());
		save_stats(data.clone(), "records".to_string());
	}*/
}