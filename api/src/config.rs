#[derive(Clone, Debug)]
pub struct Config {
    pub bind: String,
    pub database_url: Option<String>,
    pub session_secret: String,
    pub owner_email: String,
    pub owner_security_question: String,
    pub owner_security_answer: String,
    /// 站长登录名。不带 @ 的登录输入等于它时按站长邮箱处理。
    pub owner_login_name: String,
    pub cookie_secure: bool,
    pub allowed_origins: Vec<String>,
    pub deepseek_api_key: String,
    pub deepseek_model: String,
    pub deepseek_base_url: String,
    pub dashscope_api_key: String,
    pub dashscope_base_url: String,
    pub dashscope_image_model: String,
    pub agnes_api_key: String,
    pub agnes_base_url: String,
    pub agnes_image_model: String,
    /// Workers AI 和 R2 同一个 Cloudflare 账号，账号 ID 缺省复用 R2 的。
    pub cloudflare_account_id: String,
    pub cloudflare_ai_token: String,
    pub cloudflare_ai_model: String,
    pub bilibili_uid: String,
    pub r2_account_id: String,
    pub r2_access_key: String,
    pub r2_secret_key: String,
    pub r2_bucket: String,
    pub r2_public_base: String,
    pub redis_url: String,
    pub redis_token: String,
    /// Zilliz Cloud 集群公共 REST 端点，例如 `https://xxxx.zillizcloud.com`。
    pub vector_uri: String,
    /// Zilliz Cloud API Key，REST 调用走 `Authorization: Bearer`。
    pub vector_token: String,
    /// 站内内容检索用的 collection 名。
    pub vector_collection: String,
    pub qstash_signing_key: String,
    pub qstash_next_signing_key: String,
    pub public_origin: String,
    pub allow_unsigned_internal: bool,
    pub github_token: String,
    pub github_repo: String,
    pub github_branch: String,
    /// `uid:名字,uid:名字`。空字符串表示用内置的四位 UP。
    pub radar_creators: String,
    pub mail_notify_to: String,
    pub smtp_host: String,
    pub smtp_port: String,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_from: String,
    /// 仓库根目录。空字符串时从当前目录或上一级寻找 `blog/source/_posts`。
    pub content_root: String,
    /// 主留言区是否接受新留言。友链分区不受这个开关影响。
    pub guestbook_main_posts: bool,
    /// 手动同步口令。空字符串时只接受站长会话。
    pub sync_trigger_token: String,
}

impl Config {
    /// 启动时读取 `.env` 文件补全环境变量，行为和 Go 版 `loadOptionalEnvFiles` 一致：
    /// 依次尝试 `/opt/acg-api/.env` 和 `.env`，已有的环境变量不会被覆盖。
    /// `dotenvy` 本身就不覆盖已存在的变量，所以直接按顺序加载即可。
    pub fn load_env_files() {
        for path in ["/opt/acg-api/.env", ".env"] {
            if !std::path::Path::new(path).exists() {
                continue;
            }
            match dotenvy::from_path(path) {
                Ok(_) => tracing::info!("env: loaded {path}"),
                Err(error) => tracing::warn!("env: {path}: {error}"),
            }
        }
    }

