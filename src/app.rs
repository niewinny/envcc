use std::collections::HashMap;
use std::path::PathBuf;

use crate::env_vars::EnvVarDef;

pub enum InputMode {
    Normal,
    Editing { buffer: String, cursor: usize },
    ConfirmQuit,
}

pub struct App {
    pub running: bool,
    pub vars: Vec<EnvVarDef>,
    pub filtered_indices: Vec<usize>,
    pub search: String,
    pub var_index: usize,
    pub var_scroll_offset: usize,
    pub desc_scroll: u16,
    pub input_mode: InputMode,
    pub values: HashMap<String, String>,
    pub dirty: bool,
    pub settings_path: PathBuf,
    pub other_settings: serde_json::Value,
}

impl App {
    pub fn new(
        vars: Vec<EnvVarDef>,
        values: HashMap<String, String>,
        other_settings: serde_json::Value,
        settings_path: PathBuf,
    ) -> Self {
        let filtered_indices: Vec<usize> = (0..vars.len()).collect();
        Self {
            running: true,
            vars,
            filtered_indices,
            search: String::new(),
            var_index: 0,
            var_scroll_offset: 0,
            desc_scroll: 0,
            input_mode: InputMode::Normal,
            values,
            dirty: false,
            settings_path,
            other_settings,
        }
    }

    pub fn current_var(&self) -> Option<&EnvVarDef> {
        self.filtered_indices
            .get(self.var_index)
            .and_then(|&i| self.vars.get(i))
    }

    pub fn get_value(&self, name: &str) -> Option<&String> {
        self.values.get(name)
    }

    pub fn toggle_bool(&mut self, name: &str) {
        if self.values.contains_key(name) {
            self.values.remove(name);
        } else {
            self.values.insert(name.to_string(), "1".to_string());
        }
        self.dirty = true;
    }

    pub fn set_value(&mut self, name: &str, value: String) {
        if value.is_empty() {
            self.values.remove(name);
        } else {
            self.values.insert(name.to_string(), value);
        }
        self.dirty = true;
    }

    pub fn clear_value(&mut self, name: &str) {
        self.values.remove(name);
        self.dirty = true;
    }

    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    pub fn update_filter(&mut self) {
        let query = self.search.to_lowercase();
        self.filtered_indices = self
            .vars
            .iter()
            .enumerate()
            .filter(|(_, v)| {
                if query.is_empty() {
                    return true;
                }
                v.name.to_lowercase().contains(&query)
                    || v.description.to_lowercase().contains(&query)
            })
            .map(|(i, _)| i)
            .collect();

        if self.var_index >= self.filtered_indices.len() {
            self.var_index = 0;
        }
        self.var_scroll_offset = 0;
        self.desc_scroll = 0;
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        crate::settings::save_settings(&self.settings_path, &self.values, &self.other_settings)?;
        self.dirty = false;
        Ok(())
    }

    pub fn try_quit(&mut self) {
        if self.dirty {
            self.input_mode = InputMode::ConfirmQuit;
        } else {
            self.running = false;
        }
    }
}
