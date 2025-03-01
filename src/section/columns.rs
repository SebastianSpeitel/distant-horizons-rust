use std::{mem::MaybeUninit, ops::Index};

const WIDTH: usize = super::Section::WIDTH;
const LEN: usize = WIDTH * WIDTH;

#[derive(Debug)]
pub struct Columns<C>([C; LEN]);

impl<C> Columns<C> {
    pub const LEN: usize = WIDTH * WIDTH;

    #[inline]
    pub const fn new(cols: [C; LEN]) -> Self {
        Self(cols)
    }

    #[allow(clippy::debug_assert_with_mut_call)]
    #[inline]
    pub fn try_from_iter<T: IntoIterator<Item = C>>(iter: T) -> Result<Self, isize> {
        let mut iter = iter.into_iter();

        let exact = match iter.size_hint() {
            // Exact number of elements
            (LEN, Some(LEN)) => true,
            // Too many elements or not enough elements
            (n @ LEN.., _) | (_, Some(n @ LEN..)) => {
                return Err(isize::try_from(n)
                    .unwrap_or(isize::MAX)
                    .saturating_sub(LEN.try_into().unwrap_or(isize::MAX)));
            }
            _ => false,
        };

        let mut cols = std::array::from_fn(|_| MaybeUninit::uninit());

        let mut col_iter = cols.iter_mut();

        col_iter.by_ref().zip(iter.by_ref()).for_each(|(col, c)| {
            col.write(c);
        });

        if exact {
            // Ensure size_hint was correct
            assert_eq!(col_iter.len(), 0);
            debug_assert!(iter.next().is_none());
        } else {
            // Uninit columns left
            if col_iter.len() > 0 {
                return Err(-isize::try_from(col_iter.len()).unwrap_or(isize::MAX));
            }
            // More elements in the iterator
            if iter.next().is_some() {
                return Err(isize::try_from(iter.count())
                    .unwrap_or(isize::MAX)
                    .saturating_add(1));
            }
        }

        // Safety: All elements have been initialized
        let cols = unsafe { cols.map(|col| col.assume_init()) };

        Ok(Self(cols))
    }
}

impl<C> Index<(usize, usize)> for Columns<C> {
    type Output = C;

    #[inline]
    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        debug_assert!(x < WIDTH && y < WIDTH);
        debug_assert!(y * WIDTH + x < LEN);
        &self.0[y * WIDTH + x]
    }
}