    pub fn from_env() -> Self {
        Self::load_env_files();
        Self {
            bind: env("ACG_API_ADDR", ":8080"),
            database_url: nonempty("DATABASE_URL"),
            session_secret: env("SESSION_SECRET", "dev-session-secret"),
            owner_email: env("OWNER_EMAIL", "").to_lowercase(),
            owner_security_question: env("OWNER_SECURITY_QUESTION", "你现在的学号"),
            owner_security_answer: env_first(&["OWNER_SECURITY_ANSWER", "AUTH_OWNER_SECURITY_ANSWER"]),
            owner_login_name: env("OWNER_LOGIN_NAME", "admin"),
            cookie_secure: env("AUTH_COOKIE_SECURE", "true") != "false",
            // 分隔符和 Go 的 configuredCORSOrigins 一致：逗号、空格、换行、制表符。
            allowed_origins: env(
                "ACG_ALLOWED_ORIGINS",
                "https://taozhiyy.top,https://www.taozhiyy.top,http://localhost:5173,http://127.0.0.1:5173",
            )
            .split(|item| matches!(item, ',' | ' ' | '\n' | '\t'))
            .filter(|item| !item.is_empty())
            .map(str::to_string)
            .collect(),
            deepseek_api_key: env("DEEPSEEK_API_KEY", ""),
            deepseek_model: env("DEEPSEEK_MODEL", "deepseek-v4-flash"),
            deepseek_base_url: env("DEEPSEEK_BASE_URL", "https://api.deepseek.com")
                .trim_end_matches('/')
                .to_string(),
            dashscope_api_key: env_first(&["DASHSCOPE_API_KEY", "ALIYUN_BAILIAN_API_KEY"]),
            dashscope_base_url: env(
                "DASHSCOPE_BASE_URL",
                "https://dashscope.aliyuncs.com/api/v1",
            )
            .trim_end_matches('/')
            .to_string(),
            dashscope_image_model: env("AI_IMAGE_MODEL", "z-image-turbo"),
            agnes_api_key: env("AGNES_API_KEY", ""),
            agnes_base_url: env("AGNES_BASE_URL", "https://apihub.agnes-ai.com")
                .trim_end_matches('/')
                .to_string(),
            agnes_image_model: agnes_model_from_env(),
            cloudflare_account_id: env_first(&["CLOUDFLARE_ACCOUNT_ID", "R2_ACCOUNT_ID"]),
            cloudflare_ai_token: env_first(&["CLOUDFLARE_AI_TOKEN", "CLOUDFLARE_API_TOKEN"]),
            cloudflare_ai_model: env(
                "CLOUDFLARE_AI_MODEL",
                "@cf/stabilityai/stable-diffusion-xl-base-1.0",
            ),
            bilibili_uid: env("BILIBILI_UID", "1061280173"),
            r2_account_id: env("R2_ACCOUNT_ID", ""),
            r2_access_key: env("R2_ACCESS_KEY_ID", ""),
            r2_secret_key: env("R2_SECRET_ACCESS_KEY", ""),
            r2_bucket: env("R2_BUCKET", "taozhiyy"),
            r2_public_base: env("R2_PUBLIC_BASE", "https://images.taozhiyy.top")
                .trim_end_matches('/')
                .to_string(),
            redis_url: env("UPSTASH_REDIS_REST_URL", ""),
            redis_token: env("UPSTASH_REDIS_REST_TOKEN", ""),
            vector_uri: env("ZILLIZ_URI", ""),
            vector_token: env_first(&["ZILLIZ_TOKEN", "ZILLIZ_API_KEY"]),
            vector_collection: env("ZILLIZ_COLLECTION", "taozhiyy_search"),
            qstash_signing_key: env("QSTASH_CURRENT_SIGNING_KEY", ""),
            qstash_next_signing_key: env("QSTASH_NEXT_SIGNING_KEY", ""),
            public_origin: env("ACG_PUBLIC_ORIGIN", "https://taozhiyy.top")
                .trim_end_matches('/')
                .to_string(),
            allow_unsigned_internal: env("ACG_ALLOW_UNSIGNED_INTERNAL", "false") == "true",
            github_token: env("GITHUB_TOKEN", ""),
            github_repo: env("GITHUB_REPO", ""),
            github_branch: env("OWNER_PUBLISH_GITHUB_BRANCH", "master"),
            radar_creators: env("RADAR_CREATORS", ""),
            mail_notify_to: env("MAIL_NOTIFY_TO", ""),
            smtp_host: env("SMTP_HOST", ""),
            smtp_port: env("SMTP_PORT", "587"),
            smtp_user: env("SMTP_USER", ""),
            smtp_pass: env("SMTP_PASS", ""),
            smtp_from: env("SMTP_FROM", ""),
            content_root: env("ACG_CONTENT_ROOT", ""),
            guestbook_main_posts: crate::guestbook::posts_enabled(
                &std::env::var("GUESTBOOK_MAIN_POSTS_ENABLED").unwrap_or_default(),
            ),
            sync_trigger_token: env("SYNC_TRIGGER_TOKEN", ""),
        }
    }

