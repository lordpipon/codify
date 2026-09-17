use std::path::Path;
use std::sync::{RwLock, OnceLock};

use serde::{Deserialize, Serialize};

/// A language contributed by an installed extension. Data-driven so that the
/// tokenizer can handle languages that aren't compiled into the binary.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynamicLang {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub types: Vec<String>,
    #[serde(default)]
    pub line_comment: Option<String>,
    #[serde(default)]
    pub block_comment: Option<Vec<String>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Html,
    Css,
    Json,
    Markdown,
    Shell,
    Toml,
    C,
    Cpp,
    CSharp,
    Java,
    Go,
    Ruby,
    Php,
    Swift,
    Kotlin,
    Lua,
    Perl,
    R,
    Scala,
    Haskell,
    Elixir,
    Dart,
    Sql,
    Yaml,
    Xml,
    Svelte,
    Vue,
    Dockerfile,
    Makefile,
    Plain,
    Custom,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokenKind {
    Plain,
    Keyword,
    Type,
    Function,
    Constant,
    Str,
    Comment,
    Number,
    Operator,
    Punctuation,
    Attribute,
}

#[derive(Clone, Copy, Debug)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub kind: TokenKind,
}

impl Token {
    fn new(start: usize, end: usize, kind: TokenKind) -> Self {
        Self { start, end, kind }
    }
}

#[derive(Clone, Copy, Default)]
pub struct HighlightState {
    pub in_block_comment: bool,
    pub in_script: bool,
    pub in_style: bool,
}

pub fn language_for_path(path: &Path) -> Language {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    match ext.as_str() {
        "rs" => Language::Rust,
        "py" | "pyi" | "pyw" => Language::Python,
        "js" | "mjs" | "cjs" => Language::JavaScript,
        "jsx" | "tsx" => Language::TypeScript,
        "ts" => Language::TypeScript,
        "vue" => Language::Vue,
        "svelte" => Language::Svelte,
        "html" | "htm" | "svg" | "xhtml" => Language::Html,
        "xml" | "xsd" | "xsl" | "xslt" | "plist" => Language::Xml,
        "css" => Language::Css,
        "scss" | "sass" | "less" => Language::Css,
        "json" | "jsonc" | "geojson" | "jsonl" => Language::Json,
        "md" | "markdown" | "mdx" => Language::Markdown,
        "sh" | "bash" | "zsh" | "fish" | "ksh" | "ash" | "dash" => Language::Shell,
        "toml" => Language::Toml,
        "yaml" | "yml" => Language::Yaml,
        "c" | "h" => Language::C,
        "cpp" | "cxx" | "cc" | "hpp" | "hxx" | "hh" | "c++" => Language::Cpp,
        "cs" => Language::CSharp,
        "java" => Language::Java,
        "go" => Language::Go,
        "rb" | "erb" | "rake" | "gemspec" => Language::Ruby,
        "php" | "phtml" => Language::Php,
        "swift" => Language::Swift,
        "kt" | "kts" => Language::Kotlin,
        "lua" => Language::Lua,
        "pl" | "pm" | "t" => Language::Perl,
        "r" | "rmd" | "R" => Language::R,
        "scala" | "sc" => Language::Scala,
        "hs" | "lhs" => Language::Haskell,
        "ex" | "exs" | "eex" | "heex" | "leex" => Language::Elixir,
        "dart" => Language::Dart,
        "sql" | "pgsql" | "mysql" => Language::Sql,
        "dockerfile" | "docker-compose" => Language::Dockerfile,
        "makefile" | "gnumakefile" | "mk" => Language::Makefile,
        _ => {
            if name == "Dockerfile" || name.starts_with("Dockerfile.") {
                Language::Dockerfile
            } else if name == "Makefile"
                || name == "GNUmakefile"
                || name.ends_with(".mk")
                || name == "CMakeLists.txt"
            {
                Language::Makefile
            } else if name == ".bashrc"
                || name == ".zshrc"
                || name == ".bash_profile"
                || name == ".zprofile"
                || name == ".profile"
                || name == ".bash_aliases"
            {
                Language::Shell
            } else if name == ".gitignore" || name == ".dockerignore" {
                Language::Shell
            } else if name == "PKGBUILD" || name == ".editorconfig" {
                Language::Shell
            } else {
                Language::Plain
            }
        }
    }
}

#[derive(Clone)]
struct Config {
    line_comment: &'static [&'static str],
    keywords: &'static [&'static str],
    types: &'static [&'static str],
    block_comment: Option<(&'static str, &'static str)>,
}

