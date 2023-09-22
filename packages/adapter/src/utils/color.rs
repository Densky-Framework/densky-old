macro_rules! create_color {
    (
    $(#[$meta:meta])*
    $vis:vis enum $name:ident { $($color:ident = [$open:expr, $close:expr] ),* $(,)? }) => {
        mod color_mod {
            $(#[$meta])*
            $vis enum $name {
                // Compose
                Custom(Box<[$name]>),
                $($color),*
            }

            impl $name {
                pub fn get_open(&self) -> String {
                    match self {
                        $name::Custom(compose) => {
                            assert!(compose.len() > 0);
                            // A B C -> AA BB CC
                            //  len       * 2
                            let mut out = Vec::with_capacity(compose.len() * 2); // String::with_capacity(compose.len() * 3 - 1);
                            for c in compose.iter() {
                                out.push(c.get_open());
                            }
                            out.join(";")
                        },
                        $($name::$color => String::from($open)),*
                    }
                }

                pub fn get_close(&self) -> String {
                    match self {
                        $name::Custom(compose) => {
                            assert!(compose.len() > 0);
                            // A B C -> AA BB CC
                            //  len       * 2
                            let mut out = Vec::with_capacity(compose.len() * 2); // String::with_capacity(compose.len() * 3 - 1);
                            for c in compose.iter() {
                                out.push(c.get_close());
                            }
                            out.join(";")
                        },
                        $($name::$color => String::from($close)),*
                    }
                }

                pub fn get_escaped_open(&self) -> String {
                    match self {
                        $name::Custom(compose) => {
                            assert!(compose.len() > 0);
                            // ABC -> AA;BB;CC; -> AA;BB;CC -> \x1b[AA;BB;CCm
                            // len      * 3          - 1             + 3
                            let mut out = String::with_capacity(compose.len() * 3 + 2);
                            out += "\x1b[";
                            for c in compose.iter() {
                                out += &c.get_open();
                                out += ";";
                            }
                            let out_len = out.len();
                            out.replace_range(out_len - 1..out_len, "m");
                            out
                        },
                        $($name::$color => concat!("\x1b[", $open, "m").into()),*
                    }
                }

                pub fn get_escaped_close(&self) -> String {
                    match self {
                        $name::Custom(compose) => {
                            assert!(compose.len() > 0);
                            // ABC -> AA;BB;CC; -> AA;BB;CC -> \x1b[AA;BB;CCm
                            // len      * 3          - 1             + 3
                            let mut out = String::with_capacity(compose.len() * 3 + 2);
                            out += "\x1b[";
                            for c in compose.iter() {
                                out += &c.get_close();
                                out += ";";
                            }
                            let out_len = out.len();
                            out.replace_range(out_len - 1..out_len, "m");
                            out
                        },
                        $($name::$color => concat!("\x1b[", $close, "m").into()),*
                    }
                }
            }
        }

        $vis use color_mod::$name;
    };
}

create_color! {
#[derive(Debug, Clone)]
pub enum Color {
    // Modifiers
    Reset = ["0", "0"],
    Bold = ["1", "22"],
    Dim = ["2", "22"],
    Italic = ["3", "23"],
    Underline = ["4", "24"],
    Overline = ["53", "55"],
    Blink = ["5", "25"],
    Reversed = ["7", "27"],
    Hidden = ["8", "28"],
    Strikethrough = ["9", "29"],

    // Foreground
    FgBlack = ["30", "39"],
    FgRed = ["31", "39"],
    FgGreen = ["32", "39"],
    FgYellow = ["33", "39"],
    FgBlue = ["34", "39"],
    FgMagenta = ["35", "39"],
    FgCyan = ["36", "39"],
    FgWhite = ["37", "39"],

    FgBrightBlack = ["90", "39"],
    FgBrightRed = ["91", "39"],
    FgBrightGreen = ["92", "39"],
    FgBrightYellow = ["93", "39"],
    FgBrightBlue = ["94", "39"],
    FgBrightMagenta = ["95", "39"],
    FgBrightCyan = ["96", "39"],
    FgBrightWhite = ["97", "39"],

    // Background
    BgBlack = ["40", "49"],
    BgRed = ["41", "49"],
    BgGreen = ["42", "49"],
    BgYellow = ["43", "49"],
    BgBlue = ["44", "49"],
    BgMagenta = ["45", "49"],
    BgCyan = ["46", "49"],
    BgWhite = ["47", "49"],

    BgBrightBlack = ["100", "49"],
    BgBrightRed = ["101", "49"],
    BgBrightGreen = ["102", "49"],
    BgBrightYellow = ["103", "49"],
    BgBrightBlue = ["104", "49"],
    BgBrightMagenta = ["105", "49"],
    BgBrightCyan = ["106", "49"],
    BgBrightWhite = ["107", "49"],
}}

impl Color {
    pub fn color(self, str: impl std::fmt::Display) -> String {
        format!(
            "{}{str}{}",
            self.get_escaped_open(),
            self.get_escaped_close()
        )
    }

    pub fn custom(compose: impl Into<Box<[Color]>>) -> Color {
        Color::Custom(compose.into())
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let modifier = self.get_escaped_open();
        f.write_str(&modifier)
    }
}

impl std::ops::BitOr for Color {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        // String::str
        match self {
            Color::Custom(compose) => Self::Custom([compose, Box::new([rhs])].concat().into()),
            _ => Self::Custom([[self], [rhs]].concat().into()),
        }
    }
}

impl std::ops::Add for Color {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        use std::ops::BitOr;
        self.bitor(rhs)
    }
}
