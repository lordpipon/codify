use std::path::Path;
use std::sync::{OnceLock, RwLock};

use serde::{Deserialize, Serialize};
use serde_json;

use crate::language::{DynamicLang, set_dynamic_languages};

static DYNAMIC: RwLock<Vec<DynamicLang>> = RwLock::new(Vec::new());

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Extension {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    #[serde(default)]
    pub languages: Vec<DynamicLang>,
}

// ────────────────────────────────────────────────────────────────
// Built-in store catalog. This is what makes the app an "extension
// store": a discoverable set of installable packages. A remote
// registry could be swapped in here (fetch over HTTP and parse).
// ────────────────────────────────────────────────────────────────

const CATALOG_JSON: &str = r##"[
  {
    "id": "zig",
    "name": "Zig Language",
    "version": "0.1.0",
    "description": "Highlighting for Zig (.zig)",
    "author": "lordpipon",
    "languages": [
      {
        "id": "zig",
        "name": "Zig",
        "extensions": ["zig", "zon"],
        "keywords": ["and","break","catch","comptime","const","continue","defer","else","enum","errdefer","error","export","extern","fn","for","if","in","inline","noalias","noinline","opaque","or","orelse","packed","pub","resume","return","struct","switch","test","threadlocal","try","union","unreachable","usingnamespace","var","volatile","while","suspend","async","await"],
        "types": ["i8","i16","i32","i64","i128","u8","u16","u32","u64","u128","usize","isize","f16","f32","f64","f80","f128","bool","void","noreturn","type","anytype","anyerror"],
        "line_comment": "//",
        "block_comment": ["/*","*/"]
      }
    ]
  },
  {
    "id": "nim",
    "name": "Nim Language",
    "version": "0.1.0",
    "description": "Highlighting for Nim (.nim)",
    "author": "lordpipon",
    "languages": [
      {
        "id": "nim",
        "name": "Nim",
        "extensions": ["nim"],
        "keywords": ["addr","and","as","asm","bind","block","break","case","cast","const","continue","converter","defer","discard","distinct","div","do","elif","else","end","enum","except","export","finally","for","from","func","if","import","in","include","interface","is","isnot","iterator","let","macro","method","mixin","mod","nil","not","notin","object","of","or","out","proc","ptr","raise","ref","return","shl","shr","static","template","try","tuple","type","using","var","when","while","xor","yield"],
        "types": ["int","int8","uint8","int16","uint16","int32","uint32","int64","uint64","float","float32","float64","bool","char","string","cstring","pointer","array","seq","set","range","openArray"],
        "line_comment": "#",
        "block_comment": ["#[","]#"]
      }
    ]
  },
  {
    "id": "groovy",
    "name": "Groovy Language",
    "version": "0.1.0",
    "description": "Highlighting for Groovy (.groovy)",
    "author": "lordpipon",
    "languages": [
      {
        "id": "groovy",
        "name": "Groovy",
        "extensions": ["groovy","gradle"],
        "keywords": ["as","assert","break","case","catch","class","const","continue","def","default","do","else","enum","extends","final","finally","for","goto","if","implements","import","in","instanceof","interface","native","new","package","private","protected","public","return","static","strictfp","super","switch","synchronized","this","throw","throws","transient","trait","try","void","volatile","while","true","false","null","with","given","when","then","ensure","until"],
        "types": ["int","char","long","short","float","double","boolean","byte","BigDecimal","BigInteger","Closure","String","List","Map","Range","GString","Tuple","Date"],
        "line_comment": "//",
        "block_comment": ["/*","*/"]
      }
    ]
  },
  {
    "id": "crystal",
    "name": "Crystal Language",
    "version": "0.1.0",
    "description": "Highlighting for Crystal (.cr)",
    "author": "lordpipon",
    "languages": [
      {
        "id": "crystal",
        "name": "Crystal",
        "extensions": ["cr"],
        "keywords": ["abstract","alias","as","asm","begin","break","case","class","def","do","else","elsif","end","ensure","enum","extend","for","fun","if","ifdef","in","include","instance_sizeof","lib","macro","module","next","nil","of","out","pointerof","private","protected","require","rescue","return","self","sizeof","struct","super","then","type","typeof","uninitialized","union","unless","until","when","while","with","yield"],
        "types": ["Int8","UInt8","Int16","UInt16","Int32","UInt32","Int64","UInt64","Float32","Float64","Bool","Char","String","Symbol","Array","ArrayLiteral","Hash","NamedTuple","Tuple","Proc","Range","Set","Slice","StaticArray"],
        "line_comment": "#",
        "block_comment": ["=begin","=end"]
      }
    ]
  },
  {
    "id": "racket",
    "name": "Racket Language",
    "version": "0.1.0",
    "description": "Highlighting for Racket (.rkt)",
    "author": "lordpipon",
    "languages": [
      {
        "id": "racket",
        "name": "Racket",
        "extensions": ["rkt","rktd","ss"],
        "keywords": ["define","define-syntax","define-syntax-rule","lambda","lambda*","let","let*","letrec","let-values","if","cond","else","and","or","case","when","unless","for","for/list","for/fold","while","begin","begin0","quote","quasiquote","unquote","unquote-splicing","require","provide","module","import","export","struct","class","object","new","send","method","public","private","protected","match","values","call/cc","dynamic-wind","delay","force","define-values","syntax-rules","define-type","define-struct","define-record-type","#t","#f","#true","#false","#null"],
        "types": ["number","string","symbol","boolean","char","pair","list","vector","hash","box","void","procedure","port"],
        "line_comment": ";",
        "block_comment": null
      }
    ]
  },
  {
    "id": "nix",
    "name": "Nix Language",
    "version": "0.1.0",
    "description": "Highlighting for Nix (.nix, flake.nix)",
    "author": "lordpipon",
    "languages": [
      {
        "id": "nix",
        "name": "Nix",
        "extensions": ["nix"],
        "keywords": ["let","in","if","then","else","assert","with","import","inherit","rec","or","builtins","true","false","null","throw","abort","map","filter","foldl","concatMap","getAttr","attrNames"],
        "types": ["int","float","string","path","bool","list","set","lambda","function","attribute"],
        "line_comment": "#",
        "block_comment": ["/*","*/"]
      }
    ]
  },
  {
    "id": "julia",
    "name": "Julia Language",
    "version": "0.1.0",
    "description": "Highlighting for Julia (.jl)",
    "author": "lordpipon",
    "languages": [
      {
        "id": "julia",
        "name": "Julia",
        "extensions": ["jl"],
        "keywords": ["abstract","as","async","await","baremodule","begin","break","catch","ccall","const","continue","do","else","elseif","end","enum","export","finally","for","function","global","if","import","in","let","local","macro","module","mutable","primitive","public","quote","return","struct","try","type","using","where","while","true","false","nothing"],
        "types": ["Bool","Int8","Int16","Int32","Int64","UInt8","UInt16","UInt32","UInt64","Float16","Float32","Float64","BigInt","BigFloat","Complex","Rational","Char","String","Symbol","Vector","Matrix","Array","Tuple","Real","Number","Any"],
        "line_comment": "#",
        "block_comment": ["#=","=#"]
      }
    ]
  }
]"##;

