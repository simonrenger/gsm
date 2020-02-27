extern crate clap;
use git2::{Repository,Error};
use clap::{Arg, App};
use std;
use std::io;
use std::fs::File;
use std::io::{BufRead};
use std::path::Path;
use std::fs::OpenOptions;
use std::io::prelude::*;


fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn get_path(submodule : &str ,path : Option<&str>) ->  io::Result<std::path::PathBuf>{
    match path{
    None =>{
    let mut path = std::env::current_dir().unwrap();
    // create path:
    let result = submodule.split("/").last().unwrap().replace(".git","");
    path.push(result);
    Ok(path)
    },
    path=>{
        let mut submodule_path = std::path::PathBuf::new();
        submodule_path.push(std::env::current_dir().unwrap());
        submodule_path.push(path.unwrap());
        Ok(submodule_path)
    }
}
}

fn can_add_submodule(repo :&Repository,submodule : &str) -> bool{
    let list_of = repo.submodules().unwrap();
    match list_of.into_iter().find(|entry|{
        if entry.url().is_some(){
            entry.url().unwrap() == submodule
        }else{
            false
        }
    }){
        Some(_) => false,
        None => true
    }
}

fn add(repo : &Repository,submodule_url : &str,path : Option<&str>){
    if can_add_submodule(repo,submodule_url){
        let submodule_path = get_path(submodule_url,path);
        match repo.submodule(submodule_url,submodule_path.unwrap().as_path(),true){
            Err(e)=> println!("Error: could not add submodule because {}",e.message()),
            Ok(mut module) =>{
                std::fs::remove_dir_all(module.path()).unwrap();
                println!("Cloning submodule ...");
                Repository::clone(& module.url().unwrap(), module.path()).unwrap();
                module.add_to_index(true).unwrap();
                module.add_finalize().unwrap();
                println!("Added {} to repositry to location {}",submodule_url,module.path().display());
            }
        }
    }else{
        println!("Repositry: {} is already added as submodule",submodule_url);
    }
}

fn remove(repo : &Repository,submodule_name : &str){
    let list_of = repo.submodules().unwrap();
    match list_of.into_iter().find(|entry| entry.name().unwrap() == submodule_name){
        Some(submodule) => {
                let path = repo.path().join("..").join(".gitmodules");
                let path_str = String::from(path.to_str().unwrap());
                println!("{:?}",path);
                if let Ok(lines) = read_lines(path) {
                    let mut count = 0;
                    let mut content: Vec<String>  = Vec::new();
                    for line in lines {
                        if let Ok(ip) = line {
                            let mut pattern = "[submodule \"".to_string();
                            pattern += submodule.name().unwrap();
                            pattern += &"\"]".to_string();
                            if ip.contains(&pattern){
                                count += 1;
                            }else if count == 0{
                                content.push(String::from(&ip));
                            }else if count >= 2{
                                count = 0;
                            }else{
                                count += 1;
                            }
                        }
                    }
                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path_str)
                    .unwrap();
            
            for entry in content{
                if let Err(e) = writeln!(file,"{}",entry) {
                        eprintln!("Couldn't write to file: {}", e);
                    }
                }
            }
           

            { // remove from config
                let mut config = git2::Config::open(std::path::Path::new(".git/config")).unwrap();
                let mut string = String::from("submodule.");
                string += submodule.name().unwrap();
                string += &String::from(".url");
                config.remove(&string).unwrap();
            }

            { // remove from index
                let mut index = repo.index().unwrap();
                index.remove_all(submodule.path(),None).unwrap();
                index.write().unwrap();
            }

            { // remove from modules folder
                let path = std::path::Path::new(".git/modules/").join(submodule.path());
                std::fs::remove_dir_all(path).unwrap();
            }
            {// remove local files:
                let current_dir = std::env::current_dir().unwrap();
                let path = current_dir.join(submodule.path());
                std::fs::remove_dir_all(path).unwrap();
            }
        }
        None => {
            println!("Submodule {} has not been added so we cannot remove it",submodule_name);
        }
    }
}

fn list(repo : &Repository){
   let list_of = repo.submodules().unwrap();
   println!("We found {} submodules.",list_of.len());
   let mut count = 0;
   for module in &list_of {
       count += 1;
       println!("#{}\nname: {}\nurl: {}\npath: {:?}\n",count,module.name().unwrap(),module.url().unwrap(),module.path().to_str());
   }
}

fn init() -> Result<Repository,Error>{
    let path = std::env::current_dir().unwrap();
    println!("{:?}",path);  
    Repository::discover(path.to_str().unwrap())
}


fn main() {
    if let Ok(repo) = init() {
    let matches = App::new("gsme")
                          .version("0.1")
                          .author("Simon Renger <simon.renger@gmail.com>")
                          .about("Submodules easy managed")
                    .arg(Arg::with_name("add")
                          .short('a')
                          .long("add")
                          .value_name("submdoule")
                          .help("Adds a submodule")
                          .takes_value(true))
                    .arg(Arg::with_name("name")
                          .short('n')
                          .long("name")
                          .help("names a submodule")
                          .requires("add")
                          .takes_value(true))
                    .arg(Arg::with_name("remove")
                          .short('r')
                          .long("remove")
                          .value_name("submdoule")
                          .help("removes a submodule")
                          .takes_value(true))
                    .arg(Arg::with_name("list")
                          .short('l')
                          .long("list")
                          .help("list submdoules"))
                    .get_matches();
    if matches.is_present("add"){
        add(&repo,matches.value_of("add").unwrap(),matches.value_of("name"));
    }else if matches.is_present("list"){
        list(&repo);
    }else if matches.is_present("remove"){
        remove(&repo,matches.value_of("remove").unwrap());
    }
}else{
    println!("Could not find a git reposity");
}

}