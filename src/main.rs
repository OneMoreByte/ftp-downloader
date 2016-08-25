#[macro_use]
extern crate ftp;
extern crate regex;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use ftp::{FtpStream, FtpError, types};


struct DownloadableFile {
    client_dir: String,
    server_dir: String,
    filename: String,
    clientfile_namescheme: String
}


struct DownRequest {
    host: String,
    is_ftps: bool,
    user: String,
    pass: String,
    remote_files: Vec<DownloadableFile>,
    schedule: [u8; 4]
}



/// Makes a directory specified
fn mk_dir(d: &str) -> std::io::Result<()> {
        try!(fs::create_dir_all(d));
        Ok(())
}

fn break_conf(b: &mut String) -> Option<DownRequest>{

    /// Find line and get the data from it, if there isn't data to get, None is returned
    fn break_line(input: &mut String, line: &str) -> Option<String>{
        if input.contains(line)
        {
            let loca =  input.find(line).unwrap().checked_add(line.len()).unwrap();
            let y = &input[loca..];
            let locb = y.find("\r\n").unwrap();
            Some(y[..locb].trim().to_string())
        }
        else
        {
            None
        }
    }

    if b.contains("host:") && b.contains("user:") && b.contains("password:") && b.contains("remoteFiles:"){ // Make sure we won't get an unexpected None

        let ftps : bool = break_line(b, "isFTPS:").unwrap_or("false".to_string()).parse().unwrap();

        fn break_remotefile(input: &mut String) -> Option<Vec<DownloadableFile>>{

            let start = input.find("remoteFiles:").unwrap().checked_add("remoteFiles:".len()).unwrap();
            let inp = &input[start..];
            let tinp = &inp[..inp.find("]").unwrap().checked_sub("]".len()).unwrap()];
            let why = &tinp[tinp.find("[").unwrap().checked_add("[".len()).unwrap()..];
            let files: Vec<&str> = why.split(',').collect();
            let mut f: Vec<DownloadableFile> = Vec::new();

            for file in files {

                if file.contains("remoteDir:") && file.contains("localDir:") && file.contains("name:")
                {
                    let a: &mut String = &mut file.to_string();
                    f.push(DownloadableFile {   client_dir: break_line(a, "localDir:").unwrap(), server_dir: break_line(a, "remoteDir:").unwrap(), filename: break_line(a, "name:").unwrap(), clientfile_namescheme: break_line(a, "nameToSaveAs:").unwrap_or(break_line(a, "name:").unwrap())});
                }
                else {}

            }

            if f.len() > 0{
                Some(f)
            }
            else {
                println!("No files found to download or check on in this config. Check your config.");
                None
            }



        }


        let temp = DownRequest {host: break_line(b, "host:").unwrap(), is_ftps: ftps, user: break_line(b, "user:").unwrap(), pass: break_line(b, "password:").unwrap(), remote_files: break_remotefile(b).unwrap(), schedule: [1, 1, 1, 1] };

        Some(temp)
    }
    else { // If we would there's no point
        None
    }

}

/// Loads all configs into the folder
fn load_configs() -> std::io::Result<Vec<DownRequest>> {

    mk_dir("./configs"); // make directory

    let config_dir = fs::read_dir("./configs/").unwrap(); //read all files in dir, and expand them out so we can read them
    let mut but_two_m8 = fs::read_dir("./configs/").unwrap();
    let mut numb_in_folder: u8 = 0;
    let mut configs = Vec::new();

    for f in config_dir{ // for all files f in the directory open them to file, and save the contained string to buff.


        numb_in_folder += 1;
        let name = but_two_m8.next();
        let mut file = try!(File::open(try!(f).path()));
        let mut buff = String::new();

        try!( file.read_to_string(&mut buff));
        let temcon = break_conf(&mut buff); // send buff off to be broken down tnto what we actully need

        if temcon.is_some(){ // Check to make sure we didn't get nothing.
            configs.push(temcon.unwrap());
            println!("Config \"{}\" loaded successfully!", name.unwrap().unwrap().file_name().to_str().unwrap());
        }else{
            println!("Config \"{}\" couldn't be loaded. Make sure it has all necessary fields", name.unwrap().unwrap().file_name().to_str().unwrap());
        }

    }
    println!("There were {} files in the config folder.", numb_in_folder);
    Ok(configs)
}

fn download_from_site(c: DownRequest) -> std::io::Result<()>{
    let host: String;
    let port: u16;
    if c.host.contains(":") {
        let sp: Vec<&str> = c.host.split(':').collect();
        host = sp[0].trim().to_string();
        port = sp[1].trim().parse().unwrap();

    }
    else {
        host = c.host;
        port = 21
    }
    println!("Downloading files from \"{}:{}\"", host, port);


    let mut fstream = FtpStream::connect((host.as_str(), port)).unwrap();

    if fstream.login(&c.user.as_str(), &c.pass.as_str()).is_ok(){


        println!("Connected to server, and logged in successfully");

        for f in c.remote_files{

            println!("\"{}\" \"{}\"", &f.client_dir.as_str(), f.clientfile_namescheme);
            mk_dir(&f.client_dir.as_str()); // Make Client dir
            let mut flnm = "".to_string(

            );
            flnm.push_str(&f.client_dir.as_str());
            flnm.push_str("/");
            flnm.push_str(&f.clientfile_namescheme.as_str());
            let mut file = File::create(flnm).unwrap(); // Make local file //TODO fix here

            fstream.cwd(&f.server_dir.as_str()); // Switch to remote dir
            fstream.transfer_type(::ftp::types::FileType::Binary);
            let cursor = fstream.simple_retr(&f.filename.as_str()).unwrap();
            let vec = cursor.into_inner();
            file.write_all(&vec).unwrap();

        }

        fstream.quit();

        Ok(())
    }
    else{
        fstream.quit();
        println!("User login credentals were incorrect");
        Ok(())
    }
}

fn main(){
    println!("\r Loading configs...");

    let configs = load_configs().unwrap();

    for con in configs {
        download_from_site(con);
    }


}