fn config(lang: Language) -> Config {
    match lang {
        Language::Rust => Config {
            line_comment: &["//"],
            keywords: &[
                "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else",
                "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match",
                "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
                "super", "trait", "true", "type", "unsafe", "use", "where", "while",
            ],
            types: &[
                "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "str",
                "u8", "u16", "u32", "u64", "u128", "usize", "String", "Vec", "Option", "Result",
                "Box", "Rc", "Arc", "RefCell", "HashMap", "HashSet",
            ],
            block_comment: Some(("/*", "*/")),
        },
        Language::Python => Config {
            line_comment: &["#"],
            keywords: &[
                "and", "as", "assert", "async", "await", "break", "class", "continue", "def",
                "del", "elif", "else", "except", "False", "finally", "for", "from", "global", "if",
                "import", "in", "is", "lambda", "None", "nonlocal", "not", "or", "pass", "raise",
                "return", "True", "try", "while", "with", "yield", "match", "case", "type",
            ],
            types: &[
                "bool", "bytes", "dict", "float", "int", "list", "object", "set", "str", "tuple",
                "type", "Any", "Optional", "List", "Dict", "Tuple",
            ],
            block_comment: None,
        },
        Language::JavaScript | Language::TypeScript => Config {
            line_comment: &["//"],
            keywords: &[
                "async", "await", "break", "case", "catch", "class", "const", "continue",
                "default", "delete", "do", "else", "export", "extends", "false", "finally", "for",
                "from", "function", "if", "import", "in", "instanceof", "let", "new", "null",
                "of", "return", "static", "super", "switch", "this", "throw", "true", "try",
                "typeof", "undefined", "var", "void", "while", "with", "yield",
            ],
            types: &[
                "string", "number", "boolean", "any", "unknown", "never", "void", "object",
                "symbol", "bigint", "Array", "Promise", "Map", "Set", "Record", "interface",
                "type", "enum", "implements", "declare", "readonly", "namespace", "public",
                "private", "protected", "as", "satisfies", "keyof", "infer", "extends",
            ],
            block_comment: Some(("/*", "*/")),
        },
        Language::C => Config {
            line_comment: &["//"],
            keywords: &[
                "if", "else", "for", "while", "do", "switch", "case", "default", "break",
                "continue", "return", "sizeof", "typedef", "struct", "union", "enum", "extern",
                "static", "const", "volatile", "register", "inline", "restrict", "auto", "goto",
                "true", "false", "NULL", "_Alignas", "_Atomic", "_Generic", "_Noreturn",
                "_Static_assert", "typeof", "_Bool",
            ],
            types: &[
                "int", "char", "short", "long", "float", "double", "void", "size_t", "uint8_t",
                "int32_t", "uint32_t", "int64_t", "uint64_t", "bool", "wchar_t", "FILE",
            ],
            block_comment: Some(("/*", "*/")),
        },
        Language::Cpp => Config {
            line_comment: &["//"],
            keywords: &[
                "if", "else", "for", "while", "do", "switch", "case", "default", "break",
                "continue", "return", "sizeof", "typedef", "struct", "union", "enum", "extern",
                "static", "const", "volatile", "register", "inline", "auto", "goto", "true",
                "false", "NULL", "nullptr", "class", "public", "private", "protected", "virtual",
                "override", "final", "template", "typename", "using", "namespace", "try", "catch",
                "throw", "noexcept", "constexpr", "consteval", "constinit", "new", "delete",
                "concept", "requires", "co_await", "co_yield", "co_return", "mutable",
            ],
            types: &[
                "int", "char", "short", "long", "float", "double", "void", "size_t", "uint8_t",
                "int32_t", "bool", "string", "vector", "map", "set", "shared_ptr", "unique_ptr",
                "optional", "variant", "pair", "array", "deque", "list", "unordered_map",
                "unordered_set", "bitset", "span", "string_view",
            ],
            block_comment: Some(("/*", "*/")),
        },
        Language::CSharp => Config {
            line_comment: &["//"],
            keywords: &[
                "using", "namespace", "class", "struct", "interface", "enum", "delegate", "var",
                "new", "typeof", "sizeof", "as", "is", "in", "out", "ref", "params", "this",
                "base", "null", "true", "false", "if", "else", "for", "foreach", "while", "do",
                "switch", "case", "default", "break", "continue", "return", "yield", "throw",
                "try", "catch", "finally", "lock", "checked", "unchecked", "async", "await",
                "operator", "where", "select", "from", "group", "join", "let", "orderby",
                "ascending", "descending", "static", "abstract", "virtual", "override", "sealed",
                "readonly", "const", "partial", "internal", "explicit", "implicit", "get", "set",
                "init", "record", "with", "when", "goto", "fixed", "stackalloc",
            ],
            types: &[
                "int", "long", "short", "byte", "float", "double", "decimal", "bool", "char",
                "string", "object", "dynamic", "void", "Array", "List", "Dictionary", "HashSet",
                "Tuple", "Task", "ValueTask", "Span", "Memory",
            ],
            block_comment: Some(("/*", "*/")),
        },
        Language::Java => Config {
            line_comment: &["//"],
            keywords: &[
                "abstract", "assert", "break", "case", "catch", "class", "continue", "default",
                "do", "else", "enum", "extends", "final", "finally", "for", "if", "implements",
                "import", "instanceof", "interface", "native", "new", "package", "private",
                "protected", "public", "return", "static", "strictfp", "super", "switch",
                "synchronized", "this", "throw", "throws", "transient", "try", "var", "void",
                "volatile", "while", "yield", "record", "sealed", "permits", "with", "true",
                "false", "null", "instanceof",
            ],
            types: &[
                "int", "byte", "short", "long", "float", "double", "char", "boolean", "void",
                "String", "Object", "List", "Map", "Set", "ArrayList", "HashMap", "HashSet",
                "Optional", "Stream", "Future", "CompletableFuture", "Integer", "Double",
                "Boolean", "Character", "Long", "Short", "Float", "Byte",
            ],
            block_comment: Some(("/*", "*/")),
        },
        Language::Go => Config {
            line_comment: &["//"],
            keywords: &[
                "break", "case", "chan", "const", "continue", "default", "defer", "else",
                "fallthrough", "for", "func", "go", "goto", "if", "import", "interface", "map",
                "package", "range", "return", "select", "struct", "switch", "type", "var", "true",
                "false", "nil",
            ],
            types: &[
                "bool", "byte", "complex64", "complex128", "error", "float32", "float64", "int",
                "int8", "int16", "int32", "int64", "rune", "string", "uint", "uint8", "uint16",
                "uint32", "uint64", "uintptr", "any", "comparable",
            ],
            block_comment: Some(("/*", "*/")),
        },
        Language::Ruby => Config {
            line_comment: &["#"],
            keywords: &[
                "alias", "and", "begin", "break", "case", "class", "def", "defined?", "do",
                "else", "elsif", "end", "ensure", "false", "for", "if", "in", "module", "next",
                "nil", "not", "or", "redo", "rescue", "retry", "return", "self", "super", "then",
                "true", "undef", "unless", "until", "when", "while", "yield", "require", "include",
                "extend", "raise", "lambda", "proc", "puts", "gets", "attr_accessor", "attr_reader",
                "attr_writer",
            ],
            types: &[
                "Array", "Hash", "Integer", "Float", "String", "Symbol", "NilClass", "TrueClass",
                "FalseClass", "Proc", "Lambda", "Range", "Regexp",
            ],
            block_comment: None,
        },
        Language::Php => Config {
            line_comment: &["//", "#"],
            keywords: &[
                "abstract", "and", "array", "as", "break", "callable", "case", "catch", "class",
                "clone", "const", "continue", "declare", "default", "die", "do", "echo", "else",
                "elseif", "empty", "enddeclare", "endfor", "endforeach", "endif", "endswitch",
                "endwhile", "eval", "exit", "extends", "final", "finally", "fn", "for", "foreach",
                "function", "global", "goto", "if", "implements", "include", "include_once",
                "instanceof", "insteadof", "interface", "isset", "list", "match", "namespace",
                "new", "or", "print", "private", "protected", "public", "readonly", "require",
                "require_once", "return", "static", "switch", "throw", "trait", "try", "unset",
                "use", "var", "while", "xor", "yield", "yield from", "true", "false", "null",
                "self", "parent",
            ],
            types: &[
                "int", "float", "string", "bool", "array", "object", "callable", "iterable",
                "self", "parent", "static", "mixed", "null", "void", "never", "false", "true",
                "iterable", "resource",
            ],
            block_comment: Some(("/*", "*/")),
        },
        Language::Swift => Config {
            line_comment: &["//"],
            keywords: &[
                "associatedtype", "break", "case", "catch", "class", "continue", "default",
                "defer", "deinit", "do", "else", "enum", "extension", "fallthrough", "false",
                "fileprivate", "for", "func", "guard", "if", "import", "in", "indirect", "init",
                "inout", "internal", "is", "let", "mutating", "nil", "none", "nonisolated", "open",
                "operator", "optional", "override", "prefix", "postfix", "private", "protocol",
                "public", "repeat", "required", "rethrows", "return", "self", "Self", "some",
                "static", "struct", "subscript", "super", "switch", "throw", "throws", "true",
                "try", "typealias", "unowned", "var", "weak", "where", "while", "willSet",
                "didSet", "any", "convenience",
            ],
            types: &[
                "Bool", "Int", "Double", "Float", "String", "Character", "Array", "Dictionary",
                "Set", "Optional", "Result", "Error", "Range", "ClosedRange",
            ],
            block_comment: Some(("/*", "*/")),
        },
        Language::Kotlin => Config {
            line_comment: &["//"],
            keywords: &[
                "as", "break", "class", "continue", "do", "else", "false", "for", "fun", "if",
                "in", "interface", "is", "it", "null", "object", "package", "return", "super",
                "this", "throw", "true", "try", "typealias", "val", "var", "when", "while", "by",
                "catch", "constructor", "companion", "data", "delegate", "dynamic", "enum",
                "expect", "final", "finally", "get", "import", "infix", "init", "inner",
                "internal", "lateinit", "noinline", "open", "operator", "out", "override", "param",
                "private", "protected", "public", "reified", "sealed", "suspend", "tailrec",
                "vararg", "actual", "annotation",
            ],
            types: &[
                "Any", "Boolean", "Byte", "Char", "Double", "Float", "Int", "Long", "Nothing",
                "Short", "String", "Unit", "Array", "List", "MutableList", "Map", "MutableMap",
                "Set", "MutableSet", "Pair", "Triple",
            ],
            block_comment: Some(("/*", "*/")),
        },
        Language::Lua => Config {
            line_comment: &["--"],
            keywords: &[
                "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto",
                "if", "in", "local", "nil", "not", "or", "repeat", "return", "then", "true",
                "until", "while",
            ],
            types: &[
                "select", "ipairs", "pairs", "pcall", "xpcall", "require", "setmetatable",
                "getmetatable", "tostring", "tonumber", "type", "rawget", "rawset", "print",
                "error", "assert", "next", "loadfile", "collectgarbage",
            ],
            block_comment: None,
        },
        Language::Perl => Config {
            line_comment: &["#"],
            keywords: &[
                "my", "our", "local", "sub", "if", "elsif", "else", "unless", "while", "until",
                "for", "foreach", "do", "use", "require", "package", "return", "last", "next",
                "redo", "goto", "die", "warn", "exit", "BEGIN", "END", "UNITCHECK", "say", "print",
                "map", "grep", "sort", "split", "join", "defined", "exists", "delete", "push",
                "pop", "shift", "unshift", "splice", "keys", "values", "each", "bless", "ref",
                "eval", "no", "CORE",
            ],
            types: &[
                "scalar", "array", "hash", "IO", "FileHandle", "Regexp", "Math::BigFloat",
            ],
            block_comment: None,
        },
        Language::R => Config {
            line_comment: &["#"],
            keywords: &[
                "if", "else", "for", "while", "repeat", "in", "next", "break", "function",
                "return", "TRUE", "FALSE", "NULL", "Inf", "NaN", "NA", "NA_integer_",
                "NA_real_", "NA_complex_", "NA_character_", "library", "require", "source",
                "invisible", "on.exit", "tryCatch", "with", "within", "function", "local",
                "expression", "quote", "eval", "sys.call", "substitute", "missing",
            ],
            types: &[
                "numeric", "integer", "double", "character", "logical", "complex", "raw", "list",
                "data.frame", "matrix", "vector", "factor", "environment",
            ],
            block_comment: None,
        },
        Language::Scala => Config {
            line_comment: &["//"],
            keywords: &[
                "abstract", "case", "catch", "class", "def", "do", "else", "extends", "false",
                "final", "finally", "for", "forSome", "if", "implicit", "import", "lazy", "match",
                "new", "null", "object", "override", "package", "private", "protected", "return",
                "sealed", "self", "super", "this", "throw", "trait", "true", "try", "type", "val",
                "var", "while", "with", "yield", "given", "using", "then", "enum", "derives",
                "end", "extension", "infix", "inline", "opaque", "open", "transparent",
            ],
            types: &[
                "Int", "Long", "Short", "Byte", "Float", "Double", "Char", "Boolean", "Unit",
                "Any", "AnyVal", "AnyRef", "Nothing", "Null", "Option", "Some", "None", "List",
                "Map", "Set", "Array", "Vector", "Tuple", "Either", "Future", "Try",
            ],
            block_comment: Some(("/*", "*/")),
        },
        Language::Haskell => Config {
            line_comment: &["--"],
            keywords: &[
                "case", "class", "data", "default", "deriving", "do", "else", "foreign", "if",
                "import", "in", "infix", "infixl", "infixr", "instance", "let", "module",
                "newtype", "of", "qualified", "then", "type", "where", "as", "hiding", "forall",
                "family", "role", "kind", "stock", "via", "generalising", "import", "qualified",
            ],
            types: &[
                "Int", "Integer", "Float", "Double", "Bool", "Char", "String", "Maybe", "Either",
                "IO", "EitherT", "List", "Seq", "Set", "Map", "HashMap", "Vector", "Array",
                "IORef", "MaybeT", "ReaderT", "WriterT", "StateT", "Any",
            ],
            block_comment: Some(("{-", "-}")),
        },
        Language::Elixir => Config {
            line_comment: &["#"],
            keywords: &[
                "def", "defp", "defmodule", "defprotocol", "defstruct", "defimpl", "defmacro",
                "defmacrop", "defdelegate", "defexception", "defguard", "defguardp",
                "defoverridable", "defcallback", "fn", "do", "end", "after", "else", "catch",
                "rescue", "with", "case", "cond", "if", "unless", "for", "receive", "try", "raise",
                "throw", "import", "require", "use", "alias", "quote", "unquote", "nil", "true",
                "false", "and", "or", "not", "when", "in",
            ],
            types: &[
                "Atom", "BitString", "Float", "Integer", "List", "Map", "MapSet", "Tuple", "PID",
                "Port", "Reference", "Fun", "Any", "none", "term",
            ],
            block_comment: None,
        },
        Language::Dart => Config {
            line_comment: &["//"],
            keywords: &[
                "abstract", "assert", "async", "await", "break", "case", "catch", "class",
                "const", "continue", "covariant", "default", "deferred", "do", "dynamic", "else",
                "enum", "export", "extends", "extension", "external", "factory", "false", "final",
                "finally", "for", "get", "hide", "if", "implements", "import", "in", "interface",
                "is", "late", "library", "mixin", "new", "null", "on", "operator", "part",
                "required", "rethrow", "return", "set", "show", "static", "super", "switch",
                "sync", "this", "throw", "true", "try", "typedef", "var", "void", "while", "with",
                "yield",
            ],
            types: &[
                "int", "double", "num", "String", "bool", "List", "Map", "Set", "DateTime",
                "Duration", "Iterable", "Stream", "Future", "StreamController", "Completer",
                "StackTrace", "Symbol", "Type", "Null", "Never", "Object", "dynamic", "void",
            ],
            block_comment: Some(("/*", "*/")),
        },
        Language::Sql => Config {
            line_comment: &["--"],
            keywords: &[
                "SELECT", "FROM", "WHERE", "INSERT", "INTO", "VALUES", "UPDATE", "SET", "DELETE",
                "CREATE", "ALTER", "DROP", "TABLE", "INDEX", "VIEW", "DATABASE", "JOIN", "INNER",
                "LEFT", "RIGHT", "FULL", "OUTER", "CROSS", "ON", "GROUP", "BY", "ORDER", "ASC",
                "DESC", "HAVING", "LIMIT", "OFFSET", "UNION", "ALL", "INTERSECT", "EXCEPT",
                "DISTINCT", "AS", "AND", "OR", "NOT", "IN", "BETWEEN", "LIKE", "IS", "NULL",
                "EXISTS", "CASE", "WHEN", "THEN", "ELSE", "END", "CAST", "COALESCE", "IF",
                "IFNULL", "OVER", "PARTITION", "WINDOW", "ROWS", "RANGE", "PRECEDING", "FOLLOWING",
                "CURRENT", "UNBOUNDED", "ROW_NUMBER", "RANK", "DENSE_RANK", "LEAD", "LAG",
                "BEGIN", "COMMIT", "ROLLBACK", "TRANSACTION", "GRANT", "REVOKE",
            ],
            types: &[
                "INT", "INTEGER", "BIGINT", "SMALLINT", "TINYINT", "FLOAT", "DOUBLE", "DECIMAL",
                "NUMERIC", "REAL", "CHAR", "VARCHAR", "TEXT", "BLOB", "BOOLEAN", "DATE", "TIME",
                "TIMESTAMP", "DATETIME", "JSON", "JSONB", "UUID",
            ],
            block_comment: None,
        },
        Language::Html | Language::Xml | Language::Vue | Language::Svelte => Config {
            line_comment: &[],
            keywords: &[],
            types: &[],
            block_comment: None,
        },
        Language::Css => Config {
            line_comment: &[],
            keywords: &[
                "important", "media", "keyframes", "import", "supports", "font-face",
            ],
            types: &[],
            block_comment: Some(("/*", "*/")),
        },
        Language::Json => Config {
            line_comment: &[],
            keywords: &["true", "false", "null"],
            types: &[],
            block_comment: None,
        },
        Language::Toml => Config {
            line_comment: &["#"],
            keywords: &["true", "false"],
            types: &[],
            block_comment: None,
        },
        Language::Shell | Language::Dockerfile | Language::Makefile => Config {
            line_comment: &["#"],
            keywords: &[
                "if", "then", "else", "elif", "fi", "for", "while", "do", "done", "case", "esac",
                "function", "in", "return", "export", "local", "echo", "cd", "sudo", "source",
                "set", "unset", "trap", "exec", "readonly", "declare", "typeset", "shift",
            ],
            types: &[],
            block_comment: None,
        },
        Language::Yaml => Config {
            line_comment: &["#"],
            keywords: &["true", "false", "null", "yes", "no", "on", "off"],
            types: &[],
            block_comment: None,
        },
        Language::Markdown | Language::Plain => Config {
            line_comment: &[],
            keywords: &[],
            types: &[],
            block_comment: None,
        },
        Language::Custom => Config {
            line_comment: &[],
            keywords: &[],
            types: &[],
            block_comment: None,
        },
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '$'
}

fn is_ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

// ── Dynamic-language registry (installed extensions) ──────────────

static DYN: OnceLock<RwLock<Vec<(String, Config)>>> = OnceLock::new();

fn dyn_map() -> &'static RwLock<Vec<(String, Config)>> {
    DYN.get_or_init(|| RwLock::new(Vec::new()))
}

