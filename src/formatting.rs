/**
 * Genera del testo che viene interpretato particolarmente
 * dal markdown di discord
 */

// ES: +50 exp, text: "50 exp"
pub fn positive(text: &str) -> String {
    format!("```diff\n+{}\n```", text)
}

// ES: -50 exp, text: "50 exp"
pub fn negative(text: &str) -> String {
    format!("```diff\n-{}\n```", text)
}

// Incapsula in codice
pub fn code(text: &str) -> String {
    format!("```\n{}\n```", text)
}