    pub fn for_test() -> Self {
        Self {
            bind: "127.0.0.1:0".into(),
            database_url: None,
            session_secret: "test-secret".into(),
            owner_email: "owner@example.com".into(),
            owner_security_question: "你现在的学号".into(),
            owner_security_answer: String::new(),
            owner_login_name: "admin".into(),
            cookie_secure: false,
            allowed_origins: vec!["http://127.0.0.1:5173".into()],
            deepseek_api_key: String::new(),
            deepseek_model: "deepseek-v4-flash".into(),
            deepseek_base_url: "https://api.deepseek.com".into(),
            dashscope_api_key: String::new(),
            dashscope_base_url: "https://dashscope.aliyuncs.com/api/v1".into(),
            dashscope_image_model: "z-image-turbo".into(),
            agnes_api_key: String::new(),
            agnes_base_url: "https://apihub.agnes-ai.com".into(),
            agnes_image_model: "agnes-image-2.1-flash".into(),
            cloudflare_account_id: String::new(),
            cloudflare_ai_token: String::new(),
            cloudflare_ai_model: "@cf/stabilityai/stable-diffusion-xl-base-1.0".into(),
            bilibili_uid: "1061280173".into(),
            r2_account_id: "account".into(),
            r2_access_key: "access".into(),
            r2_secret_key: "secret".into(),
            r2_bucket: "taozhiyy".into(),
            r2_public_base: "https://images.taozhiyy.top".into(),
            redis_url: String::new(),
            redis_token: String::new(),
            vector_uri: String::new(),
            vector_token: String::new(),
            vector_collection: "taozhiyy_search".into(),
            qstash_signing_key: String::new(),
            qstash_next_signing_key: String::new(),
            public_origin: "https://taozhiyy.top".into(),
            allow_unsigned_internal: false,
            github_token: String::new(),
            github_repo: String::new(),
            github_branch: "master".into(),
            radar_creators: String::new(),
            mail_notify_to: String::new(),
            smtp_host: String::new(),
            smtp_port: "587".into(),
            smtp_user: String::new(),
            smtp_pass: String::new(),
            smtp_from: String::new(),
            content_root: String::new(),
            guestbook_main_posts: true,
            sync_trigger_token: String::new(),
        }
    }

    pub fn r2_ready(&self) -> bool {
        !self.r2_account_id.is_empty()
            && !self.r2_access_key.is_empty()
            && !self.r2_secret_key.is_empty()
    }

    pub fn redis_ready(&self) -> bool {
        !self.redis_url.is_empty() && !self.redis_token.is_empty()
    }

    pub fn vector_ready(&self) -> bool {
        !self.vector_uri.is_empty() && !self.vector_token.is_empty()
    }
}

pub fn bind_addr(raw: &str) -> String {
    if let Some(port) = raw.strip_prefix(':') {
        format!("0.0.0.0:{port}")
    } else {
        raw.to_string()
    }
}

fn env(key: &str, fallback: &str) -> String {
    std::env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn nonempty(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn env_first(keys: &[&str]) -> String {
    keys.iter()
        .find_map(|key| nonempty(key))
        .unwrap_or_default()
}

fn agnes_model_from_env() -> String {
    if let Some(model) = nonempty("AGNES_IMAGE_MODEL") {
        return model;
    }
    if let Some(model) = nonempty("AI_IMAGE_MODEL") {
        if model.to_ascii_lowercase().starts_with("agnes-") {
            return model;
        }
    }
    "agnes-image-2.1-flash".to_string()
}