/// Leak an owned `String` into a `&'static str` so that the dynamically-built
/// [`Config`] references are valid for the lifetime of the process. The
/// allocation is tiny and only happens when an extension is installed.
fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn leak_slice(v: &[String]) -> &'static [&'static str] {
    Vec::leak(v.iter().map(|s| leak(s.clone())).collect())
}

/// Rebuild the in-memory map of extension-contributed languages that the
/// tokenizer can resolve at render time.
pub fn set_dynamic_languages(langs: Vec<DynamicLang>) {
    let map: Vec<(String, Config)> = langs
        .into_iter()
        .filter_map(|l| {
            let mut comments = Vec::new();
            if let Some(ref s) = l.line_comment {
                comments.push(leak(s.clone()));
            }
            let bc = l.block_comment.as_ref().and_then(|v| {
                if v.len() >= 2 {
                    Some((leak(v[0].clone()), leak(v[1].clone())))
                } else {
                    None
                }
            });
            let cfg = Config {
                line_comment: Vec::leak(comments),
                keywords: leak_slice(&l.keywords),
                types: leak_slice(&l.types),
                block_comment: bc,
            };
            Some((l.id.clone(), cfg))
        })
        .collect();
    *dyn_map().write().unwrap() = map;
}

fn dynamic_cfg(id: &str) -> Option<Config> {
    dyn_map().read().unwrap().iter().find(|(k, _)| k == id).map(|(_, c)| c.clone())
}

