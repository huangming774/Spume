use taozhiyy_api::chat_prompt::{request_body, user_content, PageFields, SYSTEM_PROMPT};

#[test]
fn page_context_is_placed_before_the_question() {
    let content = user_content(
        "What is on this page?",
        "https://taozhiyy.top/about",
        "About Taozhiyy",
        PageFields {
            page_url: "",
            page_title: "",
            page_path: "/about",
            language: "zh-CN",
            site_section: "main",
            headings: &["About", "Tech stack"],
            visible_text:
                "This is the real about page text with origin, tech stack, and contact details.",
        },
    );
    for part in [
        "Current page context",
        "Title: About Taozhiyy",
        "URL: https://taozhiyy.top/about",
        "Path: /about",
        "Section: main",
        "Language: zh-CN",
        "Headings: About | Tech stack",
        "Visible text: This is the real about page text",
        "[User question]",
        "What is on this page?",
    ] {
        assert!(content.contains(part), "{content}");
    }
}

#[test]
fn a_bare_question_stays_a_bare_question() {
    let content = user_content(
        " 你好 ",
        "",
        "",
        PageFields {
            page_url: "",
            page_title: "",
            page_path: "",
            language: "",
            site_section: "",
            headings: &[],
            visible_text: "   ",
        },
    );
    assert_eq!(content, "你好");
}

#[test]
fn request_uses_the_system_prompt_and_token_cap() {
    let body = request_body("deepseek-v4-flash", "你好");
    assert_eq!(body["model"], "deepseek-v4-flash");
    assert_eq!(body["temperature"], 0.7);
    assert_eq!(body["max_tokens"], 400);
    assert_eq!(body["messages"][0]["content"], SYSTEM_PROMPT);
    assert_eq!(body["messages"][1]["content"], "你好");
}
