#[derive(Debug, PartialEq)]
pub enum Token {
    Character(String),
    Label(String),
    PlayMusic(String),
    PlaySound(String),
    SceneBg(String),
} //TODO: Complete the AST tree node structure