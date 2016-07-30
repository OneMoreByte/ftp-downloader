
use std::fs;

fn main(){

    let config_dir = fs::read_dir("./configs/").unwrap();
    let configs: u8 = 0;

    for f in config_dir{
        configs = 1 + configs;
    }

    println!("There are {0} in the config folder.", configs);

}