// ── Public tokenise API ──────────────────────────────────────────

pub fn tokenize_line(
    lang: Language,
    dyn_id: Option<&str>,
    line: &str,
    state: HighlightState,
) -> (Vec<Token>, HighlightState) {
    match lang {
        Language::Html | Language::Xml | Language::Vue | Language::Svelte => {
            let (tokens, next) = tokenize_html_family(lang, line, state);
            (tokens, next)
        }
        Language::Markdown => (tokenize_markdown(line), state),
        Language::Yaml => (tokenize_yaml(line), state),
        Language::Plain => (vec![Token::new(0, line.len(), TokenKind::Plain)], state),
        Language::Json => (tokenize_json(line), state),
        Language::Custom => {
            let mut in_block = state.in_block_comment;
            if let Some(dyn_id) = dyn_id
                && let Some(cfg) = dynamic_cfg(dyn_id)
            {
                let tokens = code_state_from_cfg(&cfg, line, &mut in_block);
                return (
                    tokens,
                    HighlightState {
                        in_block_comment: in_block,
                        ..Default::default()
                    },
                );
            }
            (
                vec![Token::new(0, line.len(), TokenKind::Plain)],
                state,
            )
        }
        _ => {
            let mut in_block = state.in_block_comment;
            let tokens = builtin_code_state(lang, line, &mut in_block);
            (
                tokens,
                HighlightState {
                    in_block_comment: in_block,
                    ..Default::default()
                },
            )
        }
    }
}

