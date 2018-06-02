use std::{
    cmp::Ordering,
    fmt,
    ops::{Deref, DerefMut}
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Token<'src> {
    Word(&'src str),
    End,
}

impl<'src> fmt::Debug for Token<'src> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Word(w) => fmt::Debug::fmt(w, fmt),
            Token::End     => fmt.write_str("$"), 
        }
    }
}

#[derive(Clone, Copy)]
pub struct Link<'src> {
    pub token: Token<'src>,
    pub count: usize,
}

impl<'src> fmt::Debug for Link<'src> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "({:?} => {})", self.token, self.count)
    }
}

impl<'src> Link<'src> {
    pub fn of(token: Token<'src>) -> Self {
        Link { token, count: 1 }
    }

    pub fn merge(&mut self, rhs: Self) {
        debug_assert!(rhs.token == self.token);
        self.count += rhs.count;
    }
}

impl<'a> PartialEq for Link<'a> {
    fn eq(&self, rhs: &Self) -> bool {
        self.count.eq(&rhs.count)
    }
}

impl<'a> Eq for Link<'a> {}

impl<'a> PartialOrd for Link<'a> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl<'a> Ord for Link<'a> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.count.cmp(&rhs.count)
    }
}

#[derive(Default)]
pub struct LinkSet<'src>(Vec<Link<'src>>);

impl<'src> Deref for LinkSet<'src> {
    type Target = Vec<Link<'src>>;

    fn deref(&self) -> &Self::Target {
        let LinkSet(vector) = self;
        vector
    }
}

impl<'src> DerefMut for LinkSet<'src> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let LinkSet(vector) = self;
        vector
    }
}

impl<'src> LinkSet<'src> {
    fn existing(&mut self, token: Token<'src>) -> Option<&mut Link<'src>> {
        self.iter_mut().find(|l| l.token == token)
    }

    pub fn insert(&mut self, token: Token<'src>) {
        let link = Link::of(token);

        if let Some(existing) = self.existing(token) {
            existing.merge(link);
            self.sort_unstable_by(|a, b| b.cmp(a)); // reverse
        } else {
            self.push(link);
        }
    }

    pub fn best(&self) -> Link<'src> {
        self[0]
    }
}

impl<'src> fmt::Debug for LinkSet<'src> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("[")?;
        let mut iter = self.iter();

        if let Some(link) = iter.next() {
            fmt::Debug::fmt(link, fmt)?;
        }

        for link in iter {
            fmt.write_str(", ")?;
            fmt::Debug::fmt(link, fmt)?;
        }

        fmt.write_str("]")
    }
}
