use cpp_demangle::Symbol;
use rustc_demangle::try_demangle;

pub(super) fn demangle(name: &str) -> Option<(String, String)> {
    if let Ok(demangled) = try_demangle(name) {
        // Format with {:#} to get the string without the hash
        Some((format!("{demangled:#}"), String::from("Rust")))
    } else if let Ok(demangled) = Symbol::new(name) {
        Some((demangled.to_string(), String::from("C++")))
    } else {
        None
    }
}
