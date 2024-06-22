use std::fmt::Debug;

pub enum MaybeOwned<'s, BorrowedT, OwnedT>
where
    BorrowedT: ?Sized,
{
    Borrowed(&'s BorrowedT),
    Owned(OwnedT),
}

impl<'s, BorrowedT, OwnedT> AsRef<BorrowedT> for MaybeOwned<'s, BorrowedT, OwnedT>
where
    OwnedT: AsRef<BorrowedT>,
{
    fn as_ref(&self) -> &BorrowedT {
        match self {
            Self::Borrowed(p) => p,
            Self::Owned(s) => s.as_ref(),
        }
    }
}

impl<'s, BorrowedT, OwnedT> From<&'s BorrowedT> for MaybeOwned<'s, BorrowedT, OwnedT> {
    fn from(value: &'s BorrowedT) -> Self {
        MaybeOwned::Borrowed(value)
    }
}

impl<'s, BorrowedT, OwnedT> Debug for MaybeOwned<'s, BorrowedT, OwnedT>
where
    &'s BorrowedT: Debug,
    OwnedT: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Borrowed(arg0) => <&'s BorrowedT as Debug>::fmt(arg0, f),
            Self::Owned(arg0) => <OwnedT as Debug>::fmt(arg0, f),
        }
    }
}

pub type MaybeOwnedString<'s> = MaybeOwned<'s, str, String>;
