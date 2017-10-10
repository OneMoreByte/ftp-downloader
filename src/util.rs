use config;
use std::fs;
use env;
use File;
use std::io::prelude::*;
use std::io;


/// Loads all configs into the folder
pub fn load_configs() -> Result<Vec<config::Config>, &'static str> {

    let mut config_dir = env::home_dir().unwrap().to_str().unwrap().to_string();
    config_dir.push_str("/.ftpdown/");

    // ?
    println!("{:?}", config_dir);

    if mk_dir(&config_dir.as_str()).is_ok() {
        println!("Made the dir needed");
    } // make directory

    let config_dir = fs::read_dir(config_dir).unwrap(); //read all files in dir, and expand them
    let mut configs: Vec<config::Config> = Vec::new();




    for file in config_dir {
        // for all files in the directory open them to the f var., and save the contained string
        let temp = file.unwrap();
        let mut config_raw = File::open(temp.path()).unwrap();
        let mut buff = String::new();

        config_raw.read_to_string(&mut buff);
        let mut con: Option<config::Config> = config::Config::new(&mut buff);


        if con.is_some() {

            // Check the success
            configs.push(con.unwrap());
            println!(
                "Config \"{}\" loaded successfully!",
                temp.file_name().to_str().unwrap()
            );
        } else {
            println!(
                "Config \"{}\" couldn't be loaded.",
                temp.file_name().to_str().unwrap()
            );
        }

    }
    println!("There were {} configs to load!", configs.len());
    Ok(configs)
}

/// Makes a directory specified
pub fn mk_dir(d: &str) -> io::Result<()>{
    fs::create_dir_all(d)?;
    Ok(())
}
