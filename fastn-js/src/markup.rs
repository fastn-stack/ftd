// Copied from ftd/src/executor/markup.rs

const MAGIC: &str = "MMMMMMMMMAMMAMSMASMDASMDAMSDMASMDASDMASMDASDMAASD";

pub static MD: once_cell::sync::Lazy<comrak::ComrakOptions> = once_cell::sync::Lazy::new(|| {
    let mut m = comrak::ComrakOptions::default();
    m.extension.strikethrough = true;
    m.extension.table = true;
    m.extension.autolink = true;
    m.extension.tasklist = true;
    m.extension.superscript = true;
    m.parse.smart = true;
    m
});

fn markup(i: &str) -> String {
    comrak::markdown_to_html(i.replace("![", MAGIC).trim(), &MD)
        .trim()
        .replace(MAGIC, "![")
}

pub fn markup_inline(i: &str) -> String {
    let (space_before, space_after) = spaces(i);
    let o = {
        let mut g = replace_last_occurrence(markup(i).as_str(), "<p>", "");
        g = replace_last_occurrence(g.as_str(), "</p>", "");
        g
    };

    let o = o.replace("</p>", "\n");
    let o = o.replace("<p>", "");

    format!(
        "{}{o}{}",
        repeated_space(space_before),
        repeated_space(space_after)
    )
}

fn repeated_space(n: usize) -> String {
    (0..n).map(|_| " ").collect::<String>()
}

/// find the count of spaces at beginning and end of the input string
fn spaces(s: &str) -> (usize, usize) {
    let mut space_before = 0;
    for (i, c) in s.chars().enumerate() {
        if !c.eq(&' ') {
            space_before = i;
            break;
        }
        space_before = i + 1;
    }
    if space_before.eq(&s.len()) {
        return (space_before, 0);
    }
    let mut space_after = 0;
    for (i, c) in s.chars().rev().enumerate() {
        if !c.eq(&' ') {
            space_after = i;
            break;
        }
        space_after = i + 1;
    }
    (space_before, space_after)
}

fn replace_last_occurrence(s: &str, old_word: &str, new_word: &str) -> String {
    if !s.contains(old_word) {
        return s.to_string();
    }
    if let Some(idx) = s.rsplit(old_word).next() {
        let idx = s.len() - idx.len() - old_word.len();
        return format!("{}{}{}", &s[..idx], new_word, &s[idx + old_word.len()..]);
    }
    s.to_string()
}
