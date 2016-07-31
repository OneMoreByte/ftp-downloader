use std::fs;
use std::fs::OpenOptions;

/// Makes a directory specified
fn mk_dir(d: &str) -> std::io::Result<()> {
        try!(fs::create_dir_all(d));
        Ok(())
}

/// Loads all configs into the folder
fn load_configs() {

    mk_dir("./configs");

    let config_dir = fs::read_dir("./configs/").unwrap();
    let mut configs: u8 = 0;

    for f in config_dir{
        configs += 1;
        let d = f.file_name();
        println!("{:?}", d);
        //let conf = OpenOptions::new().read(true).write(true).open(f);
    }

    println!("There were {0} files in the config folder.", configs);
}

fn main(){
    load_configs();

}
