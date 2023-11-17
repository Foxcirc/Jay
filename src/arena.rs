
use std::fmt;

const INVALID_ID: &str = "invalid id for this arena";

#[derive(Default)]
pub(crate) struct StrArena {
    pub buffer: Vec<u8>,
}

impl StrArena {

    pub fn put(&mut self, val: &str) -> Id {
        let from = self.buffer.len() as u32;
        let to = from + val.len() as u32 - 1;
        self.buffer.extend_from_slice(val.as_bytes());
        dbg!(from, to, val.len());
        Id { from, to }
    }

    pub fn get<'a>(&'a self, id: Id) -> &'a str {
        dbg!(id.from, id.to);
        std::str::from_utf8(
            &self.buffer.get(id.from as usize .. id.to as usize).expect(INVALID_ID)
        ).expect(INVALID_ID)
    }

}

impl fmt::Debug for StrArena {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StrArena").finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] // todo: remove eq, hash etc. derive
pub(crate) struct Id {
    from: u32,
    to: u32,
}
