use shim::io;
use shim::path::{Path, PathBuf, Component};


use stack_vec::StackVec;
use alloc::vec::Vec;
use alloc::string::String;
use pi::atags::Atags;

use fat32::traits::FileSystem;
use fat32::traits::{Dir, Entry as EntryTrait, Metadata, Timestamp};
use fat32::vfat::Entry;

use crate::console::{kprint, kprintln, CONSOLE};
use crate::fs::PiVFatHandle;
use crate::ALLOCATOR;
use crate::FILESYSTEM;

use shim::io::Read;
use core::time::Duration;
use pi::timer::spin_sleep;
use core::str::from_utf8;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs,
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>,
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }
}




/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns.
pub fn shell(prefix: &str) -> ! {

    let mut cwd = PathBuf::new();
    cwd.push("/");


    spin_sleep(Duration::new(5, 0));
    
    kprintln!("a VERY warm welcome to ...");
    kprintln!("~~~~~~~~~~~~~~ JOS ~~~~~~~~~~~~~~");
    loop {
        let cwd_string =  cwd.clone().into_os_string().into_string().unwrap();
        kprint!("jakob@cs3210:[{}]{}", cwd_string, prefix);
        
        let a = &mut [0u8; 512];
        let mut input_buf = StackVec::new(a);  //should I be handling if the prefix is longer than one character?
        loop {
            let mut consoley = CONSOLE.lock();
            let current_byte = consoley.read_byte();
            if current_byte == b'\n' || current_byte == b'\r' {
                match from_utf8(input_buf.as_slice()) {
                    Ok(res) => {
                        kprintln!();
                        let arg_buf = &mut [""; 64]; //slice that the arguments will be put into
                        match Command::parse(res, arg_buf) {
                            Ok(command) => {
                                invoke_appropriate_command(command, &mut cwd);
                            },
                            Err(Error::Empty) => {
                                kprintln!();
                            },
                            Err(Error::TooManyArgs) => {
                                kprintln!("error: too many arguments");
                            }
                        };
                    },
                    Err(e) => kprintln!("Failed converting from str to [u8]"),
                }
                break;
            } else {
                if !input_buf.is_full() {
                    if is_visible_ascii(current_byte) {
                        if is_erase_ascii(current_byte) {
                            if input_buf.len() > 0 {
                                consoley.write_byte(8);
                                consoley.write_byte(b' ');
                                consoley.write_byte(8);
                                input_buf.pop();
                            }
                        } else {
                            input_buf.push(current_byte);
                            consoley.write_byte(current_byte);
                        }
                    }
                } else {
                    if is_erase_ascii(current_byte) {
                        if input_buf.len() > 0 {
                            consoley.write_byte(8);
                            consoley.write_byte(b' ');
                            consoley.write_byte(8);
                            input_buf.pop();
                        }
                    } else {
                        consoley.write_byte(7u8);
                    }
                }
            }
        }
    }
}

//helper functions for my shell() function


fn invoke_appropriate_command(command: Command, cwd: &mut PathBuf) {
    let command_path = command.path();
    let command_args = & command.args.as_slice()[1..];
    match command_path {
        "echo" => echo(command_args),
        "ls" => ls(command_args, cwd),
        "cat" => cat(command_args, cwd),
        "pwd" => pwd(cwd),
        "cd" => cd(command_args, cwd),
        _ => kprintln!("unknown command: ${}", command_path)
    }
}


//my aresenal of commands lives down here
fn echo(args: & [&str]) {
    for s in args {
        kprint!("{} ", s);
    }
    kprintln!();
}
fn pwd(cwd: &mut PathBuf) {
    let path_str = cwd.clone().into_os_string().into_string().unwrap();
    kprintln!("{}", path_str);
}
fn cat(args: & [&str], cwd: &mut PathBuf) {
    let mut count = 0;
    if args.len() == 0 { kprintln!("Please enter a file name!"); }
    for file in args {
        let path = construct_path(file, cwd);
        match FILESYSTEM.open(path.clone()) {
            Ok(entry) => {
                if entry.is_dir() { 
                    kprintln!("cat: {}: No such file or directory", entry.name());
                 } else {
                    let mut file = entry.into_file().unwrap();
                    let mut buf = Vec::new();
                    for i in 0..file.size { buf.push(0u8) }; 
                    file.read(buf.as_mut_slice());
                    let file_contents = if let Ok(string) = String::from_utf8(buf) {
                                    string
                                } else {
                                    String::from("File contains invalid UTF-8\n")
                                };
                    if count < args.len() - 1 {
                        kprintln!("{}", file_contents);
                    } else {
                        kprint!("{}", file_contents);
                    }
                    
                 }
            },
            Err(_) => kprintln!("cat: {}: No such file or directory", file)
        }
        count += 1;
    }
}
fn cd(args: & [&str], cwd: &mut PathBuf) {
    let length = args.len();
    let dest;
    if length == 0 {
        dest = "/";
    } else if length == 1 {
        dest = args[0];
    } else {
        kprintln!("too many arguments");
        return ();
    }
    let path = construct_path(dest, cwd);
    match FILESYSTEM.open(path.clone()) {
        Ok(entry) => {
            //resolve path       
            while cwd.pop() == true {;};
            cwd.push(path);

        },
        Err(_) => { kprintln!("no such file or directory"); }
    }


}
fn ls(args: & [&str], cwd: &mut PathBuf) {
    let mut path = PathBuf::new();
    let show_hidden;
    if args.len() == 0 {
        //current directory
        show_hidden = false;
        path = cwd.clone();
        let entries = get_entries(&mut path, show_hidden);
        display_entries(entries);
    } else if args.len() == 1 {
        //specified directory
        if args[0] == "-a" {
            show_hidden = true;
            path = cwd.clone();
        } else {
            show_hidden = false;
            path = construct_path(args[0], cwd);
        }
        let entries = get_entries(&mut path, show_hidden);
        display_entries(entries);
    } else if args.len() == 2 {
        //flag and specified directory
        if args[0] == "-a" {
            //display all 
            show_hidden = true;
            path = construct_path(args[1], cwd);
            let entries = get_entries(&mut path, show_hidden);
            display_entries(entries);

        } else {
            kprintln!("Only the '-a' flag is supported currently!");
        } 
    } else {
        kprintln!("Only the '-a' flag is supported currently!");
    }
}


