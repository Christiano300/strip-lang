#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template<'a> {
    pub pattern: Vec<Pattern<'a>>,
    pub name: Option<&'a str>,
    pub body: Vec<Line<'a>>,
}

impl Template<'_> {
    pub fn pattern_fmt(&self) -> String {
        format!("({})", self.pattern
            .iter()
            .map(|p| match p {
                Pattern::Text(text) => (*text).to_string(),
                Pattern::Variable(var) => format!("${var}"),
            })
            .collect::<String>())
    }
}
 
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern<'a> {
    Text(&'a str),
    Variable(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line<'a> {
    pub items: Vec<Item<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item<'a> {
    Text(&'a str),
    Variable(&'a str),
    Sub(Vec<Item<'a>>),
}