fn builtin_code_state(lang: Language, line: &str, in_block: &mut bool) -> Vec<Token> {
    let cfg = config(lang);
    code_state_from_cfg(&cfg, line, in_block)
}

fn code_state_from_cfg(cfg: &Config, line: &str, in_block: &mut bool) -> Vec<Token> {
    if *in_block {
        if let Some((_, close)) = cfg.block_comment
            && let Some(rel) = line.find(close)
        {
            let end = rel + close.len();
            *in_block = false;
            if end < line.len() {
                let mut tokens = vec![Token::new(0, end, TokenKind::Comment)];
                tokens.extend(code_from_cfg(cfg, &line[end..]));
                return fixup(tokens, line.len(), 0);
            }
            return vec![Token::new(0, line.len(), TokenKind::Comment)];
        }
        return vec![Token::new(0, line.len(), TokenKind::Comment)];
    }

    code_from_cfg(cfg, line)
}

fn code_from_cfg(cfg: &Config, line: &str) -> Vec<Token> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut tokens: Vec<Token> = Vec::new();
    let mut i = 0usize;

    while i < len {
        let rest = &line[i..];

        if let Some(_marker) = cfg.line_comment.iter().find(|m| rest.starts_with(**m)) {
            tokens.push(Token::new(i, len, TokenKind::Comment));
            break;
        }

        if let Some((open, close)) = cfg.block_comment {
            if rest.starts_with(open) {
                if let Some(rel) = line[i + open.len()..].find(close) {
                    let end = i + open.len() + rel + close.len();
                    tokens.push(Token::new(i, end, TokenKind::Comment));
                    i = end;
                    continue;
                } else {
                    tokens.push(Token::new(i, len, TokenKind::Comment));
                    break;
                }
            }
        }

        let c = rest.chars().next().unwrap();

        if c == '"' || c == '\'' || c == '`' {
            let quote = c;
            let mut j = i + c.len_utf8();
            let mut escaped = false;
            while j < len {
                let cj = line[j..].chars().next().unwrap();
                if escaped {
                    escaped = false;
                } else if cj == '\\' {
                    escaped = true;
                } else if cj == quote {
                    j += cj.len_utf8();
                    break;
                }
                j += cj.len_utf8();
            }
            tokens.push(Token::new(i, j, TokenKind::Str));
            i = j;
            continue;
        }

        if c.is_ascii_digit() {
            let mut j = i;
            let mut seen_dot = false;
            let mut seen_e = false;
            while j < len {
                let cj = line[j..].chars().next().unwrap();
                if cj.is_ascii_alphanumeric() || cj == '_' {
                    if cj == 'e' || cj == 'E' {
                        seen_e = true;
                    }
                    j += cj.len_utf8();
                } else if cj == '.' && !seen_dot && !seen_e {
                    seen_dot = true;
                    j += 1;
                } else {
                    break;
                }
            }
            tokens.push(Token::new(i, j, TokenKind::Number));
            i = j;
            continue;
        }

        if is_ident_start(c) {
            let mut j = i;
            while j < len {
                let cj = line[j..].chars().next().unwrap();
                if is_ident_continue(cj) {
                    j += cj.len_utf8();
                } else {
                    break;
                }
            }
            let word = &line[i..j];
            let kind = classify(&cfg, word, &line[j..]);
            tokens.push(Token::new(i, j, kind));
            i = j;
            continue;
        }

        let kind = if "()[]{};,.".contains(c) {
            TokenKind::Punctuation
        } else {
            TokenKind::Operator
        };
        let end = i + c.len_utf8();
        tokens.push(Token::new(i, end, kind));
        i = end;
    }

    finalize(tokens, len)
}

