
use std::fs;



fn load_configs()
{

    let config_dir = fs::read_dir("./configs/").unwrap();
    {fs::create_dir("./configs/"); fs::read_dir("./configs");

    let mut configs: u8 = 0;

    for f in config_dir{
        configs = 1 + configs;
    }

    println!("There are {0} in the config folder.", configs);

}



fn main(){

    load_configs();

}
