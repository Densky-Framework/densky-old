pub const PREPARE_PREFIX: &str = "__matcher_prepare";
pub const SERIAL_PREFIX: &str = "__matcher_serial";
pub const MATCHER_PREFIX: &str = "__matcher_matcher_";

#[derive(Debug, Clone)]
pub enum UrlMatcherSegment {
    Static(String),
    Var(String),
}

impl UrlMatcherSegment {
    pub fn is_static(&self) -> bool {
        match self {
            Self::Static(_) => true,
            _ => false,
        }
    }

    pub fn is_var(&self) -> bool {
        match self {
            Self::Var(_) => true,
            _ => false,
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{ raw: \"{}\", isVar: {}{} }}",
            match self {
                Self::Static(raw) => raw.clone(),
                Self::Var(varname) => format!("${}", varname),
            },
            self.is_var(),
            match self {
                Self::Static(_) => "".to_owned(),
                Self::Var(varname) => format!(", varname: \"{}\"", varname),
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct UrlMatcher {
    pub url: String,
    pub segments: Vec<UrlMatcherSegment>,
    pub has_variables: bool,
}

impl UrlMatcher {
    pub fn new(url: String) -> UrlMatcher {
        let mut has_variables = false;
        let segments: Vec<UrlMatcherSegment> = url
            .split('/')
            .map(|segment| {
                if segment.len() != 0 && &segment[0..1] == "$" {
                    has_variables = true;
                    UrlMatcherSegment::Var(segment[1..].to_string())
                } else {
                    UrlMatcherSegment::Static(segment.to_string())
                }
            })
            .collect();

        UrlMatcher {
            url,
            segments,
            has_variables,
        }
    }

    pub fn exact_decl<V>(&self, req: V, has_children: bool) -> String
    where
        V: AsRef<str>,
    {
        let req = req.as_ref();
        if has_children {
            format!("{req}.__accumulator__.segments.length === 0")
        } else {
            if self.has_variables {
                format!(
                    "{MATCHER_PREFIX}EXACT({req}.__accumulator__.segments, {req}.params, new Map())",
                )
            } else {
                format!("{req}.__accumulator__.path === '{}'", self.url)
            }
        }
    }

    pub fn exact_inline<V>(&self, req: V, has_children: bool) -> String
    where
        V: AsRef<str>,
    {
        let req = req.as_ref();
        if has_children {
            format!("{req}.__accumulator__.segments.length === 0")
        } else {
            if self.has_variables {
                format!(
                "$_Densky_Runtime_$.matcherExact({}, {req}.__accumulator__.segments, {req}.params, new Map())",
                self.serial_inline()
            )
            } else {
                format!("{req}.__accumulator__.path === '{}'", self.url)
            }
        }
    }

    pub fn start_decl<V>(&self, req: V) -> String
    where
        V: AsRef<str>,
    {
        let req = req.as_ref();
        if self.has_variables {
            format!(
                "{1}START({0}.__accumulator__.segments, {0}.params, new Map())",
                req, MATCHER_PREFIX,
            )
        } else {
            format!("{}.__accumulator__.path.startsWith('{}')", req, self.url)
        }
    }

    pub fn start_inline<V>(&self, req: V) -> String
    where
        V: AsRef<str>,
    {
        let req = req.as_ref();
        if self.has_variables {
            format!(
                "$_Densky_Runtime_$.matcherStart({}, {req}.__accumulator__.segments, {req}.params, new Map())",
                self.serial_inline()
            )
        } else {
            format!("{req}.__accumulator__.path.startsWith('{}')", self.url)
        }
    }

    pub fn update_decl<V>(&self, val: V) -> String
    where
        V: AsRef<str>,
    {
        let accumulator = format!("{}.__accumulator__", val.as_ref());
        let corrector = if self.url == "/" { 1 } else { 0 };
        format!(
            "// @ts-ignore\n{0}.segments = {0}.segments.slice({1});
// @ts-ignore\n{0}.path = {0}.segments.join(\"/\");",
            accumulator,
            self.segments.len() - corrector
        )
    }

    pub fn serial_decl(&self) -> String {
        if !self.has_variables {
            "".to_string()
        } else {
            format!(
                "{}\n{}\n{}",
                format!("const {} = {};", SERIAL_PREFIX, self.serial_inline()),
                format!(
                    "// @ts-ignore\nconst {}EXACT = $_Densky_Runtime_$.matcherExact({});",
                    MATCHER_PREFIX, SERIAL_PREFIX
                ),
                format!(
                    "// @ts-ignore\nconst {}START = $_Densky_Runtime_$.matcherStart({});",
                    MATCHER_PREFIX, SERIAL_PREFIX
                )
            )
        }
    }

    /// Transform the struct to json
    pub fn serial_inline(&self) -> String {
        let mut serialized = "[".to_string();
        for segment in &self.segments {
            serialized += &segment.to_json();
            serialized += ",";
        }
        serialized.pop();
        serialized += "]";
        serialized
    }
}
