use regex::Regex;

pub struct Ban;

impl Action for Ban {
    fn execute(&mut self, args: &[String]) {
        todo!()
    }
}

pub trait Action {
    fn execute(&mut self, args: &[String]);
}

pub struct Command {
    regex: Regex,
    action: Box<dyn Action>,
}

impl Command {
    pub fn new(regex: Regex, action: Box<dyn Action>) -> Self {
        Self { regex, action }
    }

    pub fn parse(&mut self, haystack: &str) -> bool {
        if let Some(captures) = self.regex.captures(haystack) {
            self.action.execute(
                &captures
                    .iter()
                    .map(|cap| cap.unwrap().as_str().to_string())
                    .collect::<Vec<String>>(),
            );
            return true;
        }
        false
    }
}

pub struct Commander {
    commands: Vec<Command>,
}

impl Commander {
    pub fn new(commands: Vec<Command>) -> Self {
        Self { commands }
    }

    pub fn parse(&mut self, haystack: &str) -> bool {
        self.commands.iter_mut().any(|comm| comm.parse(haystack))
    }
}
