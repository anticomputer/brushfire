//! Macro expansion for firejail profile variables.

use std::collections::HashMap;
use std::env;

/// Expands macros in firejail profiles.
#[derive(Debug, Clone)]
pub struct MacroExpander {
    /// Macro variable mappings.
    pub(crate) vars: HashMap<String, String>,
}

impl MacroExpander {
    /// Create a new macro expander with standard firejail macros.
    #[must_use]
    pub fn new() -> Self {
        let mut vars = HashMap::new();

        // Standard firejail macros
        if let Ok(home) = env::var("HOME") {
            vars.insert("HOME".to_string(), home.clone());
            vars.insert("DOWNLOADS".to_string(), format!("{home}/Downloads"));
            vars.insert("DOCUMENTS".to_string(), format!("{home}/Documents"));
            vars.insert("DESKTOP".to_string(), format!("{home}/Desktop"));
            vars.insert("PICTURES".to_string(), format!("{home}/Pictures"));
            vars.insert("MUSIC".to_string(), format!("{home}/Music"));
            vars.insert("VIDEOS".to_string(), format!("{home}/Videos"));
        }

        if let Ok(user) = env::var("USER") {
            vars.insert("USER".to_string(), user.clone());

            // RUNUSER for runtime directory
            if let Ok(xdg_runtime) = env::var("XDG_RUNTIME_DIR") {
                vars.insert("RUNUSER".to_string(), xdg_runtime);
            } else {
                vars.insert("RUNUSER".to_string(), format!("/run/user/{user}"));
            }
        }

        if let Ok(path) = env::var("PATH") {
            vars.insert("PATH".to_string(), path);
        }

        // Config directory
        if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
            vars.insert("CFG".to_string(), xdg_config);
        } else if let Ok(home) = env::var("HOME") {
            vars.insert("CFG".to_string(), format!("{home}/.config"));
        }

        Self { vars }
    }

    /// Expand all macros in a string.
    ///
    /// Replaces `${MACRO}` with the corresponding value.
    #[must_use]
    pub fn expand(&self, input: &str) -> String {
        let mut result = input.to_string();

        for (key, value) in &self.vars {
            let pattern = format!("${{{key}}}");
            result = result.replace(&pattern, value);
        }

        result
    }

    /// Add or override a macro definition.
    pub fn set_macro(&mut self, key: String, value: String) {
        self.vars.insert(key, value);
    }

    /// Get the value of a macro if it exists.
    #[must_use]
    pub fn get_macro(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }
}

impl Default for MacroExpander {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_home() {
        let expander = MacroExpander::new();

        // Should expand ${HOME}
        if let Ok(home) = env::var("HOME") {
            assert_eq!(expander.expand("${HOME}/test"), format!("{home}/test"));
        }
    }

    #[test]
    fn test_expand_multiple() {
        let mut expander = MacroExpander::new();
        expander.set_macro("TEST".to_string(), "value".to_string());

        let result = expander.expand("${TEST} and ${TEST} again");
        assert_eq!(result, "value and value again");
    }

    #[test]
    fn test_no_expansion() {
        let expander = MacroExpander::new();

        // Should not expand unknown macros
        assert_eq!(expander.expand("${UNKNOWN}"), "${UNKNOWN}");
    }
}
