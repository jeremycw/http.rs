extern crate collections;
extern crate getopts;
extern crate pcre;

use collections::HashMap;
use getopts::{OptGroup, getopts, optflag};
use pcre::Pcre;
use std::io;
use std::io::stdio::stderr;
use std::os;
use std::ascii::StrAsciiExt;

enum ParseState {
  StartLine,
  Headers,
  Body
}

fn print_usage(program: &str, opts: &[OptGroup]) {
  drop(opts);
  println!("Usage: {} [options] pattern subject", program);
  println!("Options:");
  println!("  -h, --help      Print usage and exit");
  println!("  --version       Print version information and exit");
}

fn print_version_info() {
  println!("rust-http 0.1");
}

fn process_regex(line: ~str, re: &Pcre) -> ~[~str] {
  let opt_m = re.exec(line);
  let m = match opt_m {
    None => return ~[],
    Some(m) => m
  };
  let mut i = 1u;
  let mut matches = ~[];
  while i < m.string_count() {
    matches.push(m.group(i).to_owned());
    i += 1;
  }
  return matches;
}

fn main() {
  let args = os::args();
  let program = args[0].clone();

  let opts = ~[
    optflag("h", "help", "print usage and exit"),
    optflag("", "version", "print version information and exit")
  ];

  let opt_matches = match getopts(args.tail(), opts) {
    Ok(m)  => m,
    Err(f) => {
      stderr().write_line(format!("Error: {}", f.to_err_msg()));
      os::set_exit_status(1);
      return;
    }
  };

  if opt_matches.opt_present("h") || opt_matches.opt_present("help") {
    print_usage(program, opts);
    return;
  }

  if opt_matches.opt_present("version") {
    print_version_info();
    return;
  }

  let start_line_re = Pcre::compile("^([A-Z]+)\\s+(\\/.*?)\\s+HTTP/1.1\r\n$").unwrap();
  let header_re = Pcre::compile("^(.*?):\\s+(.*?)\r\n$").unwrap();
  loop {
    let mut state = StartLine;
    let mut env = HashMap::new();

    for line in io::stdin().lines() {
      let subject = line.unwrap();

      state = match state {
        Headers => if subject == ~"\r\n" { Body } else { Headers },
        _ => state
      };
        
      match state {
        StartLine => {
          let matches = process_regex(subject.to_owned(), &start_line_re);
          env.insert(~"REQUEST_METHOD", matches[0].clone());
          env.insert(~"SCRIPT_NAME", ~"");
          let fullpath = matches[1].clone();
          let mut peices = fullpath.split_str("?");
          env.insert(~"PATH_INFO", peices.next().unwrap().to_owned());
          let query_string = match peices.next() {
            Some(s) => s,
            None => ""
          };
          env.insert(~"QUERY_STRING", query_string.to_owned());
          state = Headers;
        },
        Headers => {
          let matches = process_regex(subject.to_owned(), &header_re);
          let key = matches[0].clone().replace("-", "_").to_ascii_upper();
          env.insert(~"HTTP_" + key, matches[1].clone());
        }
        Body => { break }
      }
    }
    print!("HTTP/1.1 200 OK\r\n");
    print!("Server: rust-http/0.1\r\n");
    print!("Content-Type: text/plain\r\n");
    let mut body = ~"";
    for (k, v) in env.iter() {
      body =  body + format!("{} : {}\n", k, v);
    }
    print!("Content-Length: {}\r\n", body.len());
    print!("\r\n");
    print!("{}", body);
    io::stdio::flush();
  }
}
