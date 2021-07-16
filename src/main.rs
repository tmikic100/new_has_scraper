#[macro_use] extern crate lazy_static;
extern crate regex;

use regex::Regex;
use std::{fs, io};
use scraper::{Selector, Html};
use term_table::table_cell::TableCell;
use term_table::row::Row;
use indicatif::ProgressBar;

pub fn get_years() -> Vec<String> {
	let res = reqwest::blocking::get("https://www.has.hr/images/stories/HAS/tabsez").unwrap();
	let year = res.text_with_charset("windows-1250").unwrap();
	let fragment = Html::parse_fragment(&year.to_string());
	let selector = Selector::parse("a").unwrap();
	let mut years = Vec::new();
	for y in fragment.select(&selector) {
		let txt = y.text().collect::<Vec<_>>();
		if txt[0].len() == 5 {
			years.push(txt[0].to_string().replace("/",""));
		}
	}
	years
}
pub fn get_ages(year:String) -> Vec<String>{
	let url = "https://www.has.hr/images/stories/HAS/tabsez/".to_owned() + &year;
	let res = reqwest::blocking::get(&url).unwrap();
	let age = res.text_with_charset("windows-1250").unwrap();
	let fragment = Html::parse_fragment(&age.to_string());
	let selector = Selector::parse("a").unwrap();
	let mut ages = Vec::new();
	for y in fragment.select(&selector) {
		let txt = y.text().collect::<Vec<_>>();
		if txt[0].find(".html").unwrap_or(99) != 99 || txt[0].find(".htm").unwrap_or(99) != 99 {
			ages.push(txt[0].to_string());
		}
	}
	ages
}
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
fn getDataFromWebsite(url: &str) -> Vec<Vec<Vec<String>>> {
	let res = reqwest::blocking::get(url).unwrap();
	let data = res.text_with_charset("windows-1250").unwrap();
	let mut v = data.split("\n").collect::<Vec<&str>>();

	let mut total:Vec<Vec<Vec<String>>> = Vec::new();
	let mut page:String = "".to_string();
	let mut pos = 0;
	let mut lpos = 0;
	let mut index = 0;
	let mut meta = false;
	let mut metapos = Vec::new();
	for x in 0..v.len(){
		if meta == true{
			if v[x].find("(").unwrap_or(99) != 99{
				metapos.push(v[x].clone());
			}
			meta = false;
		}
		if v[x].find("<b>").unwrap_or(99) != 99 || v[x].find("</html>").unwrap_or(99) != 99{
			if pos > 37{
				if lpos != 0{
					meta = true;
					total.push(extractdata(page.as_str(), v[lpos].replace("<b>","").replace("</b>","").as_str(), metapos.clone()));
					metapos.clear();
					page = "".to_string();
					index += 1;
					lpos = 0;
				}
				lpos = pos;
			}
		}else{
			if lpos != 0 {
				if v[x].to_string().replace(" ","") == "    nema podataka o vjetru / no wind information".to_string().replace(" ","") || v[x].to_string().replace(" ","") ==  "    uz pomoć vjetra / wind assisted".to_string().replace(" ",""){
					metapos.push(v[x].clone());
					metapos.push( v[x+1].clone());
				}
				page.push_str(v[x]);
				page.push_str("\n");
			}
		}
		pos += 1;
	}
	total
}
fn extractdata(text: &str, discp: &str, extra: Vec<&str>) -> Vec<Vec<String>> {
    lazy_static! {
    	static ref wind: Regex = Regex::new(r"([+-]+\d,\d)| 0,0").unwrap();
        static ref re1: Regex = Regex::new(r"(?u)(\d*[;:]*\d*:*\d+,\d{1,2}|\d*[;:]*\d*:\d+) ([ +-]{0,1}\d,\d)* +([A-Za-zčćžšđČĆŽŠĐ ]+)(\d{2}.\d{2}.\d{4}) ([A-ZČĆŽŠĐ]+) +([A-Za-zčćžšđČĆŽŠĐ,/ ]+)(\d{2}.\d{2}.\d{4})").unwrap();
        static ref re2: Regex = Regex::new(r"(?um)(\d{3,}) +([A-Za-zčćžšđČĆŽŠĐ ]+) +(\d{2}.\d{2}.\d{4}) ([A-ZČĆŽŠĐ]+) +([A-Za-zčćžšđČĆŽŠĐ,/ ]+) +(\d{2}.\d{2}.\d{4})\s+<font style='font-size:7.0pt'>([a-z0-9/:+, -]+)").unwrap();
        static ref re3: Regex = Regex::new(r"(?um)(\d*:*\d+,\d{2}) +([A-Za-zčćžšđČĆŽŠĐ -]+[,-] [A-Za-zčćžšđČĆŽŠĐ]+ [A-Za-zčćžšđČĆŽŠĐ]*) +([A-Za-zčćžšđČĆŽŠĐ,/ ]+) +(\d{2}.\d{2}.\d{4})\s+<font style='font-size:7.0pt'>([A-Za-zčćžšđČĆŽŠĐ ]+[()0-9]+), ([A-Za-zčćžšđČĆŽŠĐ ]+[()0-9]+), ([A-Za-zčćžšđČĆŽŠĐ ]+[()0-9]+), ([A-Za-zčćžšđČĆŽŠĐ ]+[()0-9]+)").unwrap();
    }
	let mut line:Vec<String> = Vec::new();
    let mut result = Vec::new();
	let mut first = true;
	let mut bwind = false;
	if wind.is_match(text) == true {
		bwind = true;
	}
	let mut id = "";
	let mut p = 0;
	for caps in re1.captures_iter(text) {
		for x in caps.iter(){
			if first != true {
				match x {
					Some(i) => {
						if p == 1 && bwind == true{
							if wind.is_match(i.as_str()) == false {
								line.push("NO_WIND".to_string());
								line.push(i.as_str().replace("  ", ""));
							}else{
								line.push(i.as_str().replace("  ", ""));
							}
						}else {
							line.push(i.as_str().replace("  ", ""));
							id = "Single";
						}
						p += 1;
					},
					None => ()
				}
			}else{
				first = false;
			}
		}
		p = 0;
		result.push(line.clone());
		line.clear();
		first = true;
	}
	for caps in re2.captures_iter(text) {
		for x in caps.iter(){
			if first != true {
				match x {
					Some(i) => {line.push(i.as_str().replace("  "," ").replace("  ",""));id="Multi"},
					None => ()
				}
			}else{
				first = false;
			}
		}
		result.push(line.clone());
		line.clear();
		first = true;
	}
	for caps in re3.captures_iter(text) {
		for x in caps.iter(){
			if first != true {
				match x {
					Some(i) => {line.push(i.as_str().replace("  ",""));id="Rally"},
					None => ()
				}
			}else{
				first = false;
			}
		}
		result.push(line.clone());
		line.clear();
		first = true;
	}

	for x in 0..extra.len(){
		for i in 0..result.len(){
			match re1.captures(extra[x]){
				Some(p) => {
					match p.get(2) {
						Some(j) => {
							if result[i].len() > 1{
								if result[i][1] == j.as_str() && result[i][2].replace(" ","") == p.get(3).unwrap().as_str().replace(" ",""){
									result.insert(i,vec!(extra[x-1].to_string()));
									break;
								}
							}
						},
						None => {
							if result[i].len() > 1 {
								if result[i][1] == "NO_WIND" && result[i][2].replace(" ", "") == p.get(3).unwrap().as_str().replace(" ", "") {
									result.insert(i, vec!(extra[x - 1].to_string()));
									break;
								}
							}
						}
					}
				},
				None => ()
			}
		}
	}
	if extra.len() != 0{
		if extra[0].find("(").unwrap_or(99) != 99 {
			result.insert(0, vec!(extra[0].to_string()));
		}
	}
	if wind.is_match(text) == true && id != "Multi" {
		id = "Wind";
	}
	result.insert(0, vec!(id.to_string(), discp.to_string()));
	result
}

