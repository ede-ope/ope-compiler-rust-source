use std::env;
use std::process::{self};

mod compiler;
mod toplevel_analyser;
// mod interprete;
mod blocklevel_analyser;
mod borrow_checker;
mod cfg_builder;
mod cfg_resourses;
mod codegen;
mod dk_expert;
mod macro_expand;
mod move_checker;
mod name_resolver;
mod parser;
// mod runner;
mod scanner;
mod type_checker;

mod llvm;

use compiler::compile_file;
// use runner::run_file;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Extension {
    L,
    Kui,
}

#[derive(Debug, Clone)]
pub enum BuildType {
    SharedLib,
    StaticLib,
    Object,
}

#[derive(Debug, Clone)]
pub struct ComdInfo {
    pub filename: String,
    pub target: String,
    pub win: bool,
    pub make: Option<BuildType>,
    pub name: Option<String>,
}

// impl ComdInfo {
//     fn new(filename: &str) -> Self {
//         Self {
//             filename: String::from(filename),
//             target: None,
//             win: false,
//             make: None,
//             name: None
//         }
//     }
// }

static VERSION: &'static str = "Ẹ̀dà 0.1.0";

fn main() {
    let mut args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "--tẹ" | "--te" => {
            // can be use to print target triples supported e.t.c
        }
        "--ẹ̀dà" | "--eda" => {
            if args.len() > 2 {
                eprintln!(
                    "Unexpected argument{}:",
                    if args.len() > 3 { "s" } else { "" }
                );
                let mut i = 2;
                for each in args {
                    if i >= 2 {
                        eprintln!(" {}", each);
                    }
                    i += 1;
                }
                print_usage();
                process::exit(1);
            }
            println!("{}", VERSION);
            process::exit(0);
        }
        _ => {
            let mut filename = args[1].clone();
            if !filename.to_lowercase().ends_with(".kui") {
                filename.push_str(".kui");
            }

            // let get_host = env!("BUILD_HOST");

            // if let Err(err) = get_host {
            //     eprintln!("WARNING: {}\n    Target triple therefore defaults to x86_64-unknown-linux-gnu", err)
            // }
            
            let mut àfojúsùn = env!("BUILD_HOST").to_string();
            
            let mut fèrèsé = false;
            let mut ṣe: Option<BuildType> = None;
            let mut p: Option<String> = None;

            if args.len() > 2 {
                // means there are flags available
                // index 2 is:
                // kui main --àfojúsùn
                //  0    1     2

                let flags = args.split_off(2);
                let mut i = 0;
                while i < flags.len() {
                    match flags[i].as_str() {
                        "--àfojúsùn" | "--afojusun" => {
                            // kui main --àfojúsùn
                            //advance
                            i += 1;
                            if flags.len() >= i + 1 {
                                // kui main --àfojúsùn x86_64-pc-windows-gnu
                                let target_triple = flags[i].clone();
                                if is_target_triple(target_triple.clone()) {
                                    àfojúsùn = target_triple;
                                } else {
                                    eprintln!("Unsupported target triple: {}", target_triple);
                                    print_supported_target_triple();

                                    process::exit(1);
                                }
                            } else {
                                eprintln!("Provide a target triple");
                                print_supported_target_triple();

                                process::exit(1);
                            }
                        }
                        "--ṣe" => {
                            i += 1;
                            if flags.len() >= i + 1 {
                                // àjọlò --p èlò
                                match flags[i].as_str() {
                                    "àjọlò" | "ajolo" => {
                                        // kui main --ṣe àjọlò
                                        ṣe = Some(BuildType::SharedLib);
                                    }
                                    "àdálò" | "adalo" => {
                                        ṣe = Some(BuildType::StaticLib);
                                    }
                                    "ohun" => {
                                        ṣe = Some(BuildType::Object);
                                    }
                                    oth => {
                                        eprintln!(
                                            "{} is not an output file type for any target",
                                            oth
                                        );
                                        println!("Examples:");
                                        println!("  --ṣe àjọlò");
                                        println!("  --ṣe àdálò");
                                        println!("  --ṣe ohun");
                                    }
                                }
                            } else {
                                eprintln!("Provide an output file type");
                                println!("Examples:");
                                println!("  --ṣe àjọlò");
                                println!("  --ṣe àdálò");
                                println!("  --ṣe ohun");

                                process::exit(1);
                            }
                        }
                        "--fèrèsé" => {
                            i += 1;
                            fèrèsé = true;
                        }
                        "--p" => {
                            i += 1;
                            if flags.len() >= i + 1 {
                                // only the name supposed to remain
                                p = Some(flags[i].clone());
                            } else {
                                eprintln!(
                                    "No name provided after --p flag:"
                                );
                                print_usage();
                                process::exit(1);
                            }
                        }
                        flag => {
                            eprintln!("Unknown flag {}", flag);
                            print_usage();
                            process::exit(1);
                        }
                    }
                    i += 1;
                }
            }

            let cli = ComdInfo {
                filename,
                target: àfojúsùn,
                win: fèrèsé,
                make: ṣe,
                name: p,
            };

            compile_file(cli);
        }
    }
}

