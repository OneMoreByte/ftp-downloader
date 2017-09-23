use config;

/// Loads all configs into the folder
pub fn load_configs() -> std::io::Result<Vec<config::Confg>> {

    let mut config_dir = env::home_dir().unwrap().to_str().unwrap().to_string();
    config_dir.push_str("/.ftpdown/");

    // ?
    println!("{:?}", conf_d);

    if mk_dir(&conf_d.as_str()).is_ok() {
        println!("Made the dir needed");
    } // make directory

    let config_dir = fs::read_dir(conf_d).unwrap(); //read all files in dir, and expand them
    let mut configs: Vec<config::Config> = Vec::new();



    for file in config_dir {
        // for all files in the directory open them to the f var., and save the contained string
        temp = file.unwrap();
        let mut config_raw = try!(File::open(temp.path()));
        let mut buff = String::new();

        try!(config_raw.read_to_string(&mut buff));
        let config = Config(&mut buff); // send buff off to be broken down

        if config.is_some() {
            // Check the success
            configs.push(config);
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
fn mk_dir(d: &str) -> std::io::Result<()> {
    try!(fs::create_dir_all(d));
    Ok(())
}
