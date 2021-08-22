use rand::Rng;
use std::{
    collections::HashMap,
    fs::File,
    hash::Hash,
    io::{self, Read},
    path::Path,
};

#[derive(Debug, Default)]
struct ChoiceMap<T: Default + Eq + Hash>(HashMap<T, usize>, usize);

impl<T: Default + Eq + Hash> ChoiceMap<T> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn insert(&mut self, val: T) {
        *self.0.entry(val).or_insert(0) += 1;
        self.1 += 1;
    }
    pub fn choose(&self) -> Option<&T> {
        let n: usize = rand::thread_rng().gen_range(0..self.1);
        self.0
            .iter()
            .scan(0, |state, (k, v)| {
                *state += v;
                Some((k, *state))
            })
            .find_map(|(k, v)| (v > n).then(|| k))
    }
    pub fn choose_biased<F>(&self, bias: F) -> Option<&T>
    where
        F: Fn(&T) -> usize,
    {
        let total = self.0.iter().fold(0, |n, (k, v)| n + v * bias(k));
        let n: usize = rand::thread_rng().gen_range(0..total);
        self.0
            .iter()
            .scan(0, |state, (k, v)| {
                *state += v * bias(k);
                Some((k, *state))
            })
            .find_map(|(k, v)| (v > n).then(|| k))
    }
}

#[derive(Debug, Default)]
pub struct Pizzabot {
    first_words: HashMap<String, ChoiceMap<(String, Option<String>)>>,
    words: HashMap<String, ChoiceMap<String>>,
    lengths: ChoiceMap<usize>,
    last_message: HashMap<String, String>,
}

impl Pizzabot {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_message(&mut self, channel: &str, message: &str) {
        self.last_message.insert(channel.into(), message.into());
    }

    pub fn add_message(&mut self, channel: &str, message: &str) {
        let last_message = self.last_message.get(channel).cloned();
        if message.len() == 0 {
            return;
        }
        self.last_message.insert(channel.into(), message.into());
        let iter = message.split(' ');
        let len = iter.clone().count();
        if len < 1 {
            return;
        }
        self.lengths.insert(len);
        let mut iter = iter.peekable();
        let first_word = *iter.peek().unwrap();
        if let Some(last_message) = last_message {
            let mut last_words = last_message.split(' ');
            if let Some(last_word) = last_words.next_back() {
                let last_word2 = last_words.next_back();
                self.first_words
                    .entry(last_word.into())
                    .or_insert_with(ChoiceMap::new)
                    .insert((first_word.into(), last_word2.map(Into::into)));
            }
        }
        let mut iter = message.split(' ').peekable();
        while let (Some(k), Some(&v)) = (iter.next(), iter.peek()) {
            self.words
                .entry(k.into())
                .or_insert_with(ChoiceMap::new)
                .insert(v.into());
        }
    }

    fn is_valid_end(end: &str) -> bool {
        match end.to_ascii_lowercase().as_str() {
            "about" | "as" | "from" | "a" | "he" | "be" | "to" | "wanted" | "want" | "has"
            | "get" | "says" | "most" | "mostly" | "got" | "she" | "just" | "we" | "they"
            | "the" | "of" | "or" | "i" | "ur" | "with" | "your" | "gonna" | "my" | "their"
            | "and" | "it's" | "its" | "but" | "ima" | "what's" | "whats" | "wheres"
            | "where's" | "whos" | "who's" | "an" | "it" | "our" | "hes" | "he's" | "thats"
            | "that's" | "also" | "theres" | "there's" | "ive" | "by" | "theyre" => false,
            w if w.ends_with(',')
                || w.ends_with('&')
                || w.ends_with('-')
                || w.ends_with("'re")
                || w.ends_with("'ll")
                || w.ends_with("'d")
                || w.ends_with("'ve") =>
            {
                false
            }
            _ => true,
        }
    }

    pub fn get_reply(&self, message: &str) -> Option<String> {
        let mut words = message.split(' ');
        let last_word = words.next_back()?;
        let last_word2 = words.next_back();
        let first_word = self
            .first_words
            .get(last_word)
            .and_then(|choices| {
                if let Some(last_word2) = last_word2 {
                    choices.choose_biased(|(_, word2)| {
                        if word2.as_deref() == Some(last_word2) {
                            4
                        } else {
                            1
                        }
                    })
                } else {
                    choices.choose()
                }
            })?
            .0
            .as_str();
        let mut length = *self.lengths.choose()?;
        let mut words = vec![first_word];
        let mut current = first_word;
        while let Some(word) = self
            .words
            .get(&current.to_owned())
            .and_then(|words| words.choose())
        {
            current = word;
            words.push(current);
            length -= 1;
            if length == 0 {
                break;
            }
        }
        for _ in 0..5 {
            if !Self::is_valid_end(current) {
                if let Some(word) = self
                    .words
                    .get(&current.to_owned())
                    .and_then(|words| words.choose())
                {
                    current = word;
                    words.push(current);
                } else {
                    break;
                }
            }
        }
        Some(words.join(" "))
    }

    pub fn load_legacy_file<S: AsRef<str>, P: AsRef<Path>>(
        &mut self,
        channel_id: S,
        path: P,
    ) -> io::Result<()> {
        let mut contents = String::new();
        File::open(path.as_ref())?.read_to_string(&mut contents)?;
        for message in contents.split('\n') {
            const MAGIC: &str = env!("PIZZABOT_MAGIC");
            let ignore = message.starts_with(MAGIC);
            let message = message.replace(MAGIC, "");
            if ignore {
                self.set_message(channel_id.as_ref(), &message);
            } else {
                self.add_message(channel_id.as_ref(), &message);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