fn is_target_triple(target_triple: String) -> bool {
    if target_triple == format!("x86_64-pc-windows-gnu") ||
        target_triple == format!("x86_64-pc-windows-msvc") ||
        target_triple == format!("i686-pc-windows-msvc") ||
        target_triple == format!("aarch64-pc-windows-msvc") ||
        // linux
        target_triple == format!("x86_64-unknown-linux-gnu") ||
        target_triple == format!("x86_64-unknown-linux-musl") ||
        target_triple == format!("aarch64-unknown-linux-gnu") ||
        target_triple == format!("aarch64-unknown-linux-musk") ||
        // MacOs
        target_triple == format!("x86_64-apple-darwin") ||
        target_triple == format!("aarch64-apple-darwin") ||
        // WASM
        target_triple == format!("wasm32-unknown-unknown") ||
        target_triple == format!("wasm32-wasi") ||
        // bare metal
        target_triple == format!("x86_64-unknown-none") ||
        target_triple == format!("thumbv7m-none-eabi") ||
        target_triple == format!("riscv32imac-unknown-none-elf") ||
        // kui lib
        target_triple == format!("odù-kúì") ||
        target_triple == format!("odu-kui")
    {
        true
    } else {
        false
    }
}

fn print_supported_target_triple() {
    println!("Here are the list of all the target_triple supported by Kui");
    println!("# Windows");
    println!("  x86_64-pc-windows-gnu");
    println!("  x86_64-pc-windows-msvc");
    println!("  i686-pc-windows-msvc");
    println!("  aarch64-pc-windows-msvc");
    println!("#Linux");
    println!("  x86_64-unknown-linux-gnu");
    println!("  x86_64-unknown-linux-musl");
    println!("  aarch64-unknown-linux-gnu");
    println!("  aarch64-unknown-linux-musk");
    println!("#MacOs");
    println!("   x86_64-apple-darwin");
    println!("   aarch64-apple-darwin");
    println!("#WASM");
    println!("  wasm32-unknown-unknown");
    println!("  wasm32-wasi");
    println!("#Bare metal");
    println!("   x86_64-unknown-none");
    println!("   thumbv7m-none-eabi");
    println!("   riscv32imac-unknown-none-elf");
    println!("#kui lib");
    println!("   odù-kúì");
    println!("   odu-kui");
}

fn print_usage() {
    println!("Usage:");
    println!("#For compilation");
    // kui main --àfojúsùn x86_64-pc-windows-gnu --p app
    println!("  kui <*source_file> [--àfojúsùn *target_triple] [--fèrèsé | --ṣe *build_type] [--p *given_name]");
    println!("  To compile main.kui: kui main");
    println!("other commands ->");
    println!("#Check version");
    println!("  kui --ẹ̀dà");
    println!("#To print info");
    println!("  kui --tẹ <target_triple | >");
}