static CATALOG: OnceLock<Vec<Extension>> = OnceLock::new();

pub fn catalog() -> &'static [Extension] {
    CATALOG
        .get_or_init(|| serde_json::from_str(CATALOG_JSON).unwrap_or_default())
        .as_slice()
}

fn extensions_dir() -> std::path::PathBuf {
    crate::theme::config_dir().join("extensions")
}

/// Extensions installed on disk (~/.config/codify/extensions/*.json).
pub fn installed() -> Vec<Extension> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(extensions_dir()) else {
        return out;
    };
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(|s| s.to_string()) else {
            continue;
        };
        if !name.ends_with(".json") {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(entry.path())
            && let Ok(ext) = serde_json::from_str::<Extension>(&text)
        {
            out.push(ext);
        }
    }
    out
}

pub fn install(id: &str) -> bool {
    let Some(ext) = catalog().iter().find(|e| e.id == id) else {
        return false;
    };
    let dir = extensions_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return false;
    }
    let path = dir.join(format!("{}.json", ext.id));
    let Ok(text) = serde_json::to_string_pretty(ext) else {
        return false;
    };
    if std::fs::write(path, text).is_err() {
        return false;
    }
    refresh();
    true
}

pub fn uninstall(id: &str) -> bool {
    let path = extensions_dir().join(format!("{}.json", id));
    let ok = std::fs::remove_file(path).is_ok();
    if ok {
        refresh();
    }
    ok
}

/// All languages contributed by installed extensions.
pub fn dynamic_languages() -> Vec<DynamicLang> {
    installed()
        .into_iter()
        .flat_map(|e| e.languages)
        .collect()
}

/// Rebuild the in-memory dynamic-language cache used by the tokenizer.
pub fn refresh() {
    let langs = dynamic_languages();
    *DYNAMIC.write().unwrap() = langs.clone();
    set_dynamic_languages(langs);
}

/// Look up an installed extension language for the given file path.
pub fn dynamic_language_for_path(path: &Path) -> Option<DynamicLang> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let cache = DYNAMIC.read().unwrap();
    cache
        .iter()
        .find(|l| l.extensions.iter().any(|e| e.eq_ignore_ascii_case(&ext)))
        .cloned()
}

/// Call once at startup.
pub fn init() {
    refresh();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::{Language, TokenKind, tokenize_line};

    #[test]
    fn catalog_parses() {
        assert!(catalog().len() >= 6, "catalog should have several entries");
        for ext in catalog() {
            for lang in &ext.languages {
                assert!(!lang.id.is_empty());
                assert!(!lang.extensions.is_empty());
            }
        }
    }

    #[test]
    fn install_lookup_tokenize_uninstall_roundtrip() {
        let dir = std::env::temp_dir().join(format!("codify-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        // Safety: test harness is single-threaded; env is scoped to this test.
        unsafe { std::env::set_var("XDG_CONFIG_HOME", &dir) };

        assert!(install("zig"), "install zig");
        assert!(installed().iter().any(|e| e.id == "zig"));
        let dl = dynamic_language_for_path(Path::new("foo.zig")).expect("zig resolved");
        assert_eq!(dl.id, "zig");

        let (tokens, _) = tokenize_line(
            Language::Custom,
            Some(&dl.id),
            "const x: u32 = 3; // hello",
            Default::default(),
        );
        assert!(
            tokens.iter().any(|t| t.kind == TokenKind::Keyword),
            "keyword should be highlighted"
        );
        assert!(
            tokens.iter().any(|t| t.kind == TokenKind::Comment),
            "comment should be highlighted"
        );

        assert!(uninstall("zig"));
        assert!(!installed().iter().any(|e| e.id == "zig"));
        assert!(dynamic_language_for_path(Path::new("foo.zig")).is_none());

        let _ = std::fs::remove_dir_all(&dir);
        unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    }
}