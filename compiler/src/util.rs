use crate::lang::GeneratorError;

pub fn for_sep<F, I : IntoIterator, E1, E2>(f: &mut F, iter: I, mut sep: impl FnMut(&mut F) -> Result<(), E1>, mut body: impl FnMut(&mut F, I::Item) -> Result<(), E2>) -> Result<(), GeneratorError> where GeneratorError : From<E1> + From<E2> {
    let mut iter = iter.into_iter();
    if let Some(item) = iter.next() {
        body(f, item)?;
        while let Some(item) = iter.next() {
            sep(f)?;
            body(f, item)?;
        }
    }

    Ok(())
}