fn classify(cfg: &Config, word: &str, after: &str) -> TokenKind {
    if cfg.keywords.contains(&word) {
        return TokenKind::Keyword;
    }
    if cfg.types.contains(&word) {
        return TokenKind::Type;
    }
    let first = word.chars().next().unwrap_or('_');
    if first.is_uppercase() {
        return TokenKind::Type;
    }
    let looks_const = word.len() > 1
        && word
            .chars()
            .all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '_')
        && word.chars().any(|c| c.is_ascii_uppercase());
    if looks_const {
        return TokenKind::Constant;
    }
    if after.trim_start().starts_with('(') {
        return TokenKind::Function;
    }
    TokenKind::Plain
}

fn fixup(mut tokens: Vec<Token>, len: usize, offset: usize) -> Vec<Token> {
    for t in &mut tokens {
        t.start += offset;
        t.end += offset;
    }
    let mut out = Vec::with_capacity(tokens.len());
    let mut cursor = offset;
    for t in tokens {
        if t.start > cursor {
            out.push(Token::new(cursor, t.start, TokenKind::Plain));
        }
        if t.end > t.start {
            out.push(t);
            cursor = t.end;
        }
    }
    if cursor < offset + len {
        out.push(Token::new(cursor, offset + len, TokenKind::Plain));
    }
    if out.is_empty() {
        out.push(Token::new(offset, offset + len, TokenKind::Plain));
    }
    out
}

fn finalize(tokens: Vec<Token>, len: usize) -> Vec<Token> {
    let mut out: Vec<Token> = Vec::with_capacity(tokens.len());
    let mut cursor = 0usize;
    for t in tokens {
        if t.start > cursor {
            out.push(Token::new(cursor, t.start, TokenKind::Plain));
        }
        let start = t.start.max(cursor);
        if t.end > start {
            out.push(Token::new(start, t.end, t.kind));
            cursor = t.end;
        }
    }
    if cursor < len {
        out.push(Token::new(cursor, len, TokenKind::Plain));
    }
    if out.is_empty() {
        out.push(Token::new(0, len, TokenKind::Plain));
    }
    out
}