fn searchName(data: Vec<Vec<Vec<String>>>, search: &str) -> Vec<Vec<Vec<String>>> {
	let mut res = Vec::new();
	let search:String = search.to_lowercase().replace(" ", "").split_whitespace().collect();
	for x in 0..data.len(){
		if data[x].len() == 1{
			println!("{:?}",data[x]);
			res.push(data[x].clone());
		}else{
			for y in 1..data[x].len() {
				if data[x][y].len() > 2 {
					if data[x][0][0] == "Single" || data[x][0][0] == "Multi" {
						if data[x][y][1].to_lowercase().replace(" ", "").find(&search.clone()).unwrap_or(99) != 99 {
							res.push(vec!(data[x][0].clone(), data[x][y].clone()));
						}
					} else if data[x][0][0] == "Wind" {
						if data[x][y][2].to_lowercase().replace(" ", "").find(&search.clone()).unwrap_or(99) != 99 {
							res.push(vec!(data[x][0].clone(), data[x][y].clone()));
						}
					} else {
						if data[x][y][4].to_lowercase().replace(" ", "").find(&search.clone()).unwrap_or(99) != 99 {
							res.push(vec!(data[x][0].clone(), data[x][y].clone()));
						} else
						if data[x][y][5].to_lowercase().replace(" ", "").find(&search.clone()).unwrap_or(99) != 99 {
							res.push(vec!(data[x][0].clone(), data[x][y].clone()));
						} else
						if data[x][y][6].to_lowercase().replace(" ", "").find(&search.clone()).unwrap_or(99) != 99 {
							res.push(vec!(data[x][0].clone(), data[x][y].clone()));
						} else
						if data[x][y][7].to_lowercase().replace(" ", "").find(&search.clone()).unwrap_or(99) != 99 {
							res.push(vec!(data[x][0].clone(), data[x][y].clone()));
						}
					}
				}
			}
		}
	}
	res
}