//TODO: Add support for ls-ing a file (return vector of length 1)
fn get_entries(cwd: &mut PathBuf, show_hidden: bool) -> io::Result<Vec<Entry<PiVFatHandle>>> {
    let entry = FILESYSTEM.open(cwd.clone())?;
    let mut entry_vec = Vec::new();
    if entry.is_dir() {
        let mut entries = entry.into_dir().unwrap().entries().unwrap();
        let filtered_entries = entries.filter(|entry| show_hidden || !entry.metadata().hidden());
        for entry in filtered_entries {
            entry_vec.push(entry);
        }
    } else {
        entry_vec.push(entry);
    }
    
    Ok(entry_vec)
}
fn display_entries(entries: io::Result<Vec<Entry<PiVFatHandle>>>) {
    match entries {
        Ok(entries) => {
            entries.into_iter().for_each(|x| {
                let mut output_string = String::new();
                format_output(&mut output_string, &x);
                kprintln!("{}", output_string);
            });
            // entries.into_iter().for_each(|x| kprint!("{}\t", x.name()));
        },
        Err(e) => kprintln!("No such directory,{:?}", e)
    }
}

fn format_output<T: EntryTrait>(formatted_output: &mut String, entry: &T) -> ::core::fmt::Result {
    use core::fmt::Write;

    fn write_bool(to: &mut String, b: bool, c: char) -> ::core::fmt::Result {
        if b {
            write!(to, "{}", c)
        } else {
            write!(to, "-")
        }
    }
    
    fn write_timestamp<T: Timestamp>(to: &mut String, ts: T) -> ::core::fmt::Result {
        write!(
            to,
            "{:02}/{:02}/{} {:02}:{:02}:{:02} ",
            ts.month(),
            ts.day(),
            ts.year(),
            ts.hour(),
            ts.minute(),
            ts.second()
        )
    }
    
    write_bool(formatted_output, entry.is_dir(), 'd')?;
    write_bool(formatted_output, entry.is_file(), 'f')?;
    write_bool(formatted_output, entry.metadata().read_only(), 'r')?;
    write_bool(formatted_output, entry.metadata().hidden(), 'h')?;
    write!(formatted_output, "\t")?;
    
    write_timestamp(formatted_output, entry.metadata().created())?;
    write_timestamp(formatted_output, entry.metadata().modified())?;
    write_timestamp(formatted_output, entry.metadata().accessed())?;
    write!(formatted_output, "\t")?;
    
    write!(formatted_output, "{}", entry.name())?;

    Ok(())
}

// takes a 'destination path' and returns the appropriate path depending on if the destination was absolute or relative
fn construct_path(dest: &str, cwd: &mut PathBuf) -> PathBuf {
    let mut path = PathBuf::new();
    path.push(dest);
    if path.is_relative() {
        path = PathBuf::new();
        path.push(cwd);
        path.push(dest);
    }
    //resolve '.' and '..' in path bc '..' was causing UB
    let mut resolved_path = PathBuf::new();
    let mut comps = path.as_path().components();
    for comp in comps {
        match comp {
            Component::RootDir => {resolved_path.push("/");},
            Component::Normal(s) => {resolved_path.push(s);},
            Component::ParentDir => {resolved_path.pop();}
            _ => {;}
        }
    }

    resolved_path
}














//helper functions for displaying correct characters
fn is_visible_ascii(ascii_value: u8) -> bool {
    ascii_value >= 32 && ascii_value <= 127
}
fn is_erase_ascii(ascii_value: u8) -> bool {
    ascii_value == 8 || ascii_value == 127
}
