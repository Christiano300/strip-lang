use std::collections::HashMap;

use crate::ast::{Pattern, Template};

pub fn find_match<'src, 'a>(
    templates: &'a Vec<Template<'src>>,
    input: &str,
) -> Option<(&'a Template<'src>, HashMap<String, String>)> {
    for template in templates {
        let result = match_template(template.pattern.clone(), input);
        if let Ok(variables) = result {
            return Some((template, variables))
        }
    }
    None
}

pub fn match_template(
    mut template: Vec<Pattern>,
    string: &str,
) -> Result<HashMap<String, String>, String> {
    let mut variables = HashMap::new();
    let mut after_wildcard = vec![];
    let wildcard = template
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            if p == &Pattern::Variable("*") {
                Some(i)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if wildcard.len() > 1 {
        return Err("Only one wildcard is allowed".to_string());
    }
    let has_wildcard;
    if wildcard.is_empty() {
        has_wildcard = false;
    } else {
        after_wildcard = template.split_off(wildcard[0] + 1);
        template.pop();
        has_wildcard = true;
    }

    let mut left_pos = 0;
    for t in &template {
        match *t {
            Pattern::Text(text) => {
                if string
                    .match_indices(text)
                    .any(|(i, _)| i == left_pos)
                {
                    left_pos += text.len();
                } else {
                    return Err(format!("Template part {text:?} not found in {string:?} at {left_pos}"));
                }
            }
            Pattern::Variable(name) => {
                let mut value = String::new();
                while let Some(c) = string.get(left_pos..).and_then(|s| s.chars().next()) {
                    if " _".contains(c) {
                        break;
                    }
                    value.push(c);
                    left_pos += 1;
                }
                if value.is_empty() {
                    return Err("empty variable".to_string());
                }
                if variables.contains_key(name) {
                    return Err("variable already exists".to_string());
                }
                variables.insert(name.to_string(), value);
            }
        }
    }
    let mut right_pos = string.len();
    for t in after_wildcard.iter().rev() {
        match *t {
            Pattern::Text(text) => {
                if string
                    .rmatch_indices(text)
                    .any(|(i, _)| i + text.len() == right_pos)
                {
                    right_pos -= text.len();
                } else {
                    return Err(format!("Template part {text:?} not found in {string:?} at {right_pos}"));
                }
            }
            Pattern::Variable(name) => {
                let mut value = String::new();
                while let Some(c) = string.get(..right_pos).and_then(|s| s.chars().last()) {
                    if " _".contains(c) {
                        break;
                    }
                    value.push(c);
                    right_pos -= 1;
                }
                if value.is_empty() {
                    return Err("empty variable".to_string());
                }
                if variables.contains_key(name) {
                    return Err("variable already exists".to_string());
                }
                variables.insert(name.to_string(), value.chars().rev().collect());
            }
        }
    }
    if has_wildcard {
        variables.insert("*".to_string(), string[left_pos..right_pos].to_string());
    } else if left_pos != right_pos {
        return Err("Input not matched completely".to_string());
    }
    Ok(variables)
}