fn get_categories(data: Vec<Vec<Vec<String>>>) -> Vec<String>{
	let mut res = Vec::new();
	for x in data{
		if x[0][1].find(".csv").unwrap_or(99) == 99 {
			res.push(x[0][1].clone());
		}
	}
	res
}
fn getDiscipline(data: Vec<Vec<Vec<String>>>, discip: &str) -> Vec<Vec<Vec<String>>> {
	let mut res = Vec::new();
	let discip = discip.replace("HJ", "High Jump")
		.replace("LJ", "Long Jump")
		.replace("TJ", "Triple Jump")
		.replace("PV", "Pole vault")
		.replace("SP", "Shoot put")
		.replace("DT", "Discus Throw")
		.replace("JT", "Javelin Throw")
		.replace("HT", "Hammer throw")
		.replace("mH", "mhurdles")
		.replace("mSC", "mSteeplechase")
		.replace("HM", "half marathon")
		.replace("mRW", "m race walking");
	for x in data{
		if x.len() == 1{
			res.push(x.clone());
		}
		if x[0][1].replace(" ","").to_lowercase().find(&discip.replace(" ","").to_lowercase()).unwrap_or(99) != 99{
			res.push(x.clone());
		}
	}
	res
}
pub fn CLI_input(question:&str) -> String {
	let mut ans:String= "".to_string();
	println!("{}", question);
	io::stdin()
		.read_line(&mut ans)
		.expect("Failed to read line");
	let ans = ans.replace("\n","");
	ans
}
pub fn CLI_question(question:&str, answers:Vec<String>) -> String{
	let mut ans:String= "".to_string();
	println!("{}", question);
	if answers.len() == 2{
		println!("{} or {}",answers[0],answers[1]);
	}else{
		for x in 0..answers.len(){
			println!("{} -> {}",x,answers[x]);
		}
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
fn CLI_question_alias(question:&str, answers:Vec<String>, alias:Vec<Vec<String>>) -> String{
	let mut ans= "".to_string();
	println!("{}", question);
	for x in 0..alias.len() {
		let mut pat = String::new();
		for y in 0..alias[x].len()-1{
			pat.push_str(&*format!("{} or ", alias[x][y]));
		}
		pat.push_str(&*format!("{}", alias[x][alias[x].len()-1]));
		println!("{}", pat);
	}
	let mut res;
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
			}
		}
		println!("Invaild answer");
	}
	answers[res].clone()
}
fn displayTable(data: Vec<Vec<Vec<String>>>) {
	for x in 0..data.len() {
		let mut table = term_table::Table::new();
		table.max_column_width = 60;
		table.style = term_table::TableStyle::extended();
		for y in 0..data[x].len() {
			let mut rows = Vec::new();
			let mut added = false;
			for z in 0..data[x][y].len() {
				if data[x].len() > 1 {
					if data[x][y].len() != 1 {
						rows.push(TableCell::new(data[x][y][z].clone()));
						added = true;
					} else {
						rows.push(TableCell::new_with_alignment(data[x][y][z].clone(), data[x][1].len(), term_table::table_cell::Alignment::Center));
						added = true;
					}
				}
			}
			if added == true{
				table.add_row(Row::new(rows));
			}
		}
		println!("{}", table.render());
	}
}
fn saveCSV(data: Vec<Vec<Vec<String>>>){
	let mut csv:String = String::new();
	let mut csvtotal:String = String::new();
	let mut first = true;
	let mut lpos = 0;
	for x in 0..data.len(){
		if data[x].len() != 1 {
			for y in 0..data[x].len(){
					if y == 0 {
						if data[x][y].len() == 2 {
							csv += &(data[x][y][1].to_string() + "\n");
						} else {
							csv += &(data[x][y][1].to_string() + "\t");
							csv += &(data[x][y][2].to_string() + "\n");
						}
						if data[x][y][0] == "Single" {
							csv += "Result\tName_Surname\tBirthday\tClub\tCity\tDate\n";
						} else if data[x][y][0] == "Wind" {
							csv += "Result\tWind\tName_Surname\tBirthday\tClub\tCity\tDate\n";
						} else if data[x][y][0] == "Multi" {
							csv += "Score\tName_Surname\tBirthday\tClub\tCity\tDate\tResults\n";
						} else {
							csv += "Result\tClub\tCity\tDate\tRunner_1\tRunner_2\tRunner_3\tRunner_4\n";
						};
					} else {
						for z in 0..data[x][y].len() - 1 {
							csv += &(data[x][y][z].to_string() + "\t");
						}
						csv += &(data[x][y][data[x][y].len() - 1].to_string() + "\n");
					}
				}
		}else{
			if first == true && data[x][0].len() == 2 && data[x][0][1].find(".csv").unwrap_or(99) != 99 {
				lpos = x;
				first = false;
			}else if data[x][0].len() == 2 && data[x][0][1].find(".csv").unwrap_or(99) != 99 {
				fs::create_dir_all("/has/".to_owned() + &data[lpos][0][0].clone() + &*"/".to_owned());
				fs::write("/has/".to_owned() + &data[lpos][0][0].clone() + &*"/".to_owned() + &data[lpos][0][1].clone(), csv.clone()).expect("Unable to write file");
				csvtotal += &(csv);
				csv.clear();
				lpos = x;
			}
		}
	}

	println!("{}", "/has/".to_owned() + &data[lpos][0][0].clone() + &*"/total.csv");
	csvtotal += &(csv);
	fs::write("/has/".to_owned() + &data[lpos][0][0].clone() + &*"/total.csv", csvtotal.clone()).expect("Unable to write file");
}
fn get_category_alias(category: String) -> Vec<String>{
	let mut res = Vec::<String> ::new();
	let split = category.split("-").collect::<Vec<_>>();
	let dual = split[1].split("/").collect::<Vec<_>>();
	if dual.len() == 1{
		res.push(dual[0].replace(" ",""));
	}else{
		res.push(dual[0].replace("",""));
		res.push(dual[1].replace("",""));
		res.push(dual[1].replace("High Jump",       "HJ" )
		.replace("Long Jump",       "LJ" )
		.replace("Triple Jump",     "TJ" )
		.replace("Pole Vault",      "PV" )
		.replace("Shoot Put",       "SP" )
		.replace("Discus Throw",    "DT" )
		.replace("Javelin Throw",   "JT" )
		.replace("Hammer Throw",    "HT" )
		.replace("m Hurdles",        "mH" )
		.replace( "m Steeplechase",  "mSC")
		.replace("Half Marathon",   "HM" )
		.replace("Race Walking",  "RW")
		.replace("(Road Race)","")
		.replace("Road","")
		.replace(" ",""));
	}
	res
}
fn main() {
	let year = CLI_question("Choose a year",get_years());
	let age:String;
	let category:String;
	let ans = CLI_question("Do you want to see all ages", vec!("yes".to_string(),"no".to_string()));
	let mut data:Vec<Vec<Vec<String>>> = vec!(vec!(vec!("".to_string())));
	if ans == "yes"{
		let bar = ProgressBar::new(get_ages(year.clone()).len() as u64);
		let mut lpos = 0;
		for x in get_ages(year.clone()){
			if lpos != 0{
				lpos += data.len()-lpos;
			}else{
				lpos = 1;
			}
			data.append(&mut getDataFromWebsite(&*("https://www.has.hr/images/stories/HAS/tabsez/".to_owned() + &year.clone() + "/" + &x.clone())));
			data.insert(lpos, vec!(vec!(year.clone(), x.clone().replace(".html",".csv").replace(".htm",".csv"))));
			bar.inc(1);
		}
		bar.finish();
	}else{
		let mut alias:Vec<Vec<String>> = vec![];
		for x in get_ages(year.clone()){
			alias.push(get_age_alias(x, year.clone()));
		}
		age = CLI_question_alias("Choose a age",get_ages(year.clone()), alias);
		data = getDataFromWebsite(&*("https://www.has.hr/images/stories/HAS/tabsez/".to_owned() + &year.clone() + "/" + &age.clone()));
		data.insert(0,vec!(vec!(year.clone() , age.clone().replace(".html",".csv").replace(".htm",".csv"))));
		let ans = CLI_question("Do you want to see all categories", vec!("yes".to_string(),"no".to_string()));
		if ans == "no"{
			let mut alias:Vec<Vec<String>> = vec![];
			for x in get_categories(data.clone()){
				alias.push(get_category_alias(x));
			}
			category = CLI_question_alias("Choose a category", get_categories(data.clone()),alias);
			data = getDiscipline(data.clone(), &*category);
		}
	}
	let ans = CLI_question("Do you want filter the results", vec!("yes".to_string(),"no".to_string()));
	if ans == "yes"{
		let key = CLI_input("What do you want to search for");
		data = searchName(data.clone(), &*key);
	}
	let ans = CLI_question("Do you want to save the result", vec!("yes".to_string(),"no".to_string()));
	if ans == "yes"{
		saveCSV(data.clone());
	}
	displayTable(data.clone());
}
