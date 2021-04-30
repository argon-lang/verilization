#![macro_use]

//! Utility functions that can be useful for generators.



/// Creates a loop with code executed in between each iteration.
/// 
/// ```
/// use verilization_compiler::for_sep;
/// let mut str = String::from("");
/// for_sep!(x, &["a", "b"], { str += ", "; }, {
///     str += x;
/// });
/// assert_eq!(str, "a, b");
/// ```
/// 
/// Loops over the second parameter with the fourth parameter as the body.
/// The third parameter block is executed in between each loop iteration.
/// The first parameter is a pattern for each value.
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

/// Capitalizes the first character of the string.
/// 
/// ```
/// use verilization_compiler::util::capitalize_identifier;
/// let mut word = String::from("hello");
/// capitalize_identifier(&mut word);
/// assert_eq!(word, "Hello");
/// ```
pub fn capitalize_identifier(word: &mut str) {
    if let Some(start) = word.get_mut(0..1) {
        start.make_ascii_uppercase()
    }
}

/// Uncapitalizes the first character of the string.
/// 
/// ```
/// use verilization_compiler::util::uncapitalize_identifier;
/// let mut word = String::from("HELLO");
/// uncapitalize_identifier(&mut word);
/// assert_eq!(word, "hELLO");
/// ```
pub fn uncapitalize_identifier(word: &mut str) {
    if let Some(start) = word.get_mut(0..1) {
        start.make_ascii_lowercase()
    }
}


