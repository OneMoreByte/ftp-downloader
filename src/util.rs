/// Loads all configs into the folder
fn load_configs() -> std::io::Result<Vec<DownRequest>> {
    let mut conf_d = env::home_dir().unwrap().to_str().unwrap().to_string();
    conf_d.push_str("/.ftpdown/");
    println!("{:?}", conf_d);
    if mk_dir(&conf_d.as_str()).is_ok() {
        println!("Made the dir needed");
    } // make directory

    let config_dir = fs::read_dir(conf_d).unwrap(); //read all files in dir, and expand them
    let mut config_files: Vec<std::fs::DirEntry> = Vec::new();

    for file in config_dir {
        config_files.push(file.unwrap());
    }

    let mut configs_fmted = Vec::new();

    for file in &config_files {
        // for all files in the directory open them to the f var., and save the contained string

        let mut f = try!(File::open(file.path()));
        let mut buff = String::new();

        try!(f.read_to_string(&mut buff));
        let temp_config = break_conf(&mut buff); // send buff off to be broken down

        if temp_config.is_some() {
            // Check to make sure we didn't get nothing.
            configs_fmted.push(temp_config.unwrap());
            println!("Config \"{}\" loaded successfully!",
                     &file.file_name().to_str().unwrap());
        } else {
            println!("Config \"{}\" couldn't be loaded.",
                     &file.file_name().to_str().unwrap());
        }

    }
    println!("There were {} configs to load!", config_files.len());
    Ok(configs_fmted)
}

/// Makes a directory specified
fn mk_dir(d: &str) -> std::io::Result<()> {
    try!(fs::create_dir_all(d));
    Ok(())
}
