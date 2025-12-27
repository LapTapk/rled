mod core;

use crate::core::control::cfg;
use crate::core::parse::ParseTree;
use crate::core::syntax::{SyntaxNode, SyntaxTree};
use erlang::binary_to_term;
use std::env;
use std::process::{Command, ExitCode, Stdio};

fn main() -> ExitCode {
    let path = match env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("Usage: rled <path>");
            return ExitCode::FAILURE;
        }
    };

    std::fs::remove_dir_all("rled.tmp");
    std::fs::create_dir("rled.tmp").expect("failed to create directory");
    let status = match Command::new("./erl/beam_disasm.escript")
        .arg(&path)
        .arg("rled.tmp/etf")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to execute {}: {}", path, e);
            return ExitCode::FAILURE;
        }
    };

    env_logger::init();
    let etf = std::fs::read("rled.tmp/etf").expect("Failed to read ETF file of BEAM");
    let term = binary_to_term(&etf).expect("Failed to parse ETF file of BEAM");
    let parsed = <SyntaxNode as ParseTree>::parse(&term);
    let SyntaxNode::Module(module) = parsed else {
        panic!("{:?}", parsed)
    };
    let SyntaxNode::Func(func) = &module.funcs[0] else {
        panic!("{:?}", module)
    };
    //println!("{}", func.dump());
    let cfged = cfg(func.instrs.clone());
    let result = cfged.dump();
    println!("{}", result);

    ExitCode::from(status.code().unwrap_or(1) as u8)
}
