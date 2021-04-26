#![macro_use]

#[macro_export]
macro_rules! for_sep {
    ($var : pat, $iterator : expr, $sep : block, $body : block) => {
        {
            let mut iter = std::iter::IntoIterator::into_iter($iterator);
            if let Some(item) = iter.next() {
                {
                    let $var = item;
                    $body;
                }

                while let Some(item) = iter.next() {
                    $sep;
                    {
                        let $var = item;
                        $body;
                    }
                }
            }
        }
    };
}


pub fn capitalize_identifier(word: &mut str) {
    if let Some(start) = word.get_mut(0..1) {
        start.make_ascii_uppercase()
    }
}

pub fn uncapitalize_identifier(word: &mut str) {
    if let Some(start) = word.get_mut(0..1) {
        start.make_ascii_lowercase()
    }
}


