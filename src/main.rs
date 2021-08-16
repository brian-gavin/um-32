use std::{env, fs};
use um32::{read_scroll, Cpu};

enum Mode {
    Execute,
    Disassemble,
}

struct Opts {
    mode: Mode,
    scroll: fs::File,
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let mode = match args[1].as_str() {
        "-e" => Mode::Execute,
        "-d" => Mode::Disassemble,
        s => {
            eprintln!("Invalid argument '{}': expected '-e' or '-d'", s);
            std::process::exit(1);
        }
    };
    let scroll = fs::File::open(args[2].clone()).unwrap();
    let opts = Opts { mode, scroll };
    match opts.mode {
        Mode::Execute => {
            let mut cpu = Cpu::new(read_scroll(opts.scroll));
            cpu.spin_cycle();
        }
        Mode::Disassemble => um32::disassemble(read_scroll(opts.scroll)),
    }
}
