// use regex::Regex;

// use std::process::exit;

// use std::{iter::{Enumerate, Peekable}, str::Chars, u32};

use core::fmt;
use std::fmt::{Display, Formatter};
// #[derive(Clone, Eq, PartialEq, Debug)]
// pub struct PosInByte (pub u32);
use std::iter::{Enumerate, Peekable};
use std::str::Chars;

use crate::parser::{self, Error};

#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Debug)]
pub struct Pos {
    pub column: usize, //The start offset is stored in a byte
    pub line: usize,   //The end offset stored in byte
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Debug)]
pub struct Span {
    pub st: Pos, //The start offset is stored in a byte
    pub en: Pos, //The end offset stored in byte
    pub file: String,
}

// #[derive(Clone, Eq, PartialEq, Debug)]
// pub struct LineSpan {
//     pub st: usize, //The start offset is stored in a byte
//     pub en: usize, //The end offset stored in byte
// }

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub struct FullTok {
    pub kind: Token,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum TruthVal {
    True,
    Unknown,
    False,
    Both,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Token::Identifier(i) => write!(f, "{}", i),
            Token::StringLiteral(s) => write!(f, "{}", s),
            Token::Foo => write!(f, "Foo"),
            _ => write!(f, "<Token>"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum Token {
    // Keywords
    Use,         // lo
    Take,        // mú
    Worker,      // òṣìṣẹ; known as decorators in python
    Macro,       // from tú (to desolve)
    Static,      // adágún
    NameAs,       // pèéní
    As,         // bíi
    Global,      // kárí
    Function,    // itú
    Async,       // tàṣé
    Await,       // dúróde
    Start,       // bẹ̀rẹ̀
    Target,     // àfojúsùn
    FlowThrough, // ré
    Struct,      // ìrísí
    Enum,        // apẹ̀rẹ̀
    If,          // tí
    Else,        // àìjẹ́bẹ̀
    Loop,        // pòrì
    While,       // níwọ̀n ọn bí / níwọ̀nọnbí
    Do,          // ṣe
    To,          // sí
    Try,         // gbìyànjú
    Catch,       // hán
    UseOutside,  // síta
    Match,       // níbàámu
    Case,        // wo -> Check
    For,         // fún
    Continue,    // tẹ̀síwájú
    Return,      // padà
    RetVal,      // jọ̀wọ́
    Break,       // já
    Public,      // gbangba
    Private,     // àdáni
    Mut,         // tóyí
    SelfKw,      // alára
    From,        // látinú
    Bridge,      // afárá
    Create,      // ṣẹ̀dá
    Extern,      // látòde
    Inherit,     // jogún
    Interpret,   // túmọ̀; like comptime in Zig
    Trait,       // àfijọ
    Have,        //ní
    Move,        // kó
    Default,     // àrọ́sí
    Peeking,     // máa_ṣa

    NewLine,

    Foo,

    // Variable
    Var,    // kí
    Assign, // yí

    // Constructor
    Constructor, // ::

    // Type hints
    TypeHint(String),
    Void, // òfìfo

    // Literals
    Number(String),
    StringLiteral(String),
    LineDoc(String),
    DocLines(Vec<String>),
    MultiLineString(String),
    CharLiteral(char),
    ArityLiteral(TruthVal), // true, maybe, and false; boolean is a subset
    Identifier(String),
    MacroCall(String), // #ident
    MacroVar(String),  // %ident

    // Operators & symbols
    Plus,
    Minus,
    Star,
    Slash,
    BackSlash,
    Div,
    Mul,
    Percent,
    Equal,
    NotEqual,
    NotLess,
    NotGreater,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Colon,         // :
    ForwardArrow,  // ->
    BackwardArrow, // <-
    FatArrow,      // =>
    Ampersand,     // &
    Pipe,          // |
    At,            // @
    DColon,        // ::
    NumSign,       // #
    QuestMark,     // ?
    Exclam,
    Tilde, // ~ is Tilde
    DLess,
    DGreater,
    DPipe,
    DAmpersand,
    DDot,
    TDot,
    DPercent,
    DMinus,
    DPlus, // D stands for Double

    // Punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Dot,
    Lt,
    Gt,
    NudDot, // NudDot is basically Dot, but for nud operator for .{}

    // Special
    Unknown(char),
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Lexed {
    pub tokens: Vec<FullTok>,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LexedSelf<'a> {
    file: String,
    pub tokens: Vec<FullTok>,
    pub lines: Vec<String>,
    pub line_num: usize, // helper field
    pub line_column: usize,
    pub chars: Peekable<Enumerate<Chars<'a>>>,

    pub errs: Vec<parser::Error>
}

impl<'a> LexedSelf<'a> {
    fn handle_lines(&mut self) {
        // this is called every time there is a new line character i.e \n

        self.tokens.push(FullTok {
            kind: Token::NewLine,
            span: Span {
                st: Pos {
                    column: self.line_column,
                    line: self.line_num,
                },
                en: Pos {
                    column: self.line_column + 1,
                    line: self.line_num,
                },
                file: self.file.clone()
            },
        });
        self.line_num += 1;
        self.line_column = 0; // resets
    }
    pub fn new(input: &'a String, file: String) -> Self {
        let lines: Vec<String> = input.lines().map(String::from).collect();

        Self {
            file,
            tokens: vec![],
            lines,
            line_num: 1,
            line_column: 0,
            chars: input.chars().enumerate().peekable(),
            errs: vec![]
        }
    }

    fn next(&mut self) -> Option<(usize, char)> {
        self.line_column += 1; //
        self.chars.next()
    }

    fn peek(&mut self) -> Option<&(usize, char)> {
        self.chars.peek()
    }

    pub fn lex(&mut self) -> Lexed {
        let _offset = 0;

        while let Some(&(_, ch)) = self.peek() {
            let st = Pos {
                column: self.line_column,
                line: self.line_num,
            };
            // let mut start = i;
            // let _ = &start;

            match ch {
                // New Line Character
                c if c == '\n' => {
                    self.next();
                    self.handle_lines();
                }

                // Identifiers, keywords, some typehints
                c if c.is_alphabetic()
                    || c == '_'
                    || c == '\u{300}'
                    || c == '\u{301}'
                    || c as u32 > 127 =>
                {
                    // c as u32 > 127 allows Unicode characters with accents
                    let mut ident = String::new();

                    while let Some(&(_i2, c2)) = self.peek() {
                        // i2, c2 Destruction...
                        // if c2 == '\n' {
                        //     self.handle_lines(i);
                        // }
                        if c2.is_alphanumeric() || c2 == '_' || c2 as u32 > 127 {
                            ident.push(c2);
                            self.next();
                        } else {
                            break;
                        }
                    }
                    let en = Pos {
                        column: st.column + (ident.chars().count()),
                        line: self.line_num,
                    };

                    let d_span = Span { st, en, file: self.file.clone() };

                    let tok = match ident.as_str() {
                        // Keywords
                        "lo" => FullTok {
                            kind: Token::Use,
                            span: d_span,
                        },
                        "mú" => FullTok {
                            kind: Token::Take,
                            span: d_span,
                        },
                        "òṣìṣẹ́" => FullTok {
                            kind: Token::Worker,
                            span: d_span,
                        },
                        "adágún" => FullTok {
                            kind: Token::Static,
                            span: d_span,
                        },
                        "àgánkọ" => FullTok {
                            kind: Token::Macro,
                            span: d_span,
                        },
                        "pèéní" => FullTok {
                            kind: Token::NameAs,
                            span: d_span,
                        },
                        "bíi" => FullTok {
                            kind: Token::As,
                            span: d_span,
                        },
                        "kárí" => FullTok {
                            kind: Token::Global,
                            span: d_span,
                        },
                        "àfojúsùn" => FullTok {
                            kind: Token::Target,
                            span: d_span,
                        },
                        "dúróde" => FullTok {
                            kind: Token::Await,
                            span: d_span,
                        },
                        "bẹ̀rẹ̀" => FullTok {
                            kind: Token::Start,
                            span: d_span,
                        },
                        "tàṣé" => FullTok {
                            kind: Token::Async,
                            span: d_span,
                        },
                        "iṣẹ́" => FullTok {
                            kind: Token::Function,
                            span: d_span,
                        },
                        "ré" => FullTok {
                            kind: Token::FlowThrough,
                            span: d_span,
                        },
                        "ìrísí" => FullTok {
                            kind: Token::Struct,
                            span: d_span,
                        },
                        "apẹ̀rẹ̀" => FullTok {
                            kind: Token::Enum,
                            span: d_span,
                        },
                        "tí" => FullTok {
                            kind: Token::If,
                            span: d_span,
                        },
                        "àìjẹ́bẹ̀" => FullTok {
                            kind: Token::Else,
                            span: d_span,
                        },
                        "tẹ̀síwájú" => FullTok {
                            kind: Token::Continue,
                            span: d_span,
                        },
                        "pòrì" => FullTok {
                            kind: Token::Loop,
                            span: d_span,
                        },
                        "níwọ̀bí" => FullTok {
                            kind: Token::While,
                            span: d_span,
                        },
                        "ṣe" => FullTok {
                            kind: Token::Do,
                            span: d_span,
                        },
                        "níbàámu" => FullTok {
                            kind: Token::Match,
                            span: d_span,
                        },
                        "wo" => FullTok {
                            kind: Token::Case,
                            span: d_span,
                        },
                        "fún" => FullTok {
                            kind: Token::For,
                            span: d_span,
                        },
                        "já" => FullTok {
                            kind: Token::Break,
                            span: d_span,
                        },
                        "gbìyànjú" => FullTok {
                            kind: Token::Try,
                            span: d_span,
                        },
                        "hán" => FullTok {
                            kind: Token::Catch,
                            span: d_span,
                        },
                        "síta" => FullTok {
                            kind: Token::UseOutside,
                            span: d_span,
                        },
                        "látòde" => FullTok {
                            kind: Token::Extern,
                            span: d_span,
                        },
                        "gbangba" => FullTok {
                            kind: Token::Public,
                            span: d_span,
                        },
                        "òwun" => FullTok {
                            kind: Token::SelfKw,
                            span: d_span,
                        },
                        "tóyí" => FullTok {
                            kind: Token::Mut,
                            span: d_span,
                        },
                        "kí" => FullTok {
                            kind: Token::Var,
                            span: d_span,
                        },
                        "yí" => FullTok {
                            kind: Token::Assign,
                            span: d_span,
                        },
                        "látinú" => FullTok {
                            kind: Token::From,
                            span: d_span,
                        },
                        "afárá" => FullTok {
                            kind: Token::Bridge,
                            span: d_span,
                        },
                        "àfijọ" => FullTok {
                            kind: Token::Trait,
                            span: d_span,
                        },
                        "padà" => FullTok {
                            kind: Token::Return,
                            span: d_span,
                        },
                        "ju" => FullTok {
                            kind: Token::RetVal,
                            span: d_span,
                        },
                        "túmọ̀" => FullTok {
                            kind: Token::Interpret,
                            span: d_span,
                        },
                        "jogún" => FullTok {
                            kind: Token::Inherit,
                            span: d_span,
                        },
                        "ní" => FullTok {
                            kind: Token::Have,
                            span: d_span,
                        },
                        "sí" => FullTok {
                            kind: Token::To,
                            span: d_span,
                        },
                        "kó" => FullTok {
                            kind: Token::Move,
                            span: d_span,
                        },
                        "máa_ṣa" => FullTok {
                            kind: Token::Peeking,
                            span: d_span,
                        },

                        "òfìfo" => FullTok {
                            kind: Token::Void,
                            span: d_span,
                        },

                        // Bool Types
                        "òtítọ́" => FullTok {
                            kind: Token::ArityLiteral(TruthVal::True),
                            span: d_span,
                        },
                        "irọ́" => FullTok {
                            kind: Token::ArityLiteral(TruthVal::False),
                            span: d_span,
                        },
                        "bóyá" => FullTok {
                            kind: Token::ArityLiteral(TruthVal::Unknown),
                            span: d_span,
                        },
                        "agbede" => FullTok {
                            kind: Token::ArityLiteral(TruthVal::Both),
                            span: d_span,
                        },
                        "àrọ́sí" => FullTok {
                            kind: Token::Default,
                            span: d_span,
                        },

                        // Type hints
                        "àtó_d" | "àtó_p" | "àtó_b" | "wóró" | "ọsán" | "ǹjẹ́" | "ibùta"
                        | "ibùrin" | "àṣàyàn" | "àyọrísí" | "òpin" => FullTok {
                            kind: Token::TypeHint(ident),
                            span: d_span,
                        },

                        // Fallback = identifier
                        _ => FullTok {
                            kind: Token::Identifier(ident),
                            span: d_span,
                        },
                    };
                    self.tokens.push(tok);
                }

                // Whitespace
                c if c.is_whitespace() => {
                    self.next();
                }

                // Numbers
                c if c.is_ascii_digit() => {
                    let mut num = String::new();
                    if c == '0' {
                        num.push(c); // for the first digit
                        self.next();
                        if let Some(&(_, 'x')) = self.peek() {
                            num.push('x'); // for the second digit
                            self.next();
                        } else if let Some(&(_, 'b')) = self.peek() {
                            num.push('b'); // for the second digit
                            self.next();
                        }
                    }
                    while let Some(&(_i2, c2)) = self.peek() {
                        // if c2 == '\n' {
                        //     self.handle_lines(i);
                        // }

                        if c2.is_ascii_digit() {
                            num.push(c2);
                            self.next();
                        } else {
                            let combined = num.clone() + &c2.to_string();
                            // let end = start + num.chars().count(); // Use chars().count()
                            let d_span = Span {
                                st: st.clone(),
                                en: Pos {
                                    column: st.column + combined.chars().count(),
                                    line: self.line_num,
                                },
                                file: self.file.clone()
                            };

                            match combined.as_str() {
                                "8d" | "16d" | "32d" | "64d" | "128d"
                                | "8p" | "16p" | "32p" | "64p" | "128p"//  unsigned
                                | "8b" | "16b" | "32b" | "64b" | "128b" => {
                                    self.tokens.push(
                                        FullTok {
                                            kind: Token::TypeHint(combined),
                                            span: d_span
                                        }
                                    );
                                    self.next();
                                    break;  // stop loop since num is gone
                                }
                                _ => {
                                    if c2.is_alphabetic() {
                                        self.errs.push(
                                            Error {
                                                message: format!("Tokenizer error: Unsupported identifier format at line: {}\n", self.line_num),
                                                span: d_span.clone()
                                            }
                                        );
                                    }
                                    self.tokens.push(FullTok { kind: Token::Number(num), span: d_span }); //  Return it as a number then
                                    break;
                                }
                            }
                        }
                    }
                }

                // Operators and punctuation
                '+' => {
                    self.next();
                    if let Some(&(_, '+')) = self.peek() {
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 2,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.next();
                        self.tokens.push(FullTok {
                            kind: Token::DPlus,
                            span: d_span,
                        });
                    } else {
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 1,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::Plus,
                            span: d_span,
                        });
                    }
                }
                '-' => {
                    self.next();
                    if let Some(&(_, '>')) = self.peek() {
                        self.next();
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 2,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::ForwardArrow,
                            span: d_span,
                        });
                    } else if let Some(&(_, '-')) = self.peek() {
                        self.next();
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 2,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::DMinus,
                            span: d_span,
                        });
                    } else {
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 1,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::Minus,
                            span: d_span,
                        });
                    }
                }

                '@' => {
                    self.next();
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + 1,
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::At,
                        span: d_span,
                    });
                }

                '*' => {
                    self.next();
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + 1,
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::Star,
                        span: d_span,
                    });
                }

                '×' => {
                    self.next();
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + 1,
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::Mul,
                        span: d_span,
                    });
                }

                '~' => {
                    self.next();
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + 1,
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::Tilde,
                        span: d_span,
                    });
                }

                '/' => {
                    let mut len = 1;
                    let _ = len;
                    self.next();
                    match self.peek() {
                        Some(&(_, '/')) => {
                            self.next();
                            let _ = len;
                            len += 1;
                            while let Some(&(_i2, c2)) = self.peek() {
                                if c2 == '\n' {
                                    self.handle_lines();
                                    self.next();
                                    break;
                                }
                                self.next();
                                len += 1;
                            }
                            let _ = len;
                        }

                        Some(&(_, '*')) => {
                            self.next();
                            let _ = len;
                            len += 1;
                            while let Some(&(_i2, c2)) = self.peek() {
                                if c2 == '\n' {
                                    self.handle_lines();
                                }
                                if c2 == '*' {
                                    self.next();
                                    len += 1;
                                    if let Some(&(_, '/')) = self.peek() {
                                        self.next();
                                        break;
                                    }
                                }
                                self.next();
                                len += 1;
                            }
                            let _ = len;
                        }
                        _ => {
                            let d_span = Span {
                                st: st.clone(),
                                en: Pos {
                                    column: st.column + 1,
                                    line: self.line_num,
                                },
                                file: self.file.clone()
                            };
                            self.tokens.push(FullTok {
                                kind: Token::Slash,
                                span: d_span,
                            });
                        }
                    }
                }
                '÷' => {
                    self.next();
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + 1,
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::Div,
                        span: d_span,
                    });
                }
                '\\' => {
                    self.next();
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + 1,
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::BackSlash,
                        span: d_span,
                    });
                }
                '%' => {
                    self.next();
                    match self.peek() {
                        Some(&(_, '%')) => {
                            let d_span = Span {
                                st: st.clone(),
                                en: Pos {
                                    column: st.column + 2,
                                    line: self.line_num,
                                },
                                file: self.file.clone()
                            };
                            self.tokens.push(FullTok {
                                kind: Token::DPercent,
                                span: d_span,
                            });
                            self.next();
                        }
                        _ => {
                            let d_span = Span {
                                st: st.clone(),
                                en: Pos {
                                    column: st.column + 1,
                                    line: self.line_num,
                                },
                                file: self.file.clone()
                            };
                            self.tokens.push(FullTok {
                                kind: Token::Percent,
                                span: d_span,
                            });
                        }
                    }
                }
                '=' => {
                    self.next();
                    match self.peek() {
                        Some(&(_, '>')) => {
                            let d_span = Span {
                                st: st.clone(),
                                en: Pos {
                                    column: st.column + 2,
                                    line: self.line_num,
                                },
                                file: self.file.clone()
                            };
                            self.tokens.push(FullTok {
                                kind: Token::FatArrow,
                                span: d_span,
                            });
                            self.next();
                        }
                        _ => {
                            let d_span = Span {
                                st: st.clone(),
                                en: Pos {
                                    column: st.column + 1,
                                    line: self.line_num,
                                },
                                file: self.file.clone()
                            };
                            self.tokens.push(FullTok {
                                kind: Token::Equal,
                                span: d_span,
                            });
                        }
                    }
                }
                '!' => {
                    self.next();
                    if let Some(&(_, '=')) = self.peek() {
                        self.next();
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 2,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::NotEqual,
                            span: d_span,
                        });
                    }
                    // else if let Some(&(_, '<')) = self.peek() {
                    //     self.line_string.push('<');
                    //     self.next();
                    //     self.tokens.push(FullTok {
                    //         kind: Token::NotLess,
                    //         span: Span {
                    //             st: Pos {
                    //                 column: self.line_column,
                    //                 line: self.line_num,
                    //             },
                    //             en: Pos {
                    //                 column: self.line_column + 1,
                    //                 line: self.line_num,
                    //             },
                    //         },
                    //     });
                    // } else if let Some(&(_, '>')) = self.peek() {
                    //     self.line_string.push('>');
                    //     self.next();
                    //     self.tokens.push(FullTok {
                    //         kind: Token::NotGreater,
                    //         span: Span {
                    //             st: Pos {
                    //             column: self.line_column,
                    //             line: self.line_num,
                    //         },
                    //         en: Pos {
                    //             column: self.line_column + 1,
                    //             line: self.line_num,
                    //         },
                    //         },
                    //     });
                    // }
                    else {
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 1,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::Exclam,
                            span: d_span,
                        });
                    }
                }
                '<' => {
                    self.next();
                    if let Some(&(_, '=')) = self.peek() {
                        self.next();
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 2,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::LessEq,
                            span: d_span,
                        });
                    } else if let Some(&(_, '-')) = self.peek() {
                        self.next();
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 2,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::BackwardArrow,
                            span: d_span,
                        });
                    } else if let Some(&(_, '<')) = self.peek() {
                        self.next();
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 2,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::DLess,
                            span: d_span,
                        });
                    } else {
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 1,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::Less,
                            span: d_span,
                        });
                    }
                }
                '>' => {
                    self.next();
                    if let Some(&(_, '=')) = self.peek() {
                        self.next();
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 2,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::GreaterEq,
                            span: d_span,
                        });
                    } else if let Some(&(_, '>')) = self.peek() {
                        self.next();
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 2,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::DGreater,
                            span: d_span,
                        });
                    } else {
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 1,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::Greater,
                            span: d_span,
                        });
                    }
                }
                '&' => {
                    self.next();
                    if let Some(&(_, '&')) = self.peek() {
                        self.next();
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 2,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::DAmpersand,
                            span: d_span,
                        });
                    } else {
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 1,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::Ampersand,
                            span: d_span,
                        });
                    }
                }

                '|' => {
                    self.next();
                    if let Some(&(_, '|')) = self.peek() {
                        self.next();
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 2,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::DPipe,
                            span: d_span,
                        });
                    } else {
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 1,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::Pipe,
                            span: d_span,
                        });
                    }
                }

                '?' => {
                    self.next();
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + 1,
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::QuestMark,
                        span: d_span,
                    });
                }

                ':' => {
                    self.next();
                    if let Some(&(_, ':')) = self.peek() {
                        self.next();
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 2,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::DColon,
                            span: d_span,
                        });
                    } else {
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 1,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::Colon,
                            span: d_span,
                        });
                    }
                }

                '(' => {
                    self.next();
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + 1,
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::LParen,
                        span: d_span,
                    });
                }

                ')' => {
                    self.next();
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + 1,
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::RParen,
                        span: d_span,
                    });
                }

                '{' => {
                    self.next();
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + 1,
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::LBrace,
                        span: d_span,
                    });
                }

                '}' => {
                    self.next();
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + 1,
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::RBrace,
                        span: d_span,
                    });
                }

                '[' => {
                    self.next();
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + 1,
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::LBracket,
                        span: d_span,
                    });
                }

                ']' => {
                    self.next();
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + 1,
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::RBracket,
                        span: d_span,
                    });
                }

                ',' => {
                    self.next();
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + 1,
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::Comma,
                        span: d_span,
                    });
                }

                ';' => {
                    self.next();
                    let mut line_doc = String::new();
                    let mut len = 0;
                    while let Some(&(_i2, c2)) = self.peek() {
                        if c2 == '\n' {
                            self.handle_lines();
                            self.next();
                            break;
                        }
                        line_doc.push(c2);
                        self.next();
                        len += 1;
                    }
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + len,
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::LineDoc(line_doc),
                        span: d_span,
                    });
                }

                '#' => {
                    self.next();
                    match self.peek() {
                        Some(&(_, ' ')) => {
                            self.next();
                            let d_span = Span {
                                st: st.clone(),
                                en: Pos {
                                    column: st.column + 1,
                                    line: self.line_num,
                                },
                                file: self.file.clone()
                            };
                            self.tokens.push(FullTok {
                                kind: Token::NumSign,
                                span: d_span,
                            });
                        }
                        Some(&(_, a)) => {
                            if a.is_alphabetic() || a == '_' || a == '\u{300}' || a == '\u{301}' {
                                // then it's a macro
                                let mut ident = String::new();

                                ident.push(a);
                                self.next();
                                while let Some(&(_i2, c2)) = self.peek() {
                                    // i2, c2 Destruction...
                                    if c2.is_alphanumeric()
                                        || c2 == '_'
                                        || a == '\u{300}'
                                        || a == '\u{301}'
                                    {
                                        ident.push(c2);
                                        self.next();
                                    } else {
                                        break;
                                    }
                                }
                                let d_span = Span {
                                    st: st.clone(),
                                    en: Pos {
                                        column: st.column + ident.chars().count(),
                                        line: self.line_num,
                                    },
                                    file: self.file.clone()
                                };
                                self.tokens.push(FullTok {
                                    kind: Token::MacroCall(ident.clone()),
                                    span: d_span,
                                });
                            } else {
                                let d_span = Span {
                                    st: st.clone(),
                                    en: Pos {
                                        column: st.column + 1,
                                        line: self.line_num,
                                    },
                                    file: self.file.clone()
                                };
                                self.tokens.push(FullTok {
                                    kind: Token::NumSign,
                                    span: d_span,
                                });
                            }
                        }
                        None => {
                            self.tokens.push(FullTok {
                                kind: Token::EOF,
                                span: Span {
                                    st: Pos {
                                        column: self.line_column,
                                        line: self.line_num,
                                    },
                                    en: Pos {
                                        column: self.line_column + 1,
                                        line: self.line_num,
                                    },
                                    file: self.file.clone()
                                },
                            });
                        }
                    }
                }
                '.' => {
                    self.next();
                    if let Some(&(_, '.')) = self.peek() {
                        self.next();
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 2,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        if let Some(&(_, '.')) = self.peek() {
                            self.next();
                            let d_span = Span {
                                st: st.clone(),
                                en: Pos {
                                    column: st.column + 2,
                                    line: self.line_num,
                                },
                                file: self.file.clone()
                            };
                            self.tokens.push(FullTok {
                                kind: Token::TDot,
                                span: d_span,
                            });
                        } else {
                            self.tokens.push(FullTok {
                                kind: Token::DDot,
                                span: d_span,
                            });
                        }
                    } else {
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 1,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::Dot,
                            span: d_span,
                        });
                    }
                }

                // Strings
                '"' => {
                    let mut span_count = 0;
                    self.next();
                    span_count += 1;
                    if let Some(&(_i2, c2)) = self.peek() {
                        // self.next();
                        if c2 == '"' {
                            self.next();
                            span_count += 1;
                            if let Some(&(_i3, c3)) = self.peek() {
                                if c3 == '"' {
                                    self.next();
                                    span_count += 1;
                                    // """c4
                                    //    ^^
                                    let mut qcount = 0;
                                    let mut doc_lines: Vec<String> = vec![String::new()];
                                    while let Some(&(_i4, c4)) = self.peek() {
                                        self.next();
                                        span_count += 1;
                                        if c4 == '"' {
                                            // 1
                                            self.next();
                                            span_count += 1;
                                            let i = doc_lines.len() - 1;
                                            if qcount == 1 {
                                                doc_lines[i].pop();
                                                doc_lines[i].pop();
                                                break;
                                            } else {
                                                qcount += 1;
                                                doc_lines[i].push(c4);
                                            }
                                        } else if c4 == '\n' {
                                            self.handle_lines();
                                            doc_lines.push(String::new());
                                        } else {
                                            let i = doc_lines.len() - 1;
                                            doc_lines[i].push(c4);
                                        }
                                    }
                                    let d_span = Span {
                                        st: st.clone(),
                                        en: Pos {
                                            column: st.column + span_count,
                                            line: self.line_num,
                                        },
                                        file: self.file.clone()
                                    };
                                    self.tokens.push(FullTok {
                                        kind: Token::DocLines(doc_lines),
                                        span: d_span,
                                    });
                                } else {
                                    let d_span = Span {
                                        st: st.clone(),
                                        en: Pos {
                                            column: st.column + 1,
                                            line: self.line_num,
                                        },
                                        file: self.file.clone()
                                    };
                                    self.tokens.push(FullTok {
                                        kind: Token::StringLiteral(String::new()),
                                        span: d_span,
                                    });
                                }
                            }
                        } else if c2 == '\n' {
                            self.handle_lines();
                            let d_span = Span {
                                st: st.clone(),
                                en: Pos {
                                    column: st.column + 1,
                                    line: self.line_num,
                                },
                                file: self.file.clone()
                            };
                            self.errs.push(
                                Error {
                                    message: format!("Handle error: Use Multiling instead\n"),
                                    span: d_span
                                }
                            );
                        } else {
                            let mut s = String::new();
                            while let Some(&(_i2, c2)) = self.peek() {
                                self.next();
                                if c2 == '"' {
                                    break;
                                } else if c2 == '\n' {
                                    self.handle_lines();
                                    let d_span = Span {
                                        st: st.clone(),
                                        en: Pos {
                                            column: st.column + s.chars().count(),
                                            line: self.line_num,
                                        },
                                        file: self.file.clone()
                                    };
                                    self.errs.push(
                                        Error {
                                            message: format!("Handle error: Use Multiling instead\n"),
                                            span: d_span
                                        }
                                    );
                                }
                                s.push(c2);
                            }
                            let d_span = Span {
                                st: st.clone(),
                                en: Pos {
                                    column: st.column + s.chars().count(),
                                    line: self.line_num,
                                },
                                file: self.file.clone()
                            };
                            self.tokens.push(FullTok {
                                kind: Token::StringLiteral(s),
                                span: d_span,
                            });
                        }
                    }
                }

                // Multiline string '''
                '`' => {
                    self.next();
                    let mut s = String::new();

                    while let Some(&(_i2, c2)) = self.peek() {
                        if c2 == '\n' {
                            self.handle_lines();
                        }
                        self.next();
                        if c2 == '`' {
                            break;
                        }
                        s.push(c2);
                    }
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + s.chars().count(),
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::MultiLineString(s),
                        span: d_span,
                    });
                }

                // Chars
                '\'' => {
                    self.next();
                    let mut s = String::new();
                    while let Some(&(_i2, c2)) = self.peek() {
                        if c2 == '\n' {
                            self.handle_lines();
                        }
                        self.next();
                        if c2 == '\'' {
                            break;
                        } else if c2 == '\n' {
                            let d_span = Span {
                                st: st.clone(),
                                en: Pos {
                                    column: st.column + s.len(),
                                    line: self.line_num,
                                },
                                file: self.file.clone()
                            };
                            self.errs.push(
                                Error {
                                    message: format!("Handle error: Use Multiling instead of '\n"),
                                    span: d_span
                                }
                            );
                        }
                        s.push(c2);
                    }
                    if s.len() == 1 {
                        //  Check if it's really a char
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + 2,
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.tokens.push(FullTok {
                            kind: Token::CharLiteral(s.chars().next().unwrap()),
                            span: d_span,
                        });
                    } else {
                        //  Print an error message
                        let d_span = Span {
                            st: st.clone(),
                            en: Pos {
                                column: st.column + s.len(),
                                line: self.line_num,
                            },
                            file: self.file.clone()
                        };
                        self.errs.push(
                            Error {
                                message: format!("Handle error: Oníwà ni e ń retí níbí yìí\n"),
                                span: d_span
                            }
                        );
                    }
                }

                _ => {
                    let d_span = Span {
                        st: st.clone(),
                        en: Pos {
                            column: st.column + 1,
                            line: self.line_num,
                        },
                        file: self.file.clone()
                    };
                    self.tokens.push(FullTok {
                        kind: Token::Unknown(ch),
                        span: d_span,
                    });
                    self.next();
                }
            }
        }
        Lexed {
            tokens: self.tokens.clone(),
            lines: self.lines.clone(),
        }
    }
}
