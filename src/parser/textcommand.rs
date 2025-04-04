#[derive(Clone)]
pub struct TextCommand<'a> {
    words: Box<[&'a str]>,
    ptr: usize,
}
impl<'a> TextCommand<'a> {
    pub fn new(message: &str) -> TextCommand {
        TextCommand {
            words: message.split_whitespace().collect(),
            ptr: 0,
        }
    }

    pub fn first(&self) -> &'a str {
        self.words[0]
    }
}
impl<'a> Iterator for TextCommand<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        if self.ptr >= self.words.len() {
            return None;
        }

        let word = Some(String::from(self.words[self.ptr]));
        self.ptr += 1;

        word
    }
}
