#[macro_use] extern crate lazy_static;
extern crate regex;

use regex::Regex;
use std::{fs, io};
use std::path::Path;
use scraper::{Selector, Html};
use term_table::table_cell::TableCell;
use term_table::row::Row;
use indicatif::{MultiProgress, ProgressBar};
use std::collections::HashSet;
use std::hash::Hash;
use std::cmp::Eq;
use std::net::SocketAddr;
//Funkcija za uzimanje godina omogucuje web hopping na hassovoj stranicu
pub fn get_years_net() -> Vec<String> {
	let res = reqwest::blocking::get("https://www.has.hr/images/stories/HAS/tabsez");
	let res = match res{
		Ok(res) => {
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
		},
		Err(e) => vec!("Failed".to_string())
	};
	res
}
//funkcija za uzimanje kategorija omogucuje uzimanje podataka s hasove stranice
pub fn get_ages_net(year:String) -> Vec<String>{
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
		.replace("&", "o");
	data
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
//uzima prvobitne podatke iz hasovih podataka
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
//uzima prvobitne podatke iz hasovih podataka
fn getDataFromCache(pathFile: &str) -> Vec<Vec<Vec<String>>> {
	let data: String = fs::read_to_string(pathFile).unwrap();
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
			if pos > 36{
				if lpos != 0{
					meta = true;
					total.push(extractdata(page.as_str(), v[lpos].replace("      <pre><font face=\"Courier New, Courier, mono\">","").replace("<br>","").replace("<b>","").replace("</b>","").as_str(), metapos.clone()));
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
//iz prvobitne podatke pretvara u polja
fn extractdata(text: &str, discp: &str, extra: Vec<&str>) -> Vec<Vec<String>> {
    lazy_static! {
		//regexi za uzimanje podataka i sortiranje njih
		//vjetar
    	static ref wind: Regex = Regex::new(r"([+-]+\d,\d)|0,0").unwrap();
		//single
        static ref re1: Regex = Regex::new(r"(?u)(\d*[;:]*\d*:*\d+,\d{1,2}|\d*[;:]*\d*:\d+) ([ +-]{0,1}\d,\d)* +([A-Za-zčćžšđČĆŽŠĐ ]+)(\d{2}.\d{2}.\d{4}|\d{4}) ([A-ZČĆŽŠĐ-]{0,}) +([\p{L},/ .]+)(\d{2}.\d{2}.\d{4})").unwrap();
		//multi
        static ref re2: Regex = Regex::new(r"(?um)(\d{3,}) +([A-Za-zčćžšđČĆŽŠĐ ]+) +(\d{2}.\d{2}.\d{4}|\d{4}) ([A-ZČĆŽŠĐ-]{0,}) +([A-Za-zčćžšđČĆŽŠĐ,/ ]+) +(\d{2}.\d{2}.\d{4})\s+<font style='font-size:7.0pt'>([a-z0-9/:+, -]+)").unwrap();
		//rallay
        static ref re3: Regex = Regex::new(r"(?um)(\d*:*\d+,\d{2}) +([A-ZČĆŽŠĐ ,-]+|[A-ZČĆŽŠĐ ,-]+ [ -] [A-Za-zčžćšđČĆŽŠĐ]*)[ ]{5,}([A-Za-zčćžšđČĆŽŠĐ,/ ]+) +(\d{2}.\d{2}.\d{4})\s+(?:<font style='font-size:7.0pt'>|<font size=' 1'>)([A-Za-zčćžšđČĆŽŠĐ ]+[()0-9]+), ([A-Za-zčćžšđČĆŽŠĐ ]+[()0-9]+), ([A-Za-zčćžšđČĆŽŠĐ ]+[()0-9]+), ([A-Za-zčćžšđČĆŽŠĐ ]+[()0-9]+)").unwrap();
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
//pretaživanje podataka
fn searchName(data: Vec<Vec<Vec<String>>>, search: String) -> Vec<Vec<Vec<String>>> {
	let mut res = Vec::new();
	let search:String = search.to_lowercase().replace(" ", "").split_whitespace().collect();
	let mut found = true;
	let mut count = 0;
	let mut last = 0;
	for x in 0..data.len(){
		if data[x].len() == 1{
			res.push(data[x].clone());
		}else{
			for y in 1..data[x].len() {
				if data[x][y].len() > 2 {
					if data[x][0][0] == "Single" || data[x][0][0] == "Multi" {
						if data[x][y][2].to_lowercase().replace(" ", "") == search.to_lowercase().replace(" ","") {
							res.push(vec!(data[x][0].clone(), data[x][y].clone()));
							found = true;
							count = count + 1;
						}
					} else if data[x][0][0] == "Wind" {
						if data[x][y][3].to_lowercase().replace(" ", "") == search.to_lowercase().replace(" ","") {
							res.push(vec!(data[x][0].clone(), data[x][y].clone()));
							found = true;
							count = count + 1;
						}
					} else if data[x][0][0] == "Rally" {
						if data[x][y][5].to_lowercase().replace(" ", "").find(&search.clone()).unwrap_or(99) != 99 {
							res.push(vec!(data[x][0].clone(), data[x][y].clone()));
							found = true;
							count = count + 1;
						} else
						if data[x][y][6].to_lowercase().replace(" ", "").find(&search.clone()).unwrap_or(99) != 99 {
							res.push(vec!(data[x][0].clone(), data[x][y].clone()));
							found = true;
							count = count + 1;
						} else
						if data[x][y][7].to_lowercase().replace(" ", "").find(&search.clone()).unwrap_or(99) != 99 {
							res.push(vec!(data[x][0].clone(), data[x][y].clone()));
							found = true;
							count = count + 1;
						} else
						if data[x][y][8].to_lowercase().replace(" ", "").find(&search.clone()).unwrap_or(99) != 99 {
							res.push(vec!(data[x][0].clone(), data[x][y].clone()));
							found = true;
							count = count + 1;
						}
					}
				}
			}
		}
	}
	res
}
//uzimanje svih disciplina iz polja
fn get_categories(data: Vec<Vec<Vec<String>>>) -> Vec<String>{
	let mut res = Vec::new();
	for x in data{
		if x[0][1].find(".csv").unwrap_or(99) == 99 {
			res.push(x[0][1].clone());
		}
	}
	res
}
//uzimanje jedne discipline iz polja
fn getDiscipline(data: Vec<Vec<Vec<String>>>, discip: &str) -> Vec<Vec<Vec<String>>> {
	let mut res = Vec::new();
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
//funkcija za CLI za unos podataka
pub fn CLI_input(question:&str) -> String {
	let mut ans:String= "".to_string();
	println!("{}", question);
	io::stdin()
		.read_line(&mut ans)
		.expect("Failed to read line");
	let ans = ans.replace("\n","");
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
			}
		}
		println!("Invaild answer");
	}
	answers[res].clone()
}
//Prikaz tablice
fn displayTable(data: Vec<Vec<Vec<String>>>) {
	if data[0][0][0] != "c"{
		for x in 1..data.len() {
			let mut table = term_table::Table::new();
			table.max_column_width = 60;
			table.style = term_table::TableStyle::extended();
			if (data[x][0][0] == "" ||data[x][0][0] == "Multi" ||data[x][0][0] == "Single" ||data[x][0][0] == "Wind" || data[x][0][0] == "Rally" ) && data[x].len() != 1{
				table.add_row(Row::new(vec![
					TableCell::new_with_alignment(data[x][0][1].to_string(), data[x][1].len(), term_table::table_cell::Alignment::Center)
				]));
				for y in 1..data[x].len() {
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
			}
			println!("{}", table.render());
		}
	}else{
		let mut table = term_table::Table::new();
		table.max_column_width = 60;
		table.style = term_table::TableStyle::extended();
		for x in 0..data[1].len(){
			let mut rows = Vec::new();
			for y in 0..data[1][x].len(){
				rows.push(TableCell::new(data[1][x][y].clone()));
			}
			table.add_row(Row::new(rows));
		}
		println!("{}", table.render());
	}
}
//funkcija za spremanje
fn saveCSV(data: Vec<Vec<Vec<String>>>){
	let mut csv:String = String::new();
	let mut csvtotal:String = String::new();
	let mut first = true;
	let mut lpos = 0;
	csv += "sep=~\n";
	if data[0][0][0] == ""{
		for x in 0..data.len(){
			if data[x].len() != 1 {
				for y in 0..data[x].len(){
						if y == 0 {
							if data[x][y].len() == 2 {
								csv += &(data[x][y][1].to_string() + "\n");
							} else {
								csv += &(data[x][y][1].to_string() + "~");
								csv += &(data[x][y][2].to_string() + "\n");
							}
						} else {
							for z in 0..data[x][y].len() - 1 {
								csv += &(data[x][y][z].to_string() + "~");
							}
							csv += &(data[x][y][data[x][y].len() - 1].to_string() + "\n");
						}
					}
			}else{
				if first == true && data[x][0].len() == 2 && data[x][0][1].find(".csv").unwrap_or(99) != 99 {
					lpos = x;
					first = false;
				}else if data[x][0].len() == 2 && data[x][0][1].find(".csv").unwrap_or(99) != 99 {
					csvtotal += &(csv);
					csv.clear();
					lpos = x;
				}
		   	}
		}
		csvtotal += &(csv);
		fs::create_dir_all("/has/export/".to_owned());
		fs::write("/has/export/".to_owned() + &data[0][0][0].clone()+&data[1][0][1].clone() + &*"_total.csv", csvtotal.clone()).expect("Unable to write file");
	}else if data[0][0][0] == "c"{
		for y in 0..data[1].len(){
				for z in 0..data[1][y].len()-1{
					csv += &(data[1][y][z].to_string() + "~");
				}
				csv += &(data[1][y][data[1][y].len()-1].to_string() + "\n");
			}
		fs::create_dir_all("/has/export/".to_owned() + &data[1][0][0].clone() + &*"/".to_owned());
		fs::write("/has/export/".to_owned() + &data[1][0][0].clone() + &*"/" + &data[1][0][0].clone() + &*"_career.csv", csv.clone()).expect("Unable to write file");
	}
}
//uzimanje nadimka za discipline
fn get_category_alias(category: String) -> Vec<String>{
	let mut res = Vec::<String>::new();
	let split = category.split("- ").collect::<Vec<_>>();
	let mut dual = split[1].split(" / ").collect::<Vec<_>>();
	let mut cat = split[0].split(" / ").collect::<Vec<_>>();
	if dual.len() == 1{
		if cat[1].replace(" ","") != "Mixed"{
			res.push(dual[0].replace(".","").replace(" ","").replace("4x400","4x400m").replace("mm","m").replace("\r",""));
		}else{
			res.push(dual[0].replace(".","").replace(" ","").replace("4x","X4x").replace("4x400","4x400m").replace("mm","m").replace("\r",""));
		}
	}else{
		res.push(dual[1].replace(".","").replace("High Jump",       "HJ" )
		.replace("Long Jump",       "LJ" )
		.replace("Triple Jump",     "TJ" )
		.replace("Pole Vault",      "PV" )
		.replace("Shot Put",       "SP" )
		.replace("Discus Throw",    "DT" )
		.replace("Javelin Throw",   "JT" )
		.replace("Hammer Throw",    "HT" )
		.replace("m Hurdles",        "mH" )
		.replace("m Steepl.", "mSC")
		.replace( "m Steeplechase",  "mSC")
		.replace("m Steepl", "mSC")
		.replace("Half Marathon",   "HM" )
		.replace("Race Walking",  "W")
		.replace("(Road Race)","")
		.replace("Road","")
		.replace(" ","")
		.replace("000mW","kmW")
		.replace("000W","kmW").replace("Relay","").replace("\r",""));
		res.push(dual[1].replace("\r",""));
		res.push(dual[0].replace("\r",""));
	}
	res
}
//modifciranje polja tako da se dota metadata i ranking
fn modify_data(mut data: Vec<Vec<Vec<String>>>, age: String, year: String) -> Vec<Vec<Vec<String>>>{
	data.insert(0,vec!(vec!(year.clone() , age.clone().replace(".html",".csv").replace(".htm",".csv"))));
	for x in 0..data.len(){
		let mut wind = false;
		for y in 1..data[x].len(){
			if data[x][y][0].to_lowercase().replace(" ","").find(&"uz pomoć vjetra / wind assisted".to_lowercase().replace(" ","")).unwrap_or(99) != 99 || data[x][y][0].to_lowercase().replace(" ","").find(&"nema podataka o vjetru / no wind information".to_lowercase().replace(" ","")).unwrap_or(99) != 99{
				wind = true;
				continue;
			}
			if wind == false{
				data[x][y].insert(0, y.to_string());
			}else{
				data[x][y].insert(0, "Invaild".to_string());
			}
		}
		if data[x][0][0] == "Single" || data[x][0][0] == "Multi" {
			data[x].insert(1, vec!("Rank".to_string(),"Result".to_string(),"Name".to_string(),"Birthday".to_string(),"Club".to_string(),"City".to_string(),"Date".to_string()));
		} else if data[x][0][0] == "Wind" {
			data[x].insert(1, vec!( "Rank".to_string(),"Result".to_string(),"Wind".to_string(),"Name".to_string(),"Birthday".to_string(),"Club".to_string(),"City".to_string(),"Date".to_string()));
		} else if data[x][0][0] == "Rally"{
			data[x].insert(1, vec!("Rank".to_string(),"Result".to_string(),"Team".to_string(),"City".to_string(),"Date".to_string(),"1st runner".to_string(),"2nd runner".to_string(),"3rd runner".to_string(),"4th runner".to_string()));
		}
		for y in 2..data[x].len(){
			if data[x][y].len() > 4{		
				let mut result:f64 = 0.0;
				let stemp = data[x][y][1].replace(";",":");
				let minutes = stemp.split(":").collect::<Vec<_>>();
				if minutes.len() == 1{
					result = minutes[0].replace(",",".").parse::<f64>().unwrap();
				}else if minutes.len() == 2{
					result = minutes[0].parse::<f64>().unwrap()*60.0+minutes[1].replace(",",".").parse::<f64>().unwrap();
				}else{
					result = minutes[0].parse::<f64>().unwrap()*60.0*60.0+minutes[1].parse::<f64>().unwrap()*60.0+minutes[2].replace(",",".").parse::<f64>().unwrap();
				}
				if data[x][0][0] != "Multi" {
					data[x][y][1] = format_result(result);
				}
			}
		}
	}
	data
}
//cleanup of data
fn clean_data(data: Vec<Vec<Vec<String>>>) -> Vec<Vec<Vec<String>>>{
	let mut res: Vec<Vec<Vec<String>>> = Vec::new();
	for x in 0..data.len(){
		if data[x].len() > 1{
			res.push(data[x].clone());
		}
	}
	res
}
//uzimanje profila atleticra u jednoj godini
fn get_single_profile(data: Vec<Vec<Vec<String>>>, name: String) -> Vec<Vec<String>>{
	let mut temp = searchName(data.clone(), name.clone());
	let mut res:Vec<Vec<Vec<String>>> = Vec::new();
	let mut export:Vec<Vec<String>> = Vec::new();
	if data.len() != 0{
		res.insert(0,data[0].clone());
		let mut rally = true;
		for x in 0..temp.len(){
			if temp[x][0][0] != "Rally"{
				rally = false;
			}
		}
		let mut names:Vec<Vec<String>> = Vec::new();
		for x in 0..temp.len(){
			if temp[x][0][0] == "Single" || temp[x][0][0] == "Multi" {
				let s = temp[x][1][2].clone().replace(" ","_");
				let s = s + "__";
				let s = s.replace("___","").replace("__","").replace("_"," ");
				names.push(vec!(s.clone(),temp[x][1][3][temp[x][1][3].len()-4..temp[x][1][3].len()].to_string()));
			} else if temp[x][0][0] == "Wind" {
				let s = temp[x][1][3].clone().replace(" ","_");
				let s = s + "__";
				let s = s.replace("___","").replace("__","").replace("_"," ");
				names.push(vec!(s.clone(),temp[x][1][4][temp[x][1][4].len()-4..temp[x][1][4].len()].to_string()));
			} else if temp[x][0][0] == "Rally"{
				if temp[x][1][5].to_lowercase().find(&name.to_lowercase().clone()).unwrap_or(99) != 99 {
					names.push(vec!(temp[x][1][5][0..temp[x][1][5].len()-7].to_string().clone(),temp[x][1][5][temp[x][1][5].len()-5..temp[x][1][5].len()-1].to_string()));
				} else
				if temp[x][1][6].to_string().find(&name.to_lowercase().clone()).unwrap_or(99) != 99 {
					names.push(vec!(temp[x][1][6][0..temp[x][1][6].len()-7].to_string().clone(),temp[x][1][6][temp[x][1][6].len()-5..temp[x][1][6].len()-1].to_string()));
				} else
				if temp[x][1][7].to_lowercase().find(&name.to_lowercase().clone()).unwrap_or(99) != 99 {
					names.push(vec!(temp[x][1][7][0..temp[x][1][7].len()-7].to_string().clone(),temp[x][1][7][temp[x][1][7].len()-5..temp[x][1][7].len()-1].to_string()));
				} else
				if temp[x][1][8].to_lowercase().find(&name.to_lowercase().clone()).unwrap_or(99) != 99 {
					names.push(vec!(temp[x][1][8][0..temp[x][1][8].len()-7].to_string().clone(),temp[x][1][8][temp[x][1][8].len()-5..temp[x][1][8].len()-1].to_string()));
				}
			}
		}
		
		let set : HashSet<_> = names.drain(..).collect();
		names.extend(set.into_iter());
		let mut last:Vec<String> = Vec::new();
		let mut offset = 0;
		if names.len() != 0{
			for y in 0..temp.len(){
				if temp[y].len() == 1 && temp[y][0][0] != ""{
					last.clear();
					last.push(temp[y][0][0].clone());
					last.push(temp[y][0][1].clone());	
					res.push(vec!(vec!(temp[y][0][0].clone(),names[0][0].clone())));
					offset = offset + 1;
				}
				if (temp[y][0][0] == "Single" || temp[y][0][0] == "Multi" || temp[y][0][0] == "Rally" || temp[y][0][0] == "Wind"){
					let age_data = get_age_data(last[1].clone(),last[0].clone()).replace("fo","").replace("fi"," Indoor").replace("mo","").replace("mi"," Indoor");
					let mut result:f64 = 0.0;
					let stemp = temp[y][1][1].replace(";",":");
					let minutes = stemp.split(":").collect::<Vec<_>>();
					if minutes.len() == 1{
						result = minutes[0].replace(",",".").parse::<f64>().unwrap();
					}else if minutes.len() == 2{
						result = minutes[0].parse::<f64>().unwrap()*60.0+minutes[1].replace(",",".").parse::<f64>().unwrap();
					}else{
						result = minutes[0].parse::<f64>().unwrap()*60.0*60.0+minutes[1].parse::<f64>().unwrap()*60.0+minutes[2].replace(",",".").parse::<f64>().unwrap();
					}
					if temp[y][0][0] == "Single" || temp[y][0][0] == "Multi" {
						let s = temp[y][1][2].clone().replace(" ","_");
						let s = s + "__";
						let s = s.replace("___","").replace("__","").replace("_"," ");
						if names[0][0] == s{
							res[offset].push(vec!(get_category_alias(temp[y][0][1].clone())[0].clone() + &age_data.clone(),temp[y][1][0].clone(),temp[y][1][1].clone(),calculate_points(result.clone(),get_category_alias(temp[y][0][1].clone())[0].clone(),last[1].clone(),last[0].clone())));
						}
					}else if temp[y][0][0] == "Wind" {
						let s = temp[y][1][3].clone().replace(" ","_");
						let s = s + "__";
						let s = s.replace("___","").replace("__","").replace("_"," ");
						if names[0][0] == s{
							if temp[y][1][0] != "Invalid"{
								res[offset].push(vec!(get_category_alias(temp[y][0][1].clone())[0].clone() + &age_data.clone(),temp[y][1][0].clone(),temp[y][1][1].clone(),calculate_points(result.clone(),get_category_alias(temp[y][0][1].clone())[0].clone(),last[1].clone(),last[0].clone())));
							}	
						}				
					}else if temp[y][0][0] == "Rally"{
						let name = names[0][0].clone() + &" (".to_string() + &names[0][1].clone() + &")".to_string();
						if temp[y][1][5].to_lowercase() == name.to_lowercase() {
							res[offset].push(vec!(get_category_alias(temp[y][0][1].clone())[0].clone() + &age_data.clone(),temp[y][1][0].clone(),temp[y][1][1].clone(),calculate_points(result.clone(),get_category_alias(temp[y][0][1].clone())[0].clone(),last[1].clone(),last[0].clone())));
						} else
						if temp[y][1][6].to_lowercase() == name.to_lowercase() {
							res[offset].push(vec!(get_category_alias(temp[y][0][1].clone())[0].clone() + &age_data.clone(),temp[y][1][0].clone(),temp[y][1][1].clone(),calculate_points(result.clone(),get_category_alias(temp[y][0][1].clone())[0].clone(),last[1].clone(),last[0].clone())));
						} else
						if temp[y][1][7].to_lowercase() == name.to_lowercase() {
							res[offset].push(vec!(get_category_alias(temp[y][0][1].clone())[0].clone() + &age_data.clone(),temp[y][1][0].clone(),temp[y][1][1].clone(),calculate_points(result.clone(),get_category_alias(temp[y][0][1].clone())[0].clone(),last[1].clone(),last[0].clone())));
						} else
						if temp[y][1][8].to_lowercase() == name.to_lowercase() {
							res[offset].push(vec!(get_category_alias(temp[y][0][1].clone())[0].clone() + &age_data.clone(),temp[y][1][0].clone(),temp[y][1][1].clone(),calculate_points(result.clone(),get_category_alias(temp[y][0][1].clone())[0].clone(),last[1].clone(),last[0].clone())));
						}
					}					
				}
			}
			let mut discip:Vec<String> = Vec::new();
			for x in res.clone(){
				for y in x{
					if y.len() > 2{
						if discip.len() == 0{
							discip.push(y[0].clone());
							export.push(y.clone());
						}else{
							let mut found = false;
							for d in discip.clone() {
								if d == y[0]{
									found = true;
								}
							}
							if found == false{
								discip.push(y[0].clone());
								export.push(y.clone());
							}
						}
					}
				}
			}
			export.insert(0,res[1][0].clone());
		}
	}
	export
}
//uzimanje profila atleticra u jednoj godini
fn get_profile(data: Vec<Vec<Vec<String>>>, name: String) -> Vec<Vec<Vec<String>>>{
	let mut temp = searchName(data.clone(), name.clone());
	let mut res:Vec<Vec<Vec<String>>> = Vec::new();
	if data.len() != 0{
	res.insert(0,data[0].clone());
	let mut rally = true;
	for x in 0..temp.len(){
		if temp[x][0][0] != "Rally"{
			rally = false;
		}
	}
	let mut names:Vec<Vec<String>> = Vec::new();
	for x in 0..temp.len(){
		if temp[x][0][0] == "Single" || temp[x][0][0] == "Multi" {
			let s = temp[x][1][2].clone().replace(" ","_");
			let s = s + "__";
			let s = s.replace("___","").replace("__","").replace("_"," ");
			println!("{}",temp[x][1][3].len());
			names.push(vec!(s.clone(),temp[x][1][3][temp[x][1][3].len()-4..temp[x][1][3].len()].to_string()));
		} else if temp[x][0][0] == "Wind" {
			let s = temp[x][1][3].clone().replace(" ","_");
			let s = s + "__";
			let s = s.replace("___","").replace("__","").replace("_"," ");
			names.push(vec!(s.clone(),temp[x][1][4][6..10].to_string()));
		} else if temp[x][0][0] == "Rally"{
			if temp[x][1][5].to_lowercase().find(&name.to_lowercase().clone()).unwrap_or(99) != 99 {
				names.push(vec!(temp[x][1][5][0..temp[x][1][5].len()-7].to_string().clone(),temp[x][1][5][temp[x][1][5].len()-5..temp[x][1][5].len()-1].to_string()));
			} else
			if temp[x][1][6].to_string().find(&name.to_lowercase().clone()).unwrap_or(99) != 99 {
				names.push(vec!(temp[x][1][6][0..temp[x][1][6].len()-7].to_string().clone(),temp[x][1][6][temp[x][1][6].len()-5..temp[x][1][6].len()-1].to_string()));
			} else
			if temp[x][1][7].to_lowercase().find(&name.to_lowercase().clone()).unwrap_or(99) != 99 {
				names.push(vec!(temp[x][1][7][0..temp[x][1][7].len()-7].to_string().clone(),temp[x][1][7][temp[x][1][7].len()-5..temp[x][1][7].len()-1].to_string()));
			} else
			if temp[x][1][8].to_lowercase().find(&name.to_lowercase().clone()).unwrap_or(99) != 99 {
				names.push(vec!(temp[x][1][8][0..temp[x][1][8].len()-7].to_string().clone(),temp[x][1][8][temp[x][1][8].len()-5..temp[x][1][8].len()-1].to_string()));
			}
		}
	}

	let set : HashSet<_> = names.drain(..).collect();
    names.extend(set.into_iter());
	let mut last:Vec<String> = Vec::new();
	let mut offset = 0;
	for x in 0..names.len(){
		for y in 0..temp.len(){
			if temp[y].len() == 1 && temp[y][0][0] != ""{
				last.clear();
				last.push(temp[y][0][0].clone());
				last.push(temp[y][0][1].clone());	
				res.push(vec!(vec!("".to_string(),names[x][0].clone() + " " + get_age_alias(temp[y][0][1].clone(),temp[y][0][0].clone())[0].replace(".csv","").as_str())));
				offset = offset + 1;
			}
			if (temp[y][0][0] == "Single" || temp[y][0][0] == "Multi" || temp[y][0][0] == "Rally" || temp[y][0][0] == "Wind"){
				
				let mut result:f64 = 0.0;
				let stemp = temp[y][1][1].replace(";",":");
				let minutes = stemp.split(":").collect::<Vec<_>>();
				if minutes.len() == 1{
					result = minutes[0].replace(",",".").parse::<f64>().unwrap();
				}else if minutes.len() == 2{
					result = minutes[0].parse::<f64>().unwrap()*60.0+minutes[1].replace(",",".").parse::<f64>().unwrap();
				}else{
					result = minutes[0].parse::<f64>().unwrap()*60.0*60.0+minutes[1].parse::<f64>().unwrap()*60.0+minutes[2].replace(",",".").parse::<f64>().unwrap();
				}
				if temp[y][0][0] == "Single" || temp[y][0][0] == "Multi" {
					let s = temp[y][1][2].clone().replace(" ","_");
					let s = s + "__";
					let s = s.replace("___","").replace("__","").replace("_"," ");
					if names[x][0] == s{
						res[offset].push(vec!(get_category_alias(temp[y][0][1].clone())[0].clone(),temp[y][1][0].clone(),temp[y][1][1].clone(),calculate_points(result.clone(),get_category_alias(temp[y][0][1].clone())[0].clone(),last[1].clone(),last[0].clone())));
					}
				}else if temp[y][0][0] == "Wind" {
					let s = temp[y][1][3].clone().replace(" ","_");
					let s = s + "__";
					let s = s.replace("___","").replace("__","").replace("_"," ");
					if names[x][0] == s{
						if temp[y][1][0] != "Invalid"{
							res[offset].push(vec!(get_category_alias(temp[y][0][1].clone())[0].clone(),temp[y][1][0].clone(),temp[y][1][1].clone(),calculate_points(result.clone(),get_category_alias(temp[y][0][1].clone())[0].clone(),last[1].clone(),last[0].clone())));					
						}	
					}				
				}else if temp[y][0][0] == "Rally"{
					let name = names[x][0].clone() + &" (".to_string() + &names[x][1].clone() + &")".to_string();					
					if temp[y][1][5].to_lowercase() == name.to_lowercase() {
						res[offset].push(vec!(get_category_alias(temp[y][0][1].clone())[0].clone(),temp[y][1][0].clone(),temp[y][1][1].clone(),calculate_points(result.clone(),get_category_alias(temp[y][0][1].clone())[0].clone(),last[1].clone(),last[0].clone())));				
					} else
					if temp[y][1][6].to_lowercase() == name.to_lowercase() {
						res[offset].push(vec!(get_category_alias(temp[y][0][1].clone())[0].clone(),temp[y][1][0].clone(),temp[y][1][1].clone(),calculate_points(result.clone(),get_category_alias(temp[y][0][1].clone())[0].clone(),last[1].clone(),last[0].clone())));
					} else
					if temp[y][1][7].to_lowercase() == name.to_lowercase() {
						res[offset].push(vec!(get_category_alias(temp[y][0][1].clone())[0].clone(),temp[y][1][0].clone(),temp[y][1][1].clone(),calculate_points(result.clone(),get_category_alias(temp[y][0][1].clone())[0].clone(),last[1].clone(),last[0].clone())));
					} else
					if temp[y][1][8].to_lowercase() == name.to_lowercase() {
						res[offset].push(vec!(get_category_alias(temp[y][0][1].clone())[0].clone(),temp[y][1][0].clone(),temp[y][1][1].clone(),calculate_points(result.clone(),get_category_alias(temp[y][0][1].clone())[0].clone(),last[1].clone(),last[0].clone())));
					}
				}
			}
		}
	}
	}
	for x in 0..res.len(){
		res[x].insert(1,vec!("Discipline".to_string(),"Rank".to_string(),"Result".to_string(),"Iaaf Points".to_string()));
	}
	res
}
//https://github.com/GlaivePro/IaafPoints/blob/master/LICENSE.md
//uzimanje magicnih brojeva iz csv-a tako da se moze izracunati IAAF-ovi (WA) bodovi
fn get_magic_numbers(discip: String, age: String, year: String) -> Vec<String>{
	let contents = fs::read_to_string("magicnumbers.csv")
    .expect("Something went wrong reading the file");
	let mut array = Vec::new();
	let mut search = get_age_data(age,year);
	search = search + &discip;
	for x in contents.split("\n"){
		array.push(x.split(",").collect::<Vec<_>>());
	}
	let mut res = Vec::new();
	for x in array{
		if search.to_lowercase() == x[0].to_lowercase(){
			for y in x{
				res.push(y.to_string());
			}
			break;
		}
	}
	res
}
//izracunjavanje IAAF-ovih bodovav
fn calculate_points(result: f64, discip: String, age: String, year: String)-> String{
	let magic = get_magic_numbers(discip, age, year);
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

fn save_cache(year: String){
	for age in get_ages_net(year.clone()){
		fs::create_dir_all("/has/cache/".to_owned() + &year.clone());
		let url = "https://www.has.hr/images/stories/HAS/tabsez/".to_owned() + &year.clone() + "/" + &age.clone();
		let res = reqwest::blocking::get(&url.clone()).unwrap();
		let data = res.text_with_charset("windows-1250").unwrap();
		fs::write(&*("/has/cache/".to_owned() + &year.clone() + "/" + &age.clone().replace("html","htm").replace("htm","html")),data.clone()).expect("Unable to write file");
	}
}
fn format_result(result:f64) -> String{
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
fn init_cache(){
	let paths = fs::read_dir("/has/").unwrap();
	let b: bool = Path::new("/has/cache").is_dir();
	if b == false{
		let years = get_years_net();
		for year in years {
			save_cache(year);
		}
	}else{
		let years = get_years_net();
		if years[0] != "Failed"{	
			//save_cache(years[years.len()-1].clone());
		}
	}
}
fn get_carrer_profile(search: &str) ->Vec<Vec<Vec<String>>>{
	let m = MultiProgress::new();
	let mut profil:Vec<Vec<Vec<String>>> = Vec::new();
	let years = get_years();
	let barYear = ProgressBar::new(years.len() as u64);
	for year in years {
		for age in get_ages(year.clone()){
			

			profil.push(get_single_profile(modify_data(getDataFromCache(&*("/has/cache/".to_owned() + &year.clone() + "/" + &age.clone())),age.clone(),year.clone()), search.to_string()));

		}
		barYear.inc(1);		
	}
	barYear.finish();
	let mut x = 0;
	while true{
		if profil[x].len() == 0{
			profil.remove(x);
		}else{
			x = x+1;
		}
		if x == profil.len(){
			break;
		}
	}
	let mut discip:Vec<String> = Vec::new();
	for x in 0..profil.len(){
		if profil[x].len() > 1{
			for y in 1..profil[x].len(){
				if discip.len() == 0{
					discip.push(profil[x][y][0].clone());
				}else{
					let mut found = false;
					for d in discip.clone() {
						if d == profil[x][y][0]{
							found = true;
						}
					}
					if found == false{
						discip.push(profil[x][y][0].clone());
					}
				}
			}
		}
	}
	let mut res:Vec<Vec<String>> = Vec::new();
	let mut years = get_years();
	let mut lyear:String = "".to_string();
	let mut byear = false;
	let mut count = 0;
	let mut pb:Vec<f64> = Vec::new();
	let mut iaafpb:Vec<String> = Vec::new();
	let mut spb:Vec<String> = Vec::new();
	for d in discip.clone(){
		pb.push(0.00);
		spb.push("".to_string());
		iaafpb.push("".to_string());
	}
	for x in 0..profil.len(){
		if lyear ==  profil[x][0][0]{
			byear = true;
		}
		let mut row:Vec<String> = Vec::new();
		if profil[x].len() > 1{
			let mut pos = Vec::new();
			for y in 1..profil[x].len(){
				for d in 0..discip.len() {
					if discip[d] == profil[x][y][0]{
						let mut result:f64 = 0.0;
						let stemp = profil[x][y][2].replace(";",":");
						let minutes = stemp.split(":").collect::<Vec<_>>();
						if minutes.len() == 1{
							result = minutes[0].replace(",",".").parse::<f64>().unwrap();
						}else if minutes.len() == 2{
							result = minutes[0].parse::<f64>().unwrap()*60.0+minutes[1].replace(",",".").parse::<f64>().unwrap();
						}else{
							result = minutes[0].parse::<f64>().unwrap()*60.0*60.0+minutes[1].parse::<f64>().unwrap()*60.0+minutes[2].replace(",",".").parse::<f64>().unwrap();
						}
						if pb[d] != 0.00{
							if discip[d].find("m").unwrap_or(99) != 99 || discip[d] == "HM" || discip[d] == "Marathon"{
								if pb[d] > result{
									pb.remove(d);
									pb.insert(d,result);
									spb.remove(d);
									spb.insert(d, profil[x][y][2].clone());
									iaafpb.remove(d);
									iaafpb.insert(d, profil[x][y][3].clone());
								}
							}else{
								if pb[d] < result{
									pb.remove(d);
									pb.insert(d,result);
									spb.remove(d);
									spb.insert(d, profil[x][y][2].clone());
									iaafpb.remove(d);
									iaafpb.insert(d, profil[x][y][3].clone());
								}								
							}
						}else {
							pb.remove(d);
							pb.insert(d,result);
							spb.remove(d);
							spb.insert(d, profil[x][y][2].clone());
							iaafpb.remove(d);
							iaafpb.insert(d, profil[x][y][3].clone());
						}
						pos.push(vec!(profil[x][y][0].clone(),profil[x][y][2].clone()));
					}
				}
			}
			for d in 0..discip.len(){	
				let mut found = false;
				for i in pos.clone(){
					if i[0] == discip[d]{
						if byear == false{
							row.insert(d,i[1].clone());
						}else{
							res[count-1].remove(d+1);
							res[count-1].insert(d+1,i[1].clone());
						}
						found = true;
					}
				}		
				if found == false{
					if byear == false{
						row.insert(d,"x".to_string());
					}
				}
			}
		}
		if byear == false{
			row.insert(0,profil[x][0][0].clone());
			res.push(row);
			count = count +1;
		}
		lyear = profil[x][0][0].clone();
		byear = false;
	}
	discip.insert(0,profil[0][0][1].clone());
	spb.insert(0,"PB".to_string());
	res.push(spb);
	iaafpb.insert(0, "Iaaf Points".to_string());
	res.push(iaafpb);
	res.insert(0,discip);
	let mut data:Vec<Vec<Vec<String>>> = vec!(vec!(vec!("".to_string())));
	data.push(vec!(vec!("c".to_string())));
	data.push(transpose(res.clone()));
	data.remove(0);
	data
}
fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>>
where 
	T:Clone{
	assert!(!v.is_empty());
	(0..v[0].len())
	.map(|i| v.iter().map(|inner| inner[i].clone()).collect::<Vec<T>>())
	.collect()
}
fn main() {
	//println!("{:?}",get_ages("2013".to_string()));
	//displayTable(getDataFromWebsite("https://www.has.hr/images/stories/HAS/tabsez/2013/ssm13d.htm"));
	//println!("{:?}",get_single_profile(modify_data(getDataFromCache("/has/cache/2016/ssm16.html"),"ssm16.html".to_string(),"2016".to_string()),"Penezić".to_string()));
	//get_ages("2020".to_string());
	init_cache();
	
	let age:String;
	let category:String;
	let mut data:Vec<Vec<Vec<String>>> = vec!(vec!(vec!("".to_string())));
	let mut alias:Vec<Vec<String>> = vec![];
	let year = CLI_question("Choose a year",get_years(),true);
	if year != "all".to_string(){
		for x in get_ages(year.clone()){
			alias.push(get_age_alias(x, year.clone()));
		}
		age = CLI_question_alias("Choose a age",get_ages(year.clone()), alias);
		data = getDataFromCache(&*("/has/cache/".to_owned() + &year.clone() + "/" + &age.clone()));
		data = modify_data(data.clone(),age.clone(),year.clone());
		let ans = CLI_question("Do you want to see all categories", vec!("yes".to_string(),"no".to_string()),false);
		if ans == "no"{
			let mut alias:Vec<Vec<String>> = vec![];
			for x in get_categories(data.clone()){
				alias.push(get_category_alias(x));
			}
			category = CLI_question_alias("Choose a category", get_categories(data.clone()),alias);
			data = getDiscipline(data.clone(), &*category);
		}
		let ans = CLI_question("Do you want filter the results", vec!("yes".to_string(),"no".to_string()),false);
		if ans == "yes"{
			let key = CLI_input("Who do you want to search for");
			data = get_profile(data,key.to_string());
		}
		let ans = CLI_question("Do you want to save the result", vec!("yes".to_string(),"no".to_string()),false);
		if ans == "yes"{
			saveCSV(data.clone());
		}
		displayTable(data.clone());
	}else{
		let key = CLI_input("Who do you want to search for");
		data = get_carrer_profile(&key);
		let ans = CLI_question("Do you want to save the result", vec!("yes".to_string(),"no".to_string()),false);
		if ans == "yes"{
			saveCSV(data.clone());
		}
		displayTable(data);
	}
}
