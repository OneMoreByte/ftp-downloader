extern crate ftp;
extern crate regex;
extern crate openssl;

use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::fs::OpenOptions;
use std::io::BufWriter;
use ftp::FtpStream;
// use openssl::ssl::*;

struct DownloadableFile {
    client_dir: String,
    server_dir: String,
    filename: String,
    clientfile_namescheme: String,
}


struct DownRequest {
    host: String,
    user: String,
    pass: String,
    remote_files: Vec<DownloadableFile>,
}


/// Makes a directory specified
fn mk_dir(d: &str) -> std::io::Result<()> {
    try!(fs::create_dir_all(d));
    Ok(())
}

/// Breaks down the config given a String of the config file's contents
fn break_conf(buff: &mut String) -> Option<DownRequest> {

    /// Find line and get the data from it, if there isn't data to get, None is returned
    fn break_line(input: &mut String, line: &str) -> Option<String> {
        if input.contains(line) {
            let loc = input.find(line).unwrap().checked_add(line.len()).unwrap();
            let temp = &input[loc..];

            let loc = temp.find(";").unwrap();
            Some(temp[..loc].trim().to_string())
        } else {
            None
        }
    }
    // Make sure we won't get an unexpected None
    if buff.contains("host:") && buff.contains("user:") && buff.contains("password:") &&
       buff.contains("remoteFiles:") {

        /// Breaks down the remoteFile: line specificly since it's kinda special in layout
        fn break_remotefile(input: &mut String) -> Option<Vec<DownloadableFile>> {

            // Trim string down to what we need
            let loc =
                input.find("remoteFiles:").unwrap().checked_add("remoteFiles:".len()).unwrap();
            let remfile = &input[loc..];
            let remfile = &remfile[..remfile.rfind(";").unwrap().checked_sub(";".len()).unwrap()];
            let remfile = &remfile[remfile.find("[").unwrap().checked_add("[".len()).unwrap()..];

            // Break it up
            let files: Vec<&str> = remfile.split(',').collect();
            let mut dlable_f: Vec<DownloadableFile> = Vec::new();

            // [LOG FUNCTION]
            println!("There are {} entries from this config to download",
                     files.len());

            for file in files {

                if file.contains("remoteDir:") && file.contains("localDir:") &&
                   file.contains("name:") {
                    let a: &mut String = &mut file.to_string();
                    dlable_f.push(DownloadableFile {
                        client_dir: break_line(a, "localDir:").unwrap(),
                        server_dir: break_line(a, "remoteDir:").unwrap(),
                        filename: break_line(a, "name:").unwrap(),
                        clientfile_namescheme: break_line(a, "nameToSaveAs:")
                            .unwrap_or(break_line(a, "name:").unwrap()),
                    });
                } else {}

            }

            if dlable_f.len() > 0 {
                Some(dlable_f)
            } else {
                // [LOG FUNCTION]
                println!("No files found to download or check on in this config. Check your \
                          config.");
                None
            }



        }


        let temp = DownRequest {
            host: break_line(buff, "host:").unwrap(),
            user: break_line(buff, "user:").unwrap(),
            pass: break_line(buff, "password:").unwrap(),
            remote_files: break_remotefile(buff).unwrap(),
        };

        Some(temp)
    } else {
        // If we would there's no point
        None
    }

}


