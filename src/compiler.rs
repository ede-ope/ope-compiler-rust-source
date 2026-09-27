use serde::Deserialize;
use colored::Colorize;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipArchive;

// use is_terminal::IsTerminal;as
use std::collections::HashMap;
use std::ffi::c_char;
use std::io::{self, IsTerminal};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Output;
use std::{fs, vec};

use crate::borrow_checker::{self, CustTypeInfo, FuncDeclInfo, ModCFG, Resource, TakeStmtInfo};
use crate::codegen::Codegen;
use crate::compiler::NameSpace::Singular;
use crate::macro_expand::AstMacroExpandsion;
use crate::parser::{Error, Ident, LibNextedLevel, ModPath, Parser, TopLevel}; // Import the parser // StaticBlock, StaticLevel,
use crate::scanner::{LexedSelf, Span, Token};
use crate::toplevel_analyser::Analyser;
use crate::{dk_expert, llvm::*, BuildType, ComdInfo};
use crate::scanner::Pos;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ModAST {
    pub mod_pub: Option<TopLevel>,
    pub globals: Vec<TopLevel>,
    pub funcs: Vec<TopLevel>,
    pub platform_n_s: Vec<TopLevel>,
    pub takes: Vec<TopLevel>,
    pub macro_def: Vec<TopLevel>,
    pub traits: Vec<TopLevel>,
    pub workers: Vec<TopLevel>,
    pub statics: Vec<TopLevel>,
    pub custom_tys: Vec<TopLevel>,
    pub bridges: Vec<TopLevel>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ASTs {
    pub file_ast: Vec<TopLevel>,
    pub entry: Option<TopLevel>,
    pub mods: Vec<ModAST>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Mod {
    pub mod_docs: Option<TopLevel>,
    pub main_fn: Option<TopLevel>,
    pub macro_defs: Vec<TopLevel>,
    pub fns: Vec<TopLevel>,
    pub platform_n_s: Vec<TopLevel>,
    pub takes: Vec<TopLevel>,
    pub globals: Vec<TopLevel>,
    pub traits: Vec<TopLevel>,
    pub workers: Vec<TopLevel>,
    pub statics: Vec<TopLevel>,
    pub custom_tys: Vec<TopLevel>,
    pub lines: Vec<String>,
    pub scope: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum NameSpace {
    Singular {
        name: String,
        file: Mod,
    },
    Plural {
        name: String,
        files: HashMap<String, Mod>,
    },
    Lib {
        name: String,
        manifest: DkManifest,
        entry: ModCFG,
        src: HashMap<String, ModCFG>,
    },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Linker {
    LLD,
    GCC,
    Clang,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CompilationUnit {
    pub name: String,
    pub mod_pub: Option<TopLevel>,
    pub main_fn: Option<TopLevel>,
    pub fns: Vec<TopLevel>,
    pub platform_n_s: Vec<TopLevel>,
    pub takes: Vec<TopLevel>,
    pub globals: Vec<TopLevel>,
    pub macro_rules: Vec<TopLevel>,
    pub traits: Vec<TopLevel>,
    pub workers: Vec<TopLevel>,
    pub statics: Vec<TopLevel>,
    pub custom_tys: Vec<TopLevel>,
    pub bridges: Vec<TopLevel>,
    pub mod_asts: HashMap<String, ModAST>,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ErrorHint {
    pub hints: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SemanticError {
    pub messages: Vec<String>,
    pub hints: Vec<ErrorHint>,
    pub span: Span,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct DkManifest {
    package: Package,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Package {
    name: String,
    version: String,
    authors: String,
    license: String,
    entry: String,
    dependencies: Vec<String>
}

pub fn handle_err(err: Vec<Error>, lines: HashMap<String, Vec<String>>) {
    // Telling 'colored' to only use colours when stdout is a real terminal
    if !std::io::stdout().is_terminal() {
        colored::control::set_override(false);
    }

    for e in err {
        // And Print out the error message
        eprintln!("{}: {}", "àṣìṣe".red().bold(), e.message);
        eprintln!("    {}\n", e.span.file);
        // Display the source code

        // let mut spaces = String::new();
        // for _i in 1..line_num.to_string().len() as i32 {
        //     spaces.push(' ');
        // }
        // println!("{:#?}", e.);
        let span = e.span.clone();
        let single_line;

        if span.st.line == span.en.line {
            single_line = true;
        } else {
            single_line = false;
        }

        if single_line {
            eprintln!(
                "{} |{}",
                span.st.line,
                lines.get(&e.span.file).unwrap_or(&vec![format!("")])[span.st.line - 1]
            );
            let mut indicator = String::new();
            for num in 0..span.en.column {
                if num >= span.st.column {
                    if num != span.en.column {
                        indicator.push('^');
                    }
                } else {
                    indicator.push(' ');
                }
            }
            let padding = String::from("   |");
            eprintln!("{}{}\n", padding, indicator);
        } else {
        }
        // eprintln!("{} |\n", spaces);
        // Then Indicate where the error is
    }
    println!("Compilation failed ❌\n");
}

pub fn handle_semantic_err(
    errs: &Vec<SemanticError>,
    lines_of_mods: &HashMap<String, Vec<String>>,
) {
    // Telling 'colored' to only use colours when stdout is a real terminal
    if !std::io::stdout().is_terminal() {
        colored::control::set_override(false);
    }

    let mut num_of_errs = 0;
    for e in errs {
        num_of_errs += 1;
        // And Print out the error message
        let mut i = 0;
        for message in &e.messages {
            if i == 0 {
                eprintln!("{}: {}", "àṣìṣe".red().bold(), message);
                eprintln!("     --{}--\n", e.span.file.bold());
            } else {
                eprintln!("     {}", message);
            }
            i += 1;
        }
        // Display the source code
        let single_line;

        if e.span.st.line == e.span.en.line {
            single_line = true;
        } else {
            single_line = false;
        }
        // let mut spaces = String::new();
        // for _i in 1..line_num.to_string().len() as i32 {
        //     spaces.push(' ');
        // }
        let lines = lines_of_mods.get(&e.span.file);

        if lines.is_some() {
            let lines = lines.unwrap();
            let line_num = format!("{}", e.span.st.line);

            eprintln!("");
            if single_line {
                if e.span.st.line != 0 {
                    eprintln!(
                        "{}{}{}",
                        line_num.cyan().bold(),
                        " | ".cyan().bold(),
                        lines[e.span.st.line - 1]
                    );
                }
                let mut indicator_padding = String::new();
                let mut indicator_red_underline = String::new();
                for num in 0..e.span.en.column {
                    if num >= e.span.st.column {
                        if num != e.span.en.column {
                            indicator_red_underline.push('^');
                        }
                    } else {
                        indicator_padding.push(' ');
                    }
                }
                let mut i = 0;
                let mut num_of_diacritics = 0;
                while i <= e.span.st.column {
                    if e.span.st.line != 0 {
                        let char = lines[e.span.st.line - 1].chars().nth(i);

                        if char.is_some() {
                            let char = char.unwrap();
                            if char == '\u{300}' || char == '\u{301}' {
                                num_of_diacritics += 1;
                            }
                        }
                        i += 1;
                    } else {
                        break;
                    }
                }
                let mut i = 0;
                while i < num_of_diacritics {
                    indicator_padding.pop();
                    i += 1;
                }
                let mut num_space = String::new();
                let mut j = 0;
                while j < line_num.len() {
                    num_space.push(' ');
                    j += 1;
                }
                let padding = format!("{}{}", num_space, " | ".cyan().bold());
                if e.span.st.line != 0 {
                    eprintln!(
                        "{}{}{}\n",
                        padding,
                        indicator_padding,
                        indicator_red_underline.red()
                    );
                }
            } else {
            }
        }
        // handle hints
        for hint in &e.hints {
            match hint {
                ErrorHint { hints, span } => {
                    let mut i = 0;
                    for message in hints {
                        if i == 0 {
                            eprintln!("{}: {}", "àkíyèsí".green().bold(), message);
                            eprintln!("     --{}--\n", span.file.bold());
                        } else {
                            eprintln!("     {}", message);
                        }
                        i += 1;
                    }
                    // Display the source code
                    let single_line;

                    if span.st.line == span.en.line {
                        single_line = true;
                    } else {
                        single_line = false;
                    }
                    // let mut spaces = String::new();
                    // for _i in 1..line_num.to_string().len() as i32 {
                    //     spaces.push(' ');
                    // }
                    let lines = lines_of_mods.get(&span.file);
                    if lines.is_some() {
                        let lines = lines.unwrap();
                        if single_line {
                            let line_num = format!("{}", span.st.line);
                            if span.st.line != 0 {
                                eprintln!(
                                    "{}{}{}",
                                    line_num.cyan().bold(),
                                    " | ".cyan().bold(),
                                    lines[span.st.line - 1]
                                );
                            }
                            let mut indicator_padding = String::new();
                            let mut indicator_green_underline = String::new();
                            for num in 0..span.en.column {
                                if num >= span.st.column {
                                    if num != span.en.column {
                                        indicator_green_underline.push('^');
                                    }
                                } else {
                                    indicator_padding.push(' ');
                                }
                            }

                            let mut num_space = String::new();
                            let mut j = 0;
                            while j < line_num.len() {
                                num_space.push(' ');
                                j += 1;
                            }
                            let padding = format!("{}{}", num_space, " | ".cyan().bold());
                            eprintln!(
                                "{}{}{}\n",
                                padding,
                                indicator_padding,
                                indicator_green_underline.green()
                            );
                        } else {
                        }
                    }
                }
            }
        }
    }
    println!(
        "\nCompilation failed, due to {} semantic error(s) ❌\n",
        num_of_errs
    );
}

fn is_file<'a>(path_str: &str, span: Span, syntax_err: &mut Vec<Error>) -> Result<bool, ()> {
    let result = fs::metadata(Path::new(path_str));

    match result {
        Ok(metadata) => {
            if metadata.is_file() {
                Ok(true)
            } else if metadata.is_dir() {
                Ok(false)
            } else {
                Ok(false)
            }
        }
        Err(result) => Err(syntax_err.push(Error {
            message: format!("{}", result),
            span,
        })),
    }
}

fn handle_link_output(
    output: Output,
    // cli: &CompInfo,
    // merged: LLVMModuleRef,
    obj_path: &str,
    to: &str,
) {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
        eprintln!("linking failed");
        std::process::exit(1);
    } else {
        if let Err(e) = std::fs::remove_file(obj_path) {
            print!("\r             \r");
            io::stdout().flush().unwrap();
            eprintln!("warning: could not delete {}: {}", obj_path, e);
        }

        println!("linked to {}", to);
    }
}

pub fn llvm_unsafe(cli: &ComdInfo, merged: LLVMModuleRef, base_name: &str, exe_dir: PathBuf) {
    unsafe {
        // 1. Initialize targets
        match &cli.target {
            Some(target_str) => match target_str.as_str() {
                t if t.contains("x86") || t.contains("i686") => {
                    LLVMInitializeX86TargetInfo();
                    LLVMInitializeX86Target();
                    LLVMInitializeX86TargetMC();
                    LLVMInitializeX86AsmPrinter();
                }
                t if t.contains("aarch64") || t.contains("arm64") => {
                    LLVMInitializeAArch64TargetInfo();
                    LLVMInitializeAArch64Target();
                    LLVMInitializeAArch64TargetMC();
                    LLVMInitializeAArch64AsmPrinter();
                }
                t if t.contains("riscv") => {
                    LLVMInitializeRISCVTargetInfo();
                    LLVMInitializeRISCVTarget();
                    LLVMInitializeRISCVTargetMC();
                    LLVMInitializeRISCVAsmPrinter();
                }
                _ => {
                    LLVMInitializeX86TargetInfo();
                    LLVMInitializeX86Target();
                    LLVMInitializeX86TargetMC();
                    LLVMInitializeX86AsmPrinter();
                }
            },
            None => {
                LLVMInitializeX86TargetInfo();
                LLVMInitializeX86Target();
                LLVMInitializeX86TargetMC();
                LLVMInitializeX86AsmPrinter();
            }
        }

        // 2. Get target triple
        let triple = match &cli.target {
            Some(t) => std::ffi::CString::new(t.as_str()).unwrap(),
            None => {
                let default = LLVMGetDefaultTargetTriple();
                let s = std::ffi::CStr::from_ptr(default)
                    .to_string_lossy()
                    .to_string();
                LLVMDisposeMessage(default);
                std::ffi::CString::new(s).unwrap()
            }
        };

        // 3. Look up target
        let mut target: LLVMTargetRef = std::ptr::null_mut();
        let mut err: *mut c_char = std::ptr::null_mut();
        LLVMGetTargetFromTriple(triple.as_ptr(), &mut target, &mut err);
        if !err.is_null() {
            let msg = std::ffi::CStr::from_ptr(err).to_string_lossy();
            eprintln!("target error: {}", msg);
            LLVMDisposeMessage(err);
            std::process::exit(1);
        }

        // 4. Create target machine
        let target_machine = LLVMCreateTargetMachine(
            target,
            triple.as_ptr(),
            c"generic".as_ptr(),
            c"".as_ptr(),
            LLVMCodeGenLevelDefault,
            LLVMRelocDefault,
            LLVMCodeModelDefault,
        );

        // 6. Detect platform from triple for file extensions
        let triple_str = triple.to_string_lossy();
        let is_windows = triple_str.contains("windows");
        let is_macos = triple_str.contains("apple") || triple_str.contains("darwin");

        // 7. Emit object file
        let obj_ext = if is_windows { "obj" } else { "o" };
        let obj_path = std::path::PathBuf::from(&base_name)
            .with_extension(obj_ext)
            .to_string_lossy()
            .to_string();

        let obj_cstring = std::ffi::CString::new(obj_path.as_str()).unwrap();
        let mut emit_err: *mut c_char = std::ptr::null_mut();
        let emit_result = LLVMTargetMachineEmitToFile(
            target_machine,
            merged,
            obj_cstring.as_ptr(),
            LLVMObjectFile,
            &mut emit_err,
        );
        if emit_result != 0 && !emit_err.is_null() {
            let msg = std::ffi::CStr::from_ptr(emit_err).to_string_lossy();
            eprintln!("emit error: {}", msg);
            LLVMDisposeMessage(emit_err);
            std::process::exit(1);
        }

        // 8. Build output based on BuildType
        match &cli.make {
            // None = default = executable
            None => {
                let has_main =
                    LLVMGetNamedFunction(merged, c"main".as_ptr()) != std::ptr::null_mut();
                if !has_main {
                    eprintln!("⚠️  Warning: Module does not have a main function!");
                    eprintln!("   No entry point!");
                    // std::process::exit(1);
                }
                if is_windows {
                    let exe_path = format!("{}.exe", base_name);
                    // pass win (GUI vs CLI) to linker only on Windows
                    let output = Codegen::link_windows(&obj_path, &exe_path, exe_dir, &cli);

                    print!("\r             \r");
                    io::stdout().flush().unwrap();

                    handle_link_output(output, &obj_path, &exe_path);
                } else if is_macos {
                    let _base_name = base_name; // no extension on macOS
                } else {
                    let _base_name = base_name; // no extension on Linux
                };
            }
            // object file only — already emitted above
            Some(BuildType::Object) => {
                println!("object: {}", obj_path);
            }
            // static library
            Some(BuildType::StaticLib) => {
                let (prefix, ext) = if is_windows {
                    ("", "lib")
                } else {
                    ("lib", "a")
                };
                let lib_path = format!("{}{}.{}", prefix, base_name, ext);

                std::process::Command::new("llvm-ar")
                    .args(["rcs", &lib_path, &obj_path])
                    .status()
                    .expect("llvm-ar not found");

                if let Err(e) = std::fs::remove_file(&obj_path) {
                    eprintln!("warning: could not delete {}: {}", obj_path, e);
                }
                println!("static lib: {}", lib_path);
            }
            // dynamic library
            Some(BuildType::SharedLib) => {
                if is_windows {
                    let dll_path = format!("{}.dll", base_name);
                    let output = Codegen::link_windows(&obj_path, &dll_path, exe_dir, &cli);

                    print!("\r             \r");
                    io::stdout().flush().unwrap();
                    handle_link_output(output, &obj_path, &dll_path);
                } else if is_macos {
                    let _dylib_path = format!("lib{}.dylib", base_name);
                } else {
                    let _so_path = format!("lib{}.so", base_name);
                };
            }
        }

        LLVMDisposeTargetMachine(target_machine);
    }
}

fn gen_from_mod_cfg(
    name: &String,
    cfg: &HashMap<String, Resource>,
    file: &ModCFG,
    each_mod_ir: &mut HashMap<String, Codegen>,
) {
    match file {
        ModCFG {
            globals: _,
            platform_n_s: _,
            takes,
            traits: _,
            workers: _,
            statics: _,
            custom_tys,
            methods: _,
            static_meths: _,
            bridges: _,
            main_fn,
            funcs,
        } => {
            let mut codegen = Codegen::new(name);
            // register all type NAMES (opaque/forward declarations)
            codegen.hoist_type_names(custom_tys);

            // fill in type BODIES (now all names are known)
            codegen.hoist_type_bodies(custom_tys);
            
            for import in takes {
                match import.1 {
                    TakeStmtInfo {
                        take,
                        from,
                        lib,
                        stm_span: _,
                    } => {
                        if lib.is_none() {
                            match from {
                                ModPath::Ident(ident) => {
                                    let src_mod = &cfg.get(&ident.ident);
                                    
                                    if src_mod.is_some() {
                                        let src_mod = src_mod.unwrap();
                                        match src_mod {
                                            Resource::File(file_src_mod) | 
                                            Resource::Package { entry: file_src_mod, src: _ }
                                            // if it's package then, we are importing from it's entry
                                             => {
                                                match &take.0.kind {
                                                    Token::Identifier(str) => {
                                                        if let Some(src_def) =
                                                            file_src_mod.custom_tys.get(str)
                                                        {
                                                            // // foreign type — opaque declaration, same name
                                                            // let src_def = get_cust_from_oth(
                                                            //     take.clone(),
                                                            //     from.clone(),
                                                            //     lib.clone(),
                                                            //     &cfg,
                                                            // );
                                                            // if src_def.is_some() {
                                                            //     let src_def = src_def.unwrap();
                                                                match src_def {
                                                                    CustTypeInfo::StructDeclInfo { .. } => {
                                                                        codegen
                                                                            .declare_foreign_struct_name(str);
                                                                    }
                                                                    CustTypeInfo::EnumDeclInfo { .. } => {
                                                                        panic!("Enum not yet addressed")
                                                                    }
                                                                }
                                                            // }
                                                        } else if let Some(src_func) =
                                                            file_src_mod.funcs.get(str)
                                                        {
                                                            // then it's a function,
                                                            // let src_func = get_func_from_oth(
                                                            //     take.clone(),
                                                            //     from.clone(),
                                                            //     lib.clone(),
                                                            //     &cfg,
                                                            // );
                                                            // if src_func.is_some() {
                                                            //     let src_func = src_func.unwrap();
                                                                match &src_func {
                                                                    FuncDeclInfo {
                                                                        expect,
                                                                        par,
                                                                        ..
                                                                    } => {
                                                                        codegen
                                                                            .gen_extern_function(
                                                                                str, expect, par,
                                                                                false,
                                                                            );
                                                                    }
                                                                }
                                                            // }
                                                        }
                                                    }
                                                    _ => (),
                                                }
                                            }
                                            Resource::Folder(_) => todo!(""),
                                        }
                                    }
                                }
                                _ => (), // if it's not a lib then the module most an ident
                            }
                        }
                    }
                }
            }

            //
            codegen.hoist_functions(funcs);

            for f in funcs {
                codegen.gen_func(f.0, f.1);
            }

            if main_fn.is_some() {
                let main_fn = main_fn.clone().unwrap();
                codegen.gen_entry(&main_fn);
            }

            // eprintln!("=== Module {} ===", name);
            // codegen._dump();
            // codegen.dump();

            each_mod_ir.insert(name.clone(), codegen);
        }
    }
}

pub fn compile_file(cli: ComdInfo) {
    let exe_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let name = Path::new(&cli.filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&cli.filename)
        .to_string();

    let mut lines_of_mods = HashMap::new();

    let source = fs::read_to_string(&cli.filename).expect("Failed to read source file\n");

    println!("Compiling {}...\n", &cli.filename);

    let mut syntax_errs: Vec<Error> = vec![];
    // PARSE THE ENTRY FILE/MODULE
    let mut new_lex = LexedSelf::new(&source, name.clone());
    let lex_result = new_lex.lex();

    if !new_lex.errs.is_empty() {
        for err in new_lex.errs {
            syntax_errs.push(err);
        }
    }

    let tokens = lex_result.tokens;

    let lines = lex_result.lines;

    let mut parser = Parser::new(tokens, name.clone(), true);

    let result = parser.parse_program();

    let mut unit: HashMap<String, NameSpace> = HashMap::new();

    lines_of_mods.insert(name.clone(), lines.clone());

    unit.insert(
        name.clone(),
        NameSpace::Singular {
            name: name.clone(),
            file: Mod {
                mod_docs: result.mod_docs,
                main_fn: result.main_fn,
                macro_defs: result.macro_def,
                fns: result.funcs,
                platform_n_s: result.platform_n_s,
                takes: result.takes,
                globals: result.globals,
                traits: result.traits,
                workers: result.workers,
                statics: result.statics,
                custom_tys: result.custom_tys,
                lines: lines.clone(),
                scope: 0,
            },
        },
    );
    // handle errors incurred from the entry file

    if !result.errs.is_empty() {
        for err in result.errs {
            syntax_errs.push(err);
        }
    }

    // ADD THE CORE MODULE
    let kui_path = std::env::current_exe().unwrap();
    let kui_dir = kui_path.parent().unwrap();

    let core_mod_path = kui_dir.join("gbùngbùn.kui");
    // For every use
    // let path_check = is_file(&core_mod_path);
    // if let Ok(result) = path_check {
    //     if result {
    // if it's a file
    let err_msg = format!(
        "Failed to read source file {}\n",
        core_mod_path.to_str().unwrap_or(&"unknown")
    );
    let core_mod_source = fs::read_to_string(&core_mod_path).expect(&err_msg);

    let mut core_mod_lex = LexedSelf::new(&core_mod_source, format!("<gbùngbùn>"));
    let core_mod_lex_result = core_mod_lex.lex();

    if !core_mod_lex.errs.is_empty() {
        for err in core_mod_lex.errs {
            syntax_errs.push(err);
        }
    }

    let core_mod_tokens = core_mod_lex_result.tokens;
    // let _mod_line_spans = mod_lex_result.line_spans;
    let core_mod_lines = core_mod_lex_result.lines;
    lines_of_mods.insert(String::from("<gbùngbùn>"), core_mod_lines.clone());
    let mut core_mod_parser = Parser::new(core_mod_tokens, format!("<gbùngbùn>"), false);
    let core_mod_result = core_mod_parser.parse_program();

    // we should check if the name does not match any of the existing ones
    unit.insert(
        String::from("<gbùngbùn>"),
        Singular {
            name: String::from("<gbùngbùn>"),
            file: Mod {
                mod_docs: core_mod_result.mod_docs,
                main_fn: core_mod_result.main_fn,
                macro_defs: core_mod_result.macro_def,
                fns: core_mod_result.funcs,
                platform_n_s: core_mod_result.platform_n_s,
                takes: core_mod_result.takes,
                globals: core_mod_result.globals,
                traits: core_mod_result.traits,
                workers: core_mod_result.workers,
                statics: core_mod_result.statics,
                custom_tys: core_mod_result.custom_tys,
                lines: core_mod_lines.clone(),
                scope: 0,
            },
        },
    );

    if !core_mod_result.errs.is_empty() {
        for err in core_mod_result.errs.clone() {
            syntax_errs.push(err);
        }
    }

    // FOR EACH MODULE, FOLDER AND LIBRARY
    for m in result.mods {
        match m {
            TopLevel::UseStmt {
                module,
                nexted_level,
                as_,
                is_public: _,
                info: _,
            } => {
                let mod_path;
                let mod_ident; // without the .kui extention
                match module.clone() {
                    Ident { ident, span: _ } => {
                        mod_path = std::path::PathBuf::from(ident.clone())
                            .with_extension("kui")
                            .to_string_lossy()
                            .to_string();

                        mod_ident = ident;
                    }
                }
                // For every use
                let path_check = is_file(&mod_ident, module.span.clone(), &mut syntax_errs);
                if let Ok(result) = path_check {
                    if result {
                        // if it's a file
                        let mut show_nexted_error = false;
                        match nexted_level {
                            LibNextedLevel::All => show_nexted_error = true,
                            LibNextedLevel::To(n) => {
                                if n > 0 {
                                    show_nexted_error = true;
                                }
                            }
                        }
                        if show_nexted_error {
                            syntax_errs.push(Error {
                                message: format!(
                                    "Warning: '{}' is a file path therefore the nexted level syntax(..) is useless here",
                                    module.ident
                                ),
                                span: module.span.clone(),
                            });
                        }

                        if mod_path.to_lowercase().ends_with(".kui") {
                            let mod_source = fs::read_to_string(&mod_path)
                                .expect("Failed to read source file\n");

                            let mut mod_lex = LexedSelf::new(&mod_source, mod_ident.clone());
                            let mod_lex_result = mod_lex.lex();

                            if !mod_lex.errs.is_empty() {
                                for err in mod_lex.errs {
                                    syntax_errs.push(err);
                                }
                            }

                            let mod_tokens = mod_lex_result.tokens;
                            // let _mod_line_spans = mod_lex_result.line_spans;
                            let mod_lines = mod_lex_result.lines;
                            lines_of_mods.insert(as_.clone(), mod_lines.clone());
                            let mut mod_parser = Parser::new(mod_tokens, mod_ident, false);
                            let mod_result = mod_parser.parse_program();

                            lines_of_mods.insert(mod_path.clone(), mod_lines.clone());

                            // we should check if the name does not match any of the existing ones
                            unit.insert(
                                mod_path.clone(),
                                Singular {
                                    name: as_.clone(),
                                    file: Mod {
                                        mod_docs: mod_result.mod_docs,
                                        main_fn: mod_result.main_fn,
                                        macro_defs: mod_result.macro_def,
                                        fns: mod_result.funcs,
                                        platform_n_s: mod_result.platform_n_s,
                                        takes: mod_result.takes,
                                        globals: mod_result.globals,
                                        traits: mod_result.traits,
                                        workers: mod_result.workers,
                                        statics: mod_result.statics,
                                        custom_tys: mod_result.custom_tys,
                                        lines: mod_lines.clone(),
                                        scope: 0,
                                    },
                                },
                            );

                            if !mod_result.errs.is_empty() {
                                for err in mod_result.errs.clone() {
                                    syntax_errs.push(err);
                                }
                            }
                        } else {
                            // Error: Not a kui file
                            syntax_errs.push(Error {
                                message: format!(
                                    "The file {:#?} on line {} is not a .kui file",
                                    module.ident.clone(),
                                    module.span.st.line.clone()
                                ),
                                span: module.span,
                            });
                        }
                    } else {
                        // It's a folder then
                        todo!("Using a folder")
                    }
                }
            }
            TopLevel::UseLibStmt {
                module: lib,
                info: _,
            } => {
                let lib_ident; // without the .kui extention
                match lib.clone() {
                    Ident { ident, span: _ } => {
                        lib_ident = ident;
                    }
                }
                // first check if it's on the lib folder from kui directory
                let mut dk_name = lib.ident.clone();
                if !dk_name.to_lowercase().ends_with(".dk") {
                    dk_name.push_str(".dk");
                }
                let dk_path = exe_dir.join(format!("lib\\kui-global-libs\\{}", dk_name));
                let dk_file = fs::File::open(dk_path);

                if let Ok(file) = dk_file {
                    let archive = ZipArchive::new(file);

                    match archive {
                        Ok(mut contents) => {
                            let mut manifest_str = String::new();
                            {
                                let mut get_manifest = contents.by_name(&"lib.toml");

                                if let Ok(manifest_file) = &mut get_manifest {
                                    let gotten = manifest_file.read_to_string(&mut manifest_str);

                                    if let Err(err) = gotten {
                                        panic!("Problem parsing {}: {}", lib.ident, err);
                                    }
                                } else {
                                    println!("Unable to read manifest of lib: {}", lib.ident);
                                }
                            }

                            let manifest_ret: Result<DkManifest, toml::de::Error> = toml::from_str(&manifest_str);

                            let manifest;
                            if let Ok(m) = manifest_ret {
                                manifest = m;
                            } else if let Err(err) = manifest_ret {
                                eprintln!("{}: {}", lib.ident, err);
                                manifest = DkManifest {
                                    package: Package {
                                        name: String::new(),
                                        authors: String::new(),
                                        version: String::new(),
                                        license: String::new(),
                                        entry: String::new(),
                                        dependencies: vec![]
                                    },
                                };
                            } else {
                                eprintln!("Manifest parsing error");
                                manifest = DkManifest {
                                    package: Package {
                                        name: String::new(),
                                        authors: String::new(),
                                        version: String::new(),
                                        license: String::new(),
                                        entry: String::new(),
                                        dependencies: vec![]
                                    },
                                };
                            }

                            let mut entry = ModCFG::new();
                            {
                                let mut get_entry = contents.by_name(&format!("{}.kk", manifest.package.entry));

                                if let Ok(entry_file) = &mut get_entry {
                                    let mut bytes = vec![];
                                    let gotten = entry_file.read_to_end(&mut bytes);

                                    if let Ok(_) = gotten {
                                        let mut kk_reader = dk_expert::KkReader::new(&bytes);

                                        entry = kk_reader.gen_mod();
                                    } else if let Err(err) = gotten {

                                        panic!("Problem reading {}.kk: {}", manifest.package.entry, err);
                                    } else {
                                        panic!("Problem parsing {}", lib.ident)
                                    }
                                } else {
                                    println!(
                                        "Unable to read lib {} entry module: {}.kk",
                                        lib.ident,
                                        manifest.package.entry
                                    );
                                }
                            }

                            let src = HashMap::new();

                            unit.insert(
                                lib_ident.clone(),
                                NameSpace::Lib {
                                    name: lib_ident,
                                    manifest,
                                    entry,
                                    src,
                                },
                            );
                        }
                        Err(err) => {
                            syntax_errs.push(Error {
                                message: format!("{}", err),
                                span: lib.span,
                            });
                        }
                    }
                } else if let Err(error) = dk_file {
                    // Before emiting error, we should check if the path is provided in command
                    syntax_errs.push(Error {
                        message: format!("{}", error),
                        span: lib.span,
                    });
                } else {
                    panic!("Shouldn't happen")
                }
            }
            _ => (),
        }
    }

    if syntax_errs.is_empty() {
        // SEMANTIC ANALYSES
        let mut analyses = Analyser::new(name.clone(), unit);
        // AST -> Free macro
        let mut expand_macro = AstMacroExpandsion::new(&mut analyses);
        let new_unit = expand_macro.solve_ast();

        analyses.unit = new_unit;

        analyses.analyse();
        // end of semantic analyses
        if analyses.errs.is_empty() {
            // generate MIR/CFG

            // 5. Determine base output name
            // use cli.name if provided, otherwise stem of entry filename
            let base_name = match &cli.name {
                Some(n) => n.clone(),
                None => name.clone(),
                // std::path::PathBuf::from(&cli.filename)
                //     .file_stem()
                //     .unwrap_or_default()
                //     .to_string_lossy()
                //     .to_string(),
            };
            let cfg = borrow_checker::gen_unit_cfg(&mut analyses);
            if cli.target.is_some() {
                // CHECKING IF TARGET IS KUI PACKAGE
                let target = cli.target.clone().unwrap();
                if target == String::from("odù-kúì") || target == String::from("odu-kui") {
                    println!("COMPILING KUI PACKAGE...");
                    println!();

                    let mut version = String::new();
                    println!("\rEnter version: ");
                    io::stdin().read_line(&mut version).unwrap();
                    version.truncate(version.trim_end().len());

                    let mut authors = String::new();
                    println!("\rEnter author names: ");
                    io::stdin().read_line(&mut authors).unwrap();
                    authors.truncate(authors.trim_end().len());

                    let mut license = String::new();
                    println!("\rEnter license: ");
                    io::stdin().read_line(&mut license).unwrap();
                    license.truncate(license.trim_end().len());
                    // io::stdout().flush().unwrap();

                    let from_file = fs::File::create(format!("{}.dk", base_name));

                    match from_file {
                        Ok(file) => {
                            let mut zip = zip::ZipWriter::new(file);

                            let options = SimpleFileOptions::default()
                                .compression_method(CompressionMethod::Deflated);

                            let mut toml_content =
                            format!("[package]\nname = \"{}\"\nversion = \"{}\"\nauthors = \"{}\"\nlicense = \"{}\"\nentry = \"{}\"\n",
                                base_name,
                                version,
                                authors,
                                license,
                                name
                            );

                            let mut dependencies = format!("dependencies = [");
                            for each_resource in &cfg {
                                match each_resource.1 {
                                    Resource::File(mod_cfg) => {
                                        let path = format!("{}.kk", each_resource.0);
                                        let _ = zip.start_file(path, options);

                                        let mut bin = dk_expert::KkWriter::new(&mod_cfg);
                                        bin.gen_from_mod();
                                        let _ = zip.write_all(&bin.buf);
                                    }
                                    Resource::Folder(_) => {}
                                    Resource::Package { .. } => {
                                        // add it to lib.toml dependency section
                                        dependencies
                                            .push_str(&format!("\"{}\", ", each_resource.0));
                                    }
                                }
                            }
                            dependencies.push_str(&format!("]\n"));

                            toml_content.push_str(&dependencies);

                            // add manifest file
                            let _ = zip.start_file("lib.toml", options);
                            let _ = zip.write_all(&toml_content.into_bytes());

                            let _ = zip.finish();

                            println!("packaged to {}.dk", base_name);
                            println!("compilation was successful ✅\n");
                        }
                        Err(err) => {
                            syntax_errs.push(Error {
                                message: format!("{}", err),
                                span: Span {
                                    st: Pos { column: 0, line: 0 },
                                    en: Pos { column: 0, line: 0 },
                                    file: format!(""),
                                },
                            });
                        }
                    }

                    std::process::exit(0);
                }
            }
            // println!("CFGs: {:#?}", _cfg);

            // generate LLVM IR
            let mut each_mod_ir = HashMap::new();

            // walk CFGs
            for each_resource in &cfg {
                match each_resource.1 {
                    Resource::File(file) => {
                        gen_from_mod_cfg(each_resource.0, &cfg, file, &mut each_mod_ir);
                    }
                    Resource::Folder(_) => todo!("fold@name/mod"),
                    Resource::Package { entry, src: _ } => {
                        gen_from_mod_cfg(
                            &format!("lib@{}/entry", each_resource.0),
                            &cfg,
                            entry,
                            &mut each_mod_ir,
                        );
                    }
                }
            }

            let main_key = std::path::PathBuf::from(&cli.filename)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let merged = Codegen::merge_all_modules(each_mod_ir, main_key.clone());

            // unsafe {
            //     LLVMDumpModule(merged);
            // }

            llvm_unsafe(&cli, merged, &base_name, exe_dir);
            // optimize code
            // create an asm file

            // println!("Compilation Unit AST: {:#?}", unit);
            println!("compilation was successful ✅\n");
        } else {
            handle_semantic_err(&analyses.errs, &lines_of_mods);
            std::process::exit(1);
        }
    } else {
        handle_err(syntax_errs, lines_of_mods);
        std::process::exit(1);
    }
}