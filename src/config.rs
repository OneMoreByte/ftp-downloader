use util;

struct Downloadable {
    client_loc: String,
    server_loc: String,
    namescheme: String,
}


pub struct Config {
    host: String,
    port: u16,
    user: String,
    pass: String,
    remote_downloadable: Vec<Downloadable>,
}

impl Config {
    /// Breaks down the config given a String of the config file's contents
    pub fn Config(buff: &mut String) {
        // Make sure we have everything we need
        if buff.contains("host:") && buff.contains("user:") && buff.contains("password:")
        {

            self.host = break_line(buff, "host:").unwrap();
            self.user = break_line(buff, "user:").unwrap();
            self.pass = break_line(buff, "password:").unwrap();
            self.remote_downloadable: break_remotefile(buff).unwrap(),

            if host.contains(":") {
                let sp: Vec<&str> = host.split(':').collect();
                host = sp[0].trim().to_string();
                port = sp[1].trim().parse().unwrap();

            } else {
                port = 21
            }

            Some(temp)
        } else {
            // If we would there's no point
            print!("Missing ");

            if !buff.contains("host:"){
                print!("\"host\" ");
            }
            if !buff.contains("user:") {
                print!("\"username\" ");
            }

            if !buff.contains("password"){
                print!("\"password\" ");
            }
            print!("\n");

            None
        }

    }

    /// Finds the line and get the data from it, if there isn't data to get, None is returned
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

    /// Breaks down the remoteFile: line specificly since it's kinda special in layout
    fn break_remotefile(input: &mut String) -> Option<Vec<Downloadable>> {

        // Trim string down to what we need
        let loc = input
            .find("remoteFiles:")
            .unwrap()
            .checked_add("remoteFiles:".len())
            .unwrap();
        let remfile = &input[loc..];
        let remfile = &remfile[..remfile.rfind(";").unwrap().checked_sub(";".len()).unwrap()];
        let remfile = &remfile[remfile.find("[").unwrap().checked_add("[".len()).unwrap()..];

        // Break it up
        let files: Vec<&str> = remfile.split(',').collect();
        let mut dlable_f: Vec<Downloadable> = Vec::new();

        // [LOG FUNCTION]
        println!(
            "There are {} entries from this config to download",
            files.len()
        );

        for file in files {

            if file.contains("remoteDir:") && file.contains("localDir:") && file.contains("name:") {
                let a: &mut String = &mut file.to_string();
                dlable_f.push(Downloadable {
                    client_dir: break_line(a, "localDir:").unwrap(),
                    server_dir: break_line(a, "remoteDir:").unwrap(),
                    filename: break_line(a, "name:").unwrap(),
                    clientfile_namescheme: break_line(a, "nameToSaveAs:").unwrap_or(
                        break_line(a, "name:")
                            .unwrap(),
                    ),
                });
            } else {
            }

        }

        if dlable_f.len() > 0 {
            Some(dlable_f)
        } else {
            // [LOG FUNCTION]
            println!(
                "No files found to download or check on in this config. Check your \
                  config."
            );
            None
        }



    }

}
