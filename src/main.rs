extern crate ftp;
extern crate regex;
extern crate openssl;

use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
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
fn break_conf(b: &mut String) -> Option<DownRequest> {

    /// Find line and get the data from it, if there isn't data to get, None is returned
    fn break_line(input: &mut String, line: &str) -> Option<String> {
        if input.contains(line) {
            let loca = input.find(line).unwrap().checked_add(line.len()).unwrap();
            let y = &input[loca..];

            let locb = y.find(";").unwrap();
            Some(y[..locb].trim().to_string())
        } else {
            None
        }
    }
    // Make sure we won't get an unexpected None
    if b.contains("host:") && b.contains("user:") && b.contains("password:") &&
       b.contains("remoteFiles:") {

        /// Breaks down the remoteFile: line specificly since it's kinda special in layout
        fn break_remotefile(input: &mut String) -> Option<Vec<DownloadableFile>> {


            let start =
                input.find("remoteFiles:").unwrap().checked_add("remoteFiles:".len()).unwrap();
            let inp = &input[start..];
            let tinp = &inp[..inp.rfind(";").unwrap().checked_sub(";".len()).unwrap()];
            // let ttinp = &tinp[..tinp.rfind("]").unwrap().checked_sub("]".len()).unwrap()];

            let why = &tinp[tinp.find("[").unwrap().checked_add("[".len()).unwrap()..];
            let files: Vec<&str> = why.split(',').collect();
            let mut f: Vec<DownloadableFile> = Vec::new();

            println!("There are {} entries from this config to download",
                     files.len());

            for file in files {

                if file.contains("remoteDir:") && file.contains("localDir:") &&
                   file.contains("name:") {
                    let a: &mut String = &mut file.to_string();
                    f.push(DownloadableFile {
                        client_dir: break_line(a, "localDir:").unwrap(),
                        server_dir: break_line(a, "remoteDir:").unwrap(),
                        filename: break_line(a, "name:").unwrap(),
                        clientfile_namescheme: break_line(a, "nameToSaveAs:")
                            .unwrap_or(break_line(a, "name:").unwrap()),
                    });
                } else {}

            }

            if f.len() > 0 {
                Some(f)
            } else {
                println!("No files found to download or check on in this config. Check your \
                          config.");
                None
            }



        }


        let temp = DownRequest {
            host: break_line(b, "host:").unwrap(),
            user: break_line(b, "user:").unwrap(),
            pass: break_line(b, "password:").unwrap(),
            remote_files: break_remotefile(b).unwrap(),
        };

        Some(temp)
    } else {
        // If we would there's no point
        None
    }

}


/// Loads all configs into the folder
fn load_configs() -> std::io::Result<Vec<DownRequest>> {

    if mk_dir("./configs").is_ok() {
        println!("Made the dir needed");
    } // make directory

    let config_dir = fs::read_dir("./configs/").unwrap(); //read all files in dir, and expand them
    let mut but_two_m8 = fs::read_dir("./configs/").unwrap();
    let mut numb_in_folder: u8 = 0;
    let mut configs = Vec::new();

    for f in config_dir {
        // for all files f in the directory open them to file, and save the contained string

        numb_in_folder += 1;
        let name = but_two_m8.next();
        let mut file = try!(File::open(try!(f).path()));
        let mut buff = String::new();

        try!(file.read_to_string(&mut buff));
        let temcon = break_conf(&mut buff); // send buff off to be broken down

        if temcon.is_some() {
            // Check to make sure we didn't get nothing.
            configs.push(temcon.unwrap());
            println!("Config \"{}\" loaded successfully!",
                     name.unwrap().unwrap().file_name().to_str().unwrap());
        } else {
            println!("Config \"{}\" couldn't be loaded.",
                     name.unwrap().unwrap().file_name().to_str().unwrap());
        }

    }
    println!("There were {} configs to load!", numb_in_folder);
    Ok(configs)
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


fn main() {

    let mut _should_ls = false;
    let mut dry_run = false;

    for ar in env::args() {
        if ar.contains("-dr") {
            dry_run = true;
        }
        println!("Arg {}", ar);
    }

    // TODO ls command

    println!("\r Loading configs...");

    let configs = load_configs().unwrap();

    for con in configs {
        let _d = download_from_site(con, dry_run);
    }
}
