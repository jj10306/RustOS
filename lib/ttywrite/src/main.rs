mod parsers;


use serial;
use structopt;
use structopt_derive::StructOpt;
use xmodem::{Xmodem, Progress};

use std::path::PathBuf;
use std::time::Duration;


use structopt::StructOpt;
use serial::core::{CharSize, BaudRate, StopBits, FlowControl, SerialDevice, SerialPortSettings};

use parsers::{parse_width, parse_stop_bits, parse_flow_control, parse_baud_rate};

#[derive(StructOpt, Debug)]
#[structopt(about = "Write to TTY using the XMODEM protocol by default.")]
struct Opt {
    #[structopt(short = "i", help = "Input file (defaults to stdin if not set)", parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(short = "b", long = "baud", parse(try_from_str = "parse_baud_rate"),
                help = "Set baud rate", default_value = "115200")]
    baud_rate: BaudRate,

    #[structopt(short = "t", long = "timeout", parse(try_from_str),
                help = "Set timeout in seconds", default_value = "10")]
    timeout: u64,

    #[structopt(short = "w", long = "width", parse(try_from_str = "parse_width"),
                help = "Set data character width in bits", default_value = "8")]
    char_width: CharSize,

    #[structopt(help = "Path to TTY device", parse(from_os_str))]
    tty_path: PathBuf,

    #[structopt(short = "f", long = "flow-control", parse(try_from_str = "parse_flow_control"),
                help = "Enable flow control ('hardware' or 'software')", default_value = "none")]
    flow_control: FlowControl,

    #[structopt(short = "s", long = "stop-bits", parse(try_from_str = "parse_stop_bits"),
                help = "Set number of stop bits", default_value = "1")]
    stop_bits: StopBits,

    #[structopt(short = "r", long = "raw", help = "Disable XMODEM")]
    raw: bool,
}

fn main() {
    use std::fs::File;
    use std::io::{self, BufReader};

    let opt = Opt::from_args();
    let mut port = serial::open(&opt.tty_path).expect("path points to invalid TTY");

    // FIXME: Implement the `ttywrite` utility.
    // let input_data;
    let mut boxy: Box<dyn io::Read>;
    if opt.input.is_some() {
        //why did copy not work when using a file and vec or another file
        let reader = File::open(opt.input.unwrap()).expect("error opening the file");
        boxy = Box::new(reader);
        // let mut writer: Vec<u8> = vec![];
        // let mut writer = fs::File::create("write.txt").expect("error creating the file");
    } else {
        //read from stdin
        boxy = Box::new(io::stdin());
    }
    if opt.raw {
        //wouldnt this give a double reference?
        io::copy(&mut boxy, &mut port).expect("copy failed");
    } else {
        fn progress_fn(progress: Progress) {
            println!("Progress: {:?}", progress);
        }
        let bytes = Xmodem::transmit_with_progress(boxy, port, progress_fn).unwrap();
        println!("Wrote {} bytes to input", bytes);
    }

}
