extern crate clap;
use git2::{Repository,Error,SubmoduleUpdateOptions};
use clap::{Arg, App};
use std;
use std::io;

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

fn init_submodules(repo : &Repository,submodule_name : &str){
    let mut submodule_options = SubmoduleUpdateOptions::new();
    let init = true;
    let opts = Some(&mut submodule_options);
    println!("Initlize {} submodules of repository {}",submodule_name,repo.path().display());
    if submodule_name == "all"{

        for mut submodule in repo.submodules().unwrap(){
            println!("Begin Initilize: {} submodule...",submodule_name);
            submodule.update(init, Some(&mut submodule_options)).unwrap();
            let submdoule_repo = submodule.open().unwrap();
            init_submodules(&submdoule_repo,"all");
            println!("Done with initlizing {} submodule...",submodule_name);
        }
    }else{
        if let Some(mut submodule) = repo.submodules().unwrap().into_iter().find(|entry| entry.name().unwrap() == submodule_name){
            println!("Begin Initilize: {} submodule...",submodule_name);
            submodule.update(init, opts).unwrap();
            let submdoule_repo = submodule.open().unwrap();
            init_submodules(&submdoule_repo,"all");
            println!("Done with initlizing {} submodule...",submodule_name);
        }
    }
    println!("Done with initlizing {} submodules of repository {}",submodule_name,repo.path().display());
}
fn update(repo : &Repository,submodule_name : &str){
    let list_of = repo.submodules().unwrap();
    if let Some(submodule) = list_of.into_iter().find(|entry| entry.name().unwrap() == submodule_name){
        let url = submodule.url().unwrap();
        remove(&repo,&submodule_name);
        add(&repo,url,Some(&submodule_name));
        println!("Submodule was updated!");
        }else{
            println!("Cannot update submodule because it does not exists!");
        }
}

fn remove_from_config(file : &str,submodule_name : &str){
    let mut config = git2::Config::open(std::path::Path::new(file)).unwrap();
    let values: Vec<String> = config
    .entries(None)
    .unwrap()
    .into_iter()
    .map(|entry|entry.unwrap().name().unwrap().into())
    .collect();

     for entry in  values{
        let mut find_str = String::from("submodule.");
        find_str += submodule_name;
        if entry.contains(&find_str){
            if config.remove(&entry).is_err(){
                println!("Cannot find {} entry in {}",entry,file);
            }
        }
    }   
}

fn remove(repo : &Repository,submodule_name : &str){
    let list_of = repo.submodules().unwrap();
    match list_of.into_iter().find(|entry| entry.name().unwrap() == submodule_name){
        Some(submodule) => {

            { // remove from .gitmodules
                remove_from_config(".gitmodules",submodule.name().unwrap());
            }

            { // remove from config
                remove_from_config(".git/config",submodule.name().unwrap());
            }

            { // remove from index
                let mut index = repo.index().unwrap();
                index.remove_dir(submodule.path(),0).unwrap();
                index.write().unwrap();
                index.write_tree().unwrap();
            }

            { // remove from modules folder
                let path = std::path::Path::new(".git/modules/").join(submodule.path());
                if std::fs::remove_dir_all(&path).is_err(){
                    println!("Error: could not remove {:?}", path.display());
                }
            }
            {// remove local files:
                let current_dir = std::env::current_dir().unwrap();
                let path = current_dir.join(submodule.path());
                if std::fs::remove_dir_all(&path).is_err(){
                    println!("Error: could not remove {:?}", path.display());
                }
            }
            println!("Submodule was removed!");
        }
        None => {}
    }
}

fn list(repo : &Repository){
   let list_of = repo.submodules().unwrap();
   println!("We found {} submodules.",list_of.len());
   let mut count = 0;
   for module in &list_of {
       count += 1;
       if module.url().is_some() {
        println!("#{}\nname: {}\nurl: {}\npath: {:?}\n",count,module.name().unwrap(),module.url().unwrap(),module.path().to_str());
       }else{
        println!("#{}\nname: {} is invalid",count,module.name().unwrap());
       }
   }
}

fn init() -> Result<Repository,Error>{
    let path = std::env::current_dir().unwrap();
    println!("{:?}",path);  
    Repository::discover(path.to_str().unwrap())
}


fn main() {
    if let Ok(repo) = init() {

    let matches = App::new("gsm")
                          .version("0.1")
                          .author("Simon Renger <simon.renger@gmail.com>")
                          .about("git submodules easily managed")
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
                    .arg(Arg::with_name("update")
                          .short('u')
                          .long("update")
                          .value_name("submdoule")
                          .help("updates submodule to latests")
                          .takes_value(true))
                    .arg(Arg::with_name("init")
                          .short('i')
                          .long("init")
                          .value_name("submdoule")
                          .help("init submodule or all if all is given")
                          .takes_value(true))
                    .arg(Arg::with_name("list")
                          .short('l')
                          .long("list")
                          .help("list submodule"))
                    .get_matches();
    if matches.is_present("add"){
        add(&repo,matches.value_of("add").unwrap(),matches.value_of("name"));
    }else if matches.is_present("list"){
        list(&repo);
    }else if matches.is_present("remove"){
        remove(&repo,matches.value_of("remove").unwrap());
    }else if matches.is_present("update"){
        update(&repo,matches.value_of("update").unwrap());
    }else if matches.is_present("init"){
        init_submodules(&repo,matches.value_of("init").unwrap());
    }
}else{
    println!("Could not find a git reposity");
}

}