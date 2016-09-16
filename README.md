# ftpdown - The FTP client no one needed!
A FTP client written in rust for commandline.

Downloads ftp files from a server automagicly using config files

*An example of which is below*
```
host: 192.168.1.48:1337; // Port is optional if using defualt ftp port
user: user;
password: apassword;
remoteFiles:
[{
  remoteDir: /var/wow/;
  localDir: ./;
  name: FilenameOnTheServer.mkv;
  nameToSaveAs: Localfsname.mkv; //Output Names can be changed with the optional namesToSaveAs:
},
{
  remoteDir: /var/www/;
  localDir: ./webBackup/;
  name: page(*).css; // (*) can be used to match anything with the strings alongside it.
}];
```
Config files can be named whatever. Or nothing. No filetype works too.
> ## Also! Arguments

You can pass it two arguments right now **-dr** and **-af**

* **-dr** will run the program in dry run mode. No files will be downloaded
* **-af** adds a file to a config used like so ``` ftpdown -af <NameOfConfigFile> <remoteDir> <localDir> <name> [Optional] <nametosaveas>```

Both args can be used at the same time (if you wanted to) just make sure you use -dr first!