fn tokenize_markdown(line: &str) -> Vec<Token> {
    let len = line.len();
    let trimmed = line.trim_start();
    let indent = len - trimmed.len();

    if trimmed.starts_with('#') {
        return vec![Token::new(0, len, TokenKind::Keyword)];
    }
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("> ") {
        let mut tokens = Vec::new();
        if indent > 0 {
            tokens.push(Token::new(0, indent, TokenKind::Plain));
        }
        tokens.push(Token::new(indent, indent + 1, TokenKind::Operator));
        let start = (indent + 2).min(len);
        if start < len {
            tokens.push(Token::new(start, len, TokenKind::Plain));
        }
        return finalize(tokens, len);
    }
    if trimmed.starts_with("```") {
        return vec![Token::new(0, len, TokenKind::Str)];
    }

    let mut tokens = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < len {
        if bytes[i] == b'`'
            && let Some(rel) = line[i + 1..].find('`')
        {
            tokens.push(Token::new(i, i + 1 + rel + 1, TokenKind::Str));
            i = i + 1 + rel + 1;
            continue;
        }
        if bytes[i] == b'[' && let Some(end) = markdown_link_end(line, i) {
            tokens.push(Token::new(i, end, TokenKind::Function));
            i = end;
            continue;
        }
        i += 1;
    }
    finalize(tokens, len)
}

fn markdown_link_end(line: &str, start: usize) -> Option<usize> {
    let rest = &line[start..];
    let bracket_close = rest.find(']')?;
    let after_bracket = start + bracket_close + 1;
    if after_bracket >= line.len() || line.as_bytes()[after_bracket] != b'(' {
        return None;
    }
    let paren_close = line[after_bracket..].find(')')?;
    Some(after_bracket + paren_close + 1)
}

fn tokenize_yaml(line: &str) -> Vec<Token> {
    let len = line.len();
    let trimmed = line.trim_start();
    let indent = len - trimmed.len();

    if trimmed.starts_with('#') {
        return vec![Token::new(0, len, TokenKind::Comment)];
    }

    if trimmed.is_empty() {
        return vec![Token::new(0, len, TokenKind::Plain)];
    }

    let mut tokens = Vec::new();

    if indent > 0 {
        tokens.push(Token::new(0, indent, TokenKind::Plain));
    }

    let rest = trimmed;

    if let Some(colon_pos) = rest.find(':') {
        let key_end = indent + colon_pos;
        tokens.push(Token::new(indent, key_end, TokenKind::Attribute));
        tokens.push(Token::new(key_end, key_end + 1, TokenKind::Operator));
        let value_start = (key_end + 1..len).find(|&i| rest.as_bytes()[i - indent] != b' ');
        if let Some(rel) = value_start {
            let vs = indent + rel;
            let val_char = line.as_bytes()[vs];
            if val_char == b'"' || val_char == b'\'' {
                if let Some(end) = quoted_end(line, vs) {
                    tokens.push(Token::new(vs, end, TokenKind::Str));
                } else {
                    tokens.push(Token::new(vs, len, TokenKind::Str));
                }
            } else if val_char == b'[' || val_char == b'{' {
                tokens.push(Token::new(vs, len, TokenKind::Punctuation));
            } else {
                tokens.push(Token::new(vs, len, TokenKind::Keyword));
            }
        }
        return finalize(tokens, len);
    }

    if trimmed.starts_with('-') {
        tokens.push(Token::new(indent, (indent + 1).min(len), TokenKind::Operator));
        let after = (indent + 1).min(len);
        if after < len {
            tokens.push(Token::new(after, len, TokenKind::Plain));
        }
        return finalize(tokens, len);
    }

    tokens.push(Token::new(indent, len, TokenKind::Plain));
    finalize(tokens, len)
}

fn tokenize_json(line: &str) -> Vec<Token> {
    let len = line.len();
    let bytes = line.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < len {
        let c = bytes[i];

        if c == b'"' {
            let mut j = i + 1;
            let mut escaped = false;
            while j < len {
                if escaped {
                    escaped = false;
                } else if bytes[j] == b'\\' {
                    escaped = true;
                } else if bytes[j] == b'"' {
                    j += 1;
                    break;
                }
                j += 1;
            }
            let kind = if i == 0 || bytes[i - 1] == b':' || bytes[i - 1] == b'{' || bytes[i - 1] == b'[' || bytes[i - 1] == b',' {
                TokenKind::Str
            } else {
                TokenKind::Str
            };
            tokens.push(Token::new(i, j, kind));
            i = j;
            continue;
        }

        if c.is_ascii_digit() || c == b'-' {
            let mut j = i + 1;
            while j < len && (bytes[j].is_ascii_digit() || bytes[j] == b'.' || bytes[j] == b'e' || bytes[j] == b'E' || bytes[j] == b'+' || bytes[j] == b'-') {
                j += 1;
            }
            tokens.push(Token::new(i, j, TokenKind::Number));
            i = j;
            continue;
        }

        if c == b't' || c == b'f' || c == b'n' {
            let word = if c == b't' { "true" } else if c == b'f' { "false" } else { "null" };
            if line[i..].starts_with(word) {
                tokens.push(Token::new(i, i + word.len(), TokenKind::Keyword));
                i += word.len();
                continue;
            }
        }

        if c == b':' || c == b',' {
            tokens.push(Token::new(i, i + 1, TokenKind::Operator));
            i += 1;
            continue;
        }

        if c == b'{' || c == b'}' || c == b'[' || c == b']' {
            tokens.push(Token::new(i, i + 1, TokenKind::Punctuation));
            i += 1;
            continue;
        }

        i += 1;
    }

    finalize(tokens, len)
}