/// Loads all configs into the folder
fn load_configs() -> std::io::Result<Vec<DownRequest>> {
    let mut conf_d = env::current_exe().unwrap();
    conf_d.push("/config/");

    if mk_dir(&conf_d.to_str().unwrap()).is_ok() {
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

/// Downloads files from a DownRequest
fn download_from_site(c: DownRequest, dr: bool) -> std::io::Result<()> {
    // TODO ftps support
    let host: String;
    let port: u16;
    if c.host.contains(":") {
        let sp: Vec<&str> = c.host.split(':').collect();
        host = sp[0].trim().to_string();
        port = sp[1].trim().parse().unwrap();

    } else {
        host = c.host;
        port = 21
    }

    println!("Downloading files from \"{}:{}\"", host, port);


    let mut fstream = FtpStream::connect((host.as_str(), port)).unwrap();

    if fstream.login(&c.user.as_str(), &c.pass.as_str()).is_ok() {


        println!("Connected to server, and logged in successfully");

        for f in c.remote_files {
            /// prints the print_progress from a usize from 0 - 100
            fn print_progress(per_done: usize) {
                let outa = per_done.checked_mul(4).unwrap().checked_div(10).unwrap();
                let mut t = 0;
                let mut pbar: String = "[".to_string();
                while t < 40 {
                    if outa < t {
                        pbar.push_str(" ")
                    } else {
                        pbar.push_str("=")
                    }

                    t = t + 1;
                }
                pbar.push_str("]");

                print!("\r{} : {}%", pbar, per_done);
            }

            let mut to_download: Vec<String> = Vec::new();
            let mut cl_names: Vec<String>;
            let _cd = fstream.cwd(&f.server_dir.as_str());

            if f.filename.contains("(*)") {
                let all_in_dir: Vec<String> = fstream.nlst(None).unwrap();
                let spieces_to_check: Vec<&str> = f.filename.split("(*)").collect();
                let cpieces_to_check: Vec<&str> = f.clientfile_namescheme.split("(*)").collect();

                for sf in all_in_dir {

                    let mut check_match = true;

                    while check_match {
                        for p in &spieces_to_check {
                            if !sf.contains(p) {
                                check_match = false;
                            }
                        }
                        break;
                    }
                    if check_match {
                        to_download.push(sf);
                    }
                }

                if f.filename.contains(f.clientfile_namescheme.as_str()) &&
                   f.filename.len() == f.clientfile_namescheme.len() {
                    cl_names = to_download.clone();
                } else {
                    cl_names = Vec::new();

                    if spieces_to_check.len() == cpieces_to_check.len() {
                        for n in &to_download {

                            let mut inv_pieces: Vec<&str> = Vec::new();
                            let mut cfn: String = "".to_string();
                            inv_pieces.push(n.as_str());
                            for piece in &spieces_to_check {
                                let mut temp: Vec<&str> = Vec::new();
                                for a in inv_pieces {
                                    if a.contains(piece) {
                                        let t: Vec<&str> = a.split(piece).collect();
                                        for tt in t {
                                            temp.push(tt);
                                        }
                                    } else {
                                        temp.push(a);
                                    }
                                }
                                inv_pieces = temp;
                            }

                            for x in 0..inv_pieces.len() {
                                if x > 0 {
                                    cfn.push_str(cpieces_to_check[x - 1])
                                }
                                cfn.push_str(inv_pieces[x]);
                            }

                            cl_names.push(cfn);
                        }
                    } else {
                        println!("Can't download files. The client and server name must have the \
                                  same number of (*)!");
                    }
                }

                println!("{} files matching \"{}\"", &to_download.len(), &f.filename);

            } else {
                cl_names = Vec::new();
                cl_names.push(f.clientfile_namescheme);
                to_download.push(f.filename);
            }


            for fz in 0..cl_names.len() {
                let dir = &f.client_dir.as_str().clone();
                let _md = mk_dir(dir); // Make Client dir

                /// Checks to see if the file is downloaded
                /// Takes a local file with it's path and the remote filee's byte size in usize
                fn is_downloaded(fileandpath: &String, rsz: &usize) -> Option<bool> {
                    let mdata = fs::metadata(fileandpath);
                    if mdata.is_ok() {

                        let lsz = mdata.unwrap().len() as usize;
                        if &lsz == rsz { Some(true) } else { Some(false) }
                    } else {
                        None
                    }
                }
                if dr == false {
                    let rsize = fstream.size(&to_download[fz].as_str()).unwrap();

                    let mut fnm = "".to_string();
                    fnm.push_str(dir);
                    fnm.push_str("/");
                    fnm.push_str(&cl_names[fz].as_str());

                    println!("Downloading \"{}\"", to_download[fz]);

                    if !is_downloaded(&fnm, &rsize.unwrap()).unwrap_or(false) {

                        // Make local file
                        let _tt = fstream.transfer_type(::ftp::types::FileType::Binary);



                        let _result = fstream.retr(&to_download[fz].as_str(), |stream| {
                            let mut buf = [0; 4096];
                            let mut flnm = "".to_string();
                            flnm.push_str(dir);
                            flnm.push_str("/");
                            flnm.push_str(&cl_names[fz].as_str());

                            let mut file = File::create(flnm).unwrap();
                            let mut lsize: usize = 0;

                            loop {
                                match stream.read(&mut buf) {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        file.write_all(&buf[0..n]).unwrap();
                                        lsize += n;
                                    }
                                    Err(_err) => break,
                                };
                                print_progress(((lsize.checked_mul(100).unwrap())
                                    .checked_div(rsize.unwrap())
                                    .unwrap()));
                            }
                            print!("\r\n");
                            Ok(())
                        });
                    } else {
                        println!("File is already downloaded");
                    }
                } else {
                    println!("Would be downloading {} and saving as {}",
                             &to_download[fz],
                             &cl_names[fz]);
                }
            }
        }

        let _f = fstream.quit();

        Ok(())
    } else {
        let _f = fstream.quit();
        println!("User login credentals were incorrect");
        Ok(())
    }
}

// Adds a file to downlaod to a config
fn add_to_file(mut file: Vec<String>) -> std::io::Result<()> {
    // TODO Catch errors and check inputs
    if file.len() == 4 {
        let g = file[3].clone();
        file.push(g);
    }
    let mut name = env::current_exe().unwrap();
    println!("{:?}", name);
    let t = file[0].clone();
    name.push(t.as_str());
    let mut b = "".to_string();
    let mut _a = File::open(&name).unwrap();
    let __ = _a.read_to_string(&mut b);
    let mut _f = OpenOptions::new().write(true).truncate(true).open(name).unwrap();
    let mut _w = BufWriter::new(_f);
    let _s1: &str;
    let _s2: &str;

    let (_s1, _s2) = b.split_at(b.rfind("}").unwrap().checked_add("}".len()).unwrap());

    println!("Writing to config");

    let _ =
        _w.write_all(format!("{},\r\n{{\r\n  remoteDir: {};\r\n  localDir: {};\r\n  name: \
                              {};\r\n  nameToSaveAs: {};\r\n}}{}",
                             _s1,
                             file[1],
                             file[2],
                             file[3],
                             file[4],
                             _s2)
            .as_bytes());
    Ok(())
}


fn main() {

    let mut _should_ls = false;
    let mut dry_run = false;
    let mut should_add = false;
    let mut add_to_config: Vec<String> = Vec::new();
    let ag: Vec<String> = env::args().collect();

    for x in 0..ag.len() {
        if ag[x].contains("-dr") {
            dry_run = true;
        }
        if ag[x].contains("-af") {
            if (x + 4) < ag.len() {
                should_add = true;
                add_to_config.push(ag[x + 1].clone());
                add_to_config.push(ag[x + 2].clone());
                add_to_config.push(ag[x + 3].clone());
                add_to_config.push(ag[x + 4].clone());
                if x + 5 < ag.len() {
                    add_to_config.push(ag[x + 5].clone());
                }
            } else {
                println!("Config name, client and server directory, and server filename must be \
                          supplied");
            }
        }
    }

    if should_add {
        let _ = add_to_file(add_to_config);
    }

    // TODO ls command
    println!("\rLoading configs...");

    let configs = load_configs().unwrap();

    for con in configs {
        let _d = download_from_site(con, dry_run);
    }
    std::process::exit(0);

}
