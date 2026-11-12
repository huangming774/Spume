//! 固定答案的问题归一化。
//!
//! 比较和去重都用小写、合并空白后的问题。展示仍保留站长原来写下的大小写。

const QUESTION_MAX: usize = 240;
const ANSWER_MAX: usize = 4000;

/// 去掉首尾空白，转成小写，并把连续空白收成一个空格。
pub fn normalize_question(value: &str) -> String {
    let mut out = String::new();
    let mut space = false;
    for ch in value.trim().chars() {
        let ch = ch.to_lowercase().next().unwrap_or(ch);
        if ch.is_whitespace() {
            space = true;
            continue;
        }
        if space && !out.is_empty() {
            out.push(' ');
        }
        space = false;
        out.push(ch);
    }
    out
}

/// 问题和答案都要有内容，问题最多 240 字，答案最多 4000 字。
pub fn input_ok(question: &str, answer: &str) -> bool {
    let question = question.trim();
    let answer = answer.trim();
    !question.is_empty()
        && !answer.is_empty()
        && question.chars().count() <= QUESTION_MAX
        && answer.chars().count() <= ANSWER_MAX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn question_folds_case_and_space() {
        assert_eq!(normalize_question("  Hello   WORLD "), "hello world");
        assert_eq!(normalize_question("你好  世界"), "你好 世界");
        assert_eq!(normalize_question("   "), "");
    }

    #[test]
    fn length_limits_follow_the_go_rules() {
        assert!(input_ok("问", "答"));
        assert!(!input_ok(" ", "答"));
        assert!(!input_ok(&"问".repeat(241), "答"));
        assert!(!input_ok("问", &"答".repeat(4001)));
    }
}