fn quoted_end(line: &str, start: usize) -> Option<usize> {
    let quote = line.as_bytes()[start];
    let bytes = line.as_bytes();
    let mut j = start + 1;
    let mut escaped = false;
    while j < bytes.len() {
        if escaped {
            escaped = false;
        } else if bytes[j] == b'\\' {
            escaped = true;
        } else if bytes[j] == quote {
            return Some(j + 1);
        }
        j += 1;
    }
    None
}

fn tokenize_html_family(lang: Language, line: &str, mut state: HighlightState) -> (Vec<Token>, HighlightState) {
    if state.in_script {
        if let Some(idx) = line.find("</script") {
            let before = &line[..idx];
            let after_tag_start = &line[idx..];
            let mut tokens = builtin_code_state(Language::JavaScript, before, &mut false);
            let tag_end = after_tag_start.find('>').map(|p| idx + p + 1).unwrap_or(line.len());
            let mut tag_tokens = tokenize_html(&line[idx..tag_end]);
            for t in &mut tag_tokens {
                t.start += idx;
                t.end += idx;
            }
            tokens.append(&mut tag_tokens);
            state.in_script = false;
            if tag_end < line.len() {
                let mut tail = tokenize_html(&line[tag_end..]);
                for t in &mut tail {
                    t.start += tag_end;
                    t.end += tag_end;
                }
                tokens.append(&mut tail);
            }
            return (finalize(tokens, line.len()), state);
        }
        let mut in_block = false;
        let tokens = builtin_code_state(Language::JavaScript, line, &mut in_block);
        return (tokens, state);
    }

    if state.in_style {
        if let Some(idx) = line.find("</style") {
            let before = &line[..idx];
            let mut tokens = builtin_code_state(Language::Css, before, &mut false);
            let after_tag_start = &line[idx..];
            let tag_end = after_tag_start.find('>').map(|p| idx + p + 1).unwrap_or(line.len());
            let mut tag_tokens = tokenize_html(&line[idx..tag_end]);
            for t in &mut tag_tokens {
                t.start += idx;
                t.end += idx;
            }
            tokens.append(&mut tag_tokens);
            state.in_style = false;
            if tag_end < line.len() {
                let mut tail = tokenize_html(&line[tag_end..]);
                for t in &mut tail {
                    t.start += tag_end;
                    t.end += tag_end;
                }
                tokens.append(&mut tail);
            }
            return (finalize(tokens, line.len()), state);
        }
        let mut in_block = false;
        let tokens = builtin_code_state(Language::Css, line, &mut in_block);
        return (tokens, state);
    }

    let tokens = tokenize_html(line);

    if lang != Language::Xml && (line.contains("<script") || line.contains("<script ")) {
        state.in_script = true;
    } else if lang != Language::Xml && (line.contains("<style") || line.contains("<style ")) {
        state.in_style = true;
    }

    (tokens, state)
}

fn tokenize_html(line: &str) -> Vec<Token> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < len {
        if bytes[i] == b'<' {
            let end = line[i..].find('>').map(|p| i + p + 1).unwrap_or(len);
            tokens.push(Token::new(i, (i + 1).min(end), TokenKind::Punctuation));
            let mut j = i + 1;
            if j < end && line.as_bytes()[j] == b'/' {
                tokens.push(Token::new(j, j + 1, TokenKind::Punctuation));
                j += 1;
            }
            let name_start = j;
            while j < end {
                let cj = line[j..].chars().next().unwrap();
                if cj.is_alphanumeric() || cj == '-' || cj == '_' {
                    j += cj.len_utf8();
                } else {
                    break;
                }
            }
            if j > name_start {
                tokens.push(Token::new(name_start, j, TokenKind::Keyword));
            }
            while j < end {
                let cj = line[j..].chars().next().unwrap();
                if cj.is_whitespace() {
                    j += cj.len_utf8();
                    continue;
                }
                if cj == '"' || cj == '\'' {
                    let quote = cj;
                    let s = j;
                    j += cj.len_utf8();
                    while j < end {
                        let cc = line[j..].chars().next().unwrap();
                        j += cc.len_utf8();
                        if cc == quote {
                            break;
                        }
                    }
                    tokens.push(Token::new(s, j, TokenKind::Str));
                    continue;
                }
                if cj.is_alphanumeric() || cj == '-' || cj == '_' {
                    let s = j;
                    while j < end {
                        let cc = line[j..].chars().next().unwrap();
                        if cc.is_alphanumeric() || cc == '-' || cc == '_' {
                            j += cc.len_utf8();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::new(s, j, TokenKind::Attribute));
                    continue;
                }
                let s = j;
                j += cj.len_utf8();
                tokens.push(Token::new(s, j, TokenKind::Operator));
            }
            if end > i && end <= len {
                tokens.push(Token::new(end.saturating_sub(1), end, TokenKind::Punctuation));
            }
            i = end;
        } else {
            let rel = line[i..].find('<').map(|p| i + p).unwrap_or(len);
            if rel > i {
                tokens.push(Token::new(i, rel, TokenKind::Plain));
            }
            i = rel.max(i + 1).min(len);
            if rel == len {
                break;
            }
        }
    }
    finalize(tokens, len)
}
