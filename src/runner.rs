use std::{collections::HashMap, fmt::Write, io::Write as IoWrite, iter::repeat, num::Wrapping, vec};

use crate::{
    ast::{Item, Pattern, Template},
    matcher::{find_match, match_template},
};

macro_rules! builtin {
    ($($pattern:ident($val:expr))+, $buffer:ident, |$variables:ident| $code:tt) => {
        if let Ok($variables) = match_template(
            vec![
                $(
                    Pattern::$pattern($val),
                )+
            ],
            $buffer,
        ) {
            return Some($code);
        }
    };
}

const ZERO: Wrapping<u16> = Wrapping('0' as u16);

pub struct Runner<'a> {
    pub templates: Vec<Template<'a>>,
}

#[allow(unused)]
impl<'a> Runner<'a> {
    pub const fn new(source: Vec<Template<'a>>) -> Self {
        Runner { templates: source }
    }

    pub fn run(&self) -> Result<String, String> {
        let entrypoint = self
            .templates
            .iter()
            .find(|t| t.pattern.is_empty())
            .ok_or("No entrypoint")?;
        self.evaluate(entrypoint, &mut HashMap::new())
    }

    pub fn evaluate(
        &self,
        template: &Template,
        context: &mut HashMap<String, String>,
    ) -> Result<String, String> {
        let mut last = String::new();
        for line in &template.body {
            last = self.eval_expr(&line.items, context)?;
        }
        Ok(last)
    }

    fn eval_expr(
        &self,
        line: &Vec<Item>,
        context: &mut HashMap<String, String>,
    ) -> Result<String, String> {
        let mut buffer = String::new();
        for item in line {
            match item {
                Item::Text(text) => {
                    buffer.push_str(text);
                }
                Item::Variable(name) => {
                    if let Some(value) = context.get(&(*name).to_string()) {
                        buffer.push_str(value);
                    } else {
                        return Err(format!("Variable {name} not found in context"));
                    }
                }
                Item::Sub(sub_template) => {
                    let sub_result = self.eval_expr(sub_template, context)?;
                    buffer.push_str(&sub_result);
                }
            }
        }

        self.eval_template(&buffer, context)
    }

    fn eval_template(
        &self,
        buffer: &str,
        context: &mut HashMap<String, String>,
    ) -> Result<String, String> {
        let temp = find_match(&self.templates, buffer);
        if let Some((template, mut variables)) = temp {
            return self.evaluate(template, &mut variables);
        }

        if let Some(value) = self.builtins(buffer, context) {
            return value;
        }

        Err(format!("No match found for {buffer}"))
    }

    fn builtins(
        &self,
        buffer: &str,
        context: &mut HashMap<String, String>,
    ) -> Option<Result<String, String>> {
        builtin!(Variable("name") Text(" = ") Variable("*"), buffer, |variables| {
            self.assign(variables, context)
        });

        builtin!(Variable("string") Text(" ? ") Variable("find"), buffer, |variables| {
            let string = variables.get("string").expect("string not parsed");
            let find = variables.get("find").expect("find not parsed");
            Ok(string
                .find(find)
                .map_or("-1".to_string(), |index| format!("{}", index + 1)))
        });

        builtin!(Variable("string") Text(" @ ") Variable("index"), buffer, |variables| {
            let string = variables.get("string").expect("string not parsed");
            let index = variables
                .get("index")
                .and_then(|i| i.parse::<usize>().ok())
                .expect("index not parsed") - 1;

            Ok(string
                .get(index..=index)
                .map_or("null".to_string(), ToString::to_string))
        });

        builtin!(Text("\"") Variable("*"), buffer, |variables| {
            Ok(variables
                .get("*")
                .expect("Wildcard not parsed")
                .to_string())
        });

        builtin!(Text("print ") Variable("*"), buffer, |variables| {
            let value = variables.get("*").expect("Wildcard not parsed");
            println!("{value}");
            Ok(value.to_string())
        });

        builtin!(Text("input ") Variable("*"), buffer, |variables| {
            let prompt = variables.get("*").expect("Wildcard not parsed");
            let mut input = String::new();
            print!("{prompt}");
            std::io::stdout().flush().expect("Failed to flush stdout");
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            Ok(input.trim().to_string())
        });

        builtin!(Variable("a") Text(" + ") Variable("b"), buffer, |variables| {
            let a = variables.get("a").expect("a not parsed");
            let b = variables.get("b").expect("b not parsed");
            Ok(Self::ascii_add(a, b))
        });

        builtin!(Variable("a") Text(" - ") Variable("b"), buffer, |variables| {
            let a = variables.get("a").expect("a not parsed");
            let b = variables.get("b").expect("b not parsed");
            Ok(Self::ascii_sub(a, b))
        });

        builtin!(Text("show rules"), buffer, |variables| {
            let mut rules = String::new();
            for template in &self.templates {
                writeln!(rules,
                    "{}: {}",
                    template.name.unwrap_or("Unnamed"),
                    template.pattern_fmt()
                );
            }
            eprintln!("{rules}");
            Ok(String::new())
        });

        None
    }

    fn assign(
        &self,
        mut variables: HashMap<String, String>,
        context: &mut HashMap<String, String>,
    ) -> Result<String, String> {
        let value =
            self.eval_template(variables.get("*").expect("value was not parsed"), context)?;
        context.insert(
            variables.remove("name").expect("name was not parsed"),
            value.clone(),
        );
        Ok(value)
    }

    fn ascii_add(a: &str, b: &str) -> String {
        Self::string_digit_operation(a, b, 0, |carry, (a, b)| {
            let sum = a + (b - ZERO) + Wrapping(*carry);
            if sum.0 > '9' as u16 {
                *carry = 1;
                Some(sum.0 - 10)
            } else {
                *carry = 0;
                Some(sum.0)
            }
        })
    }

    fn ascii_sub(a: &str, b: &str) -> String {
        Self::string_digit_operation(a, b, 0, |carry, (a, b)| {
            let diff = a - (b - ZERO) - Wrapping(*carry);
            if diff.0 < '0' as u16 {
                *carry = 1;
                Some(diff.0 + 10)
            } else {
                *carry = 0;
                Some(diff.0)
            }
        })
    }

    fn string_digit_operation<St, F>(a: &str, b: &str, initial_state: St, f: F) -> String
    where
        F: FnMut(&mut St, (Wrapping<u16>, Wrapping<u16>)) -> Option<u16>,
    {
        let first = a
            .chars()
            .rev()
            .chain(repeat('0'))
            .map(|c| Wrapping(c as u16));
        let second = b
            .chars()
            .rev()
            .chain(repeat('0'))
            .map(|c| Wrapping(c as u16));

        let z = first.zip(second).take(a.len().max(b.len()) + 1);
        let result = z.scan(initial_state, f);

        let mut result_string = char::decode_utf16(result)
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<String>();

        while result_string.starts_with('0') && result_string.len() > 1 {
            result_string.remove(0);
        }

        result_string
    }
}
