extern crate ftp;
extern crate regex;

use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::fs::OpenOptions;
use std::io::BufWriter;
use ftp::FtpStream;

mod config;
mod util;

/// Downloads files from a DownRequest
fn download_from_site(c: config::Config, dr: bool) -> std::io::Result<()> {
    // TODO ftps support


    println!("Downloading files from \"{}:{}\"", c.host, c.port);


    let mut fstream = FtpStream::connect((c.host.as_str(), c.port)).unwrap();

    if fstream.login(&c.user.as_str(), &c.pass.as_str()).is_ok() {


        println!("Connected to server, and logged in successfully");

        for d in c.remote_downloadable {
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
                    f.filename.len() == f.clientfile_namescheme.len()
                {
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
                        println!(
                            "Can't download files. The client and server name must have the \
                                  same number of (*)!"
                        );
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
                                print_progress(
                                    ((lsize.checked_mul(100).unwrap())
                                         .checked_div(rsize.unwrap())
                                         .unwrap()),
                                );
                            }
                            print!("\r\n");
                            Ok(())
                        });
                    } else {
                        println!("File is already downloaded");
                    }
                } else {
                    println!(
                        "Would be downloading {} and saving as {}",
                        &to_download[fz],
                        &cl_names[fz]
                    );
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
    let mut conf_d = env::home_dir().unwrap().to_str().unwrap().to_string();
    conf_d.push_str("/.ftpdown/");
    let t = file[0].clone();
    conf_d.push_str(t.as_str());
    let mut b = "".to_string();
    let mut _a = File::open(&conf_d).unwrap();
    let __ = _a.read_to_string(&mut b);
    let mut _f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(conf_d)
        .unwrap();
    let mut _w = BufWriter::new(_f);
    let _s1: &str;
    let _s2: &str;

    let (_s1, _s2) = b.split_at(b.rfind("}").unwrap().checked_add("}".len()).unwrap());

    println!("Writing to config");

    let _ = _w.write_all(
        format!(
            "{},\r\n{{\r\n  remoteDir: {};\r\n  localDir: {};\r\n  name: \
                              {};\r\n  nameToSaveAs: {};\r\n}}{}",
            _s1,
            file[1],
            file[2],
            file[3],
            file[4],
            _s2
        ).as_bytes(),
    );
    Ok(())
}


fn main() {

    let mut _should_ls = false;
    let mut dry_run = false;
    let mut should_add = false;
    let mut add_to_config: Vec<String> = Vec::new();
    let ag: Vec<String> = env::args().collect();

    // TODO add App arguments instead
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
                println!(
                    "Config name, client and server directory, and server filename must be \
                          supplied"
                );
            }
        }
    }

    if should_add {
        let _ = add_to_file(add_to_config);
    }

    // TODO ls command
    println!("\rLoading configs...");

    let configs = util::load_configs().unwrap();

    for con in configs {
        let _d = download_from_site(con, dry_run);
    }
    std::process::exit(0);

}
