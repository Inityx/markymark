#![allow(dead_code)]

mod link;

use self::link::{Link, LinkSet, Token};

use std::{
    cmp::min,
    collections::HashMap,
    fmt,
    str::pattern::Pattern,
};

use rand::Rng;


#[derive(Default)]
pub struct Markov<'src> {
    chain: HashMap<Vec<&'src str>, LinkSet<'src>>,
    entry_points: Vec<&'src str>,
    depth: usize
}

impl<'src> Markov<'src> {
    pub fn with_depth(depth: usize) -> Self {
        Markov {
            depth,
            ..Default::default()
        }
    }

    pub fn train_text(&mut self, text: &'src str, sentence_end: impl Pattern<'src>) {
        text
            .split_terminator(sentence_end)
            .filter(|s| !s.is_empty())
            .for_each(|sentence| self.train_sentence(sentence))
    }

    fn train_sentence(&mut self, sentence: &'src str) {
        let words: Vec<&'src str> = sentence.split_whitespace().collect();
        if words.is_empty() { return; }

        let depth = min(self.depth, words.len() - 1);

        // Train start
        if !self.entry_points.contains(&words[0]) {
            self.entry_points.push(words[0]);
        }

        for width in 1..=depth {
            // Train mids
            for window in words.windows(width + 1) {
                self.train_link(
                    &window[..window.len() - 1],
                    Token::Word(window.last().unwrap())
                ); 
            }

            // Train ends
            self.train_link(
                &words[words.len() - width..],
                Token::End
            );
        }
    }

    fn train_link(&mut self, context: &[&'src str], token: Token<'src>) {
        if let Some(link_set) = self.chain.get_mut(context) {
            link_set.insert(token);
            return;
        }
        
        self
            .chain
            .entry(context.to_vec())
            .or_default()
            .insert(token);
    }

    pub fn generate_sentence(&self, rng: &mut impl Rng) -> String {
        let mut words: Vec<&'src str> = Vec::new();

        // Start seed
        words.push(rng.choose(&self.entry_points).unwrap());

        fn context<'src, 'a>(words: &'a [&'src str], depth: usize) -> &'a [&'src str] {
            &words[words.len().saturating_sub(depth)..]
        };

        while let Token::Word(word) = self.next_word(rng, context(&words, self.depth)) {
            words.push(word);
        }

        words.join(" ") + "."
    }

    fn next_word(
        &self,
        rng: &mut impl Rng,
        context: &[&'src str]
    ) -> Token<'src> {
        let subcontext = |width| &context[context.len() - width..];
        let depth = min(self.depth, context.len());

        let link_sets = (1..=depth).filter_map(|width|
            self.chain.get(subcontext(width))
                .map(|link_set| (width, link_set))
        );

        let mut pooled_links: Vec<Link<'src>> = {
            // LinkSet of the smallest context contains all possible next tokens
            let (_, link_set) = link_sets.clone().next().unwrap();
            let num_links = link_set.len();
            
            Vec::with_capacity(num_links)
        };

        for (width, link_set) in link_sets {
            for mut link in link_set.iter().cloned() {
                link.count *= width; // Emphasize longer contexts

                if let Some(existing) = pooled_links.iter_mut().find(|l| l.token == link.token) {
                    existing.merge(link);
                } else {
                    pooled_links.push(link);
                }
            }
        }

        Self::weighted_selection(rng, &pooled_links).token
    }

    fn weighted_selection(rng: &mut impl Rng, links: &[Link<'src>]) -> Link<'src> {
        let total_count: usize = links.iter().map(|l| l.count).sum();
        links
            .iter()
            .cloned()
            .cycle()
            .skip(rng.gen::<usize>() % total_count)
            .scan(total_count, |remaining, link| {
                *remaining = remaining.saturating_sub(link.count);
                Some((*remaining, link))
            })
            .filter(|(remaining, _)| *remaining == 0)
            .map(|(_, link)| link)
            .next()
            .unwrap()
    }

    pub fn num_contexts_links(&self) -> (usize, usize) {
        (
            self.chain.len(),
            self.chain.iter().map(|(_, v)| v.len()).sum(),
        )
    }
}

impl<'src> fmt::Debug for Markov<'src> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Entry Points: {:#?}\nChain: {:#?}", self.entry_points, self.chain)
    }
}
