use std::fmt;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputResultType {
    Bypass,
    Consume,
    ClearPreedit,
    Preedit,
    Commit,
    CommitBypass,
    CommitPreedit,
    CommitCommit,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct InputResult {
    pub ty: InputResultType,
    pub char1: u32,
    pub char2: u32,
}

impl fmt::Debug for InputResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InputResult")
            .field("ty", &self.ty)
            .field("char1", unsafe {
                &std::char::from_u32_unchecked(self.char1)
            })
            .field("char2", unsafe {
                &std::char::from_u32_unchecked(self.char2)
            })
            .finish()
    }
}

impl InputResult {
    pub const fn clear_preedit() -> Self {
        Self {
            ty: InputResultType::ClearPreedit,
            char1: 0,
            char2: 0,
        }
    }

    pub const fn bypass() -> Self {
        Self {
            ty: InputResultType::Bypass,
            char1: 0,
            char2: 0,
        }
    }

    pub const fn consume() -> Self {
        Self {
            ty: InputResultType::Consume,
            char1: 0,
            char2: 0,
        }
    }

    pub const fn preedit(c: char) -> Self {
        Self {
            ty: InputResultType::Preedit,
            char1: c as u32,
            char2: 0,
        }
    }

    pub const fn commit(c: char) -> Self {
        Self {
            ty: InputResultType::Commit,
            char1: c as u32,
            char2: 0,
        }
    }

    pub const fn commit_bypass(c: char) -> Self {
        Self {
            ty: InputResultType::CommitBypass,
            char1: c as u32,
            char2: 0,
        }
    }

    pub const fn commit_preedit(c: char, p: char) -> Self {
        Self {
            ty: InputResultType::CommitPreedit,
            char1: c as u32,
            char2: p as u32,
        }
    }

    pub fn commit2(c1: char, c2: char) -> Self {
        Self {
            ty: InputResultType::CommitCommit,
            char1: c1 as u32,
            char2: c2 as u32,
        }
    }
}
