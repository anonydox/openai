use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: String,
    pub description: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PromptTemplate {
    pub fields: Vec<Field>,
    pub system_input: Option<String>,
    pub user_input: Option<String>,
}

impl PromptTemplate {
    pub fn new(
        fields: Vec<Field>,
        system_input: Option<String>,
        user_input: Option<String>,
    ) -> Self {
        Self {
            fields,
            system_input,
            user_input,
        }
    }

    pub fn generate_prompt(&self, objects: HashMap<String, String>) -> String {
        let mut prompt = String::new();

        if let Some(system_input) = &self.system_input {
            prompt.push_str(&format!("System Input: {}\n", system_input));
        }

        if let Some(user_input) = &self.user_input {
            prompt.push_str(&format!("User Input: {}\n", user_input));
        }

        prompt.push_str("JSON INSTRUCT with Fields:\n");

        for field in &self.fields {
            if let Some(value) = objects.get(&field.name) {
                prompt.push_str(&format!(
                    "{} ({}): {}\n",
                    field.name, field.field_type, value
                ));
            } else {
                prompt.push_str(&format!("{} ({}): \n", field.name, field.field_type));
            }
        }

        prompt
    }
}
