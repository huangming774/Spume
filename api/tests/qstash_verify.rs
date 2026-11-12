use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use taozhiyy_api::qstash::{verify, QStashError, VerifyInput, CLOCK_TOLERANCE_SECS};

fn sign(body: &str, key: &str, url: &str, now: i64) -> String {
    let hash = URL_SAFE_NO_PAD.encode(Sha256::digest(body.as_bytes()));
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT"}"#);
    let payload_json = format!(
        r#"{{"iss":"Upstash","sub":"{url}","exp":{},"nbf":{now},"body":"{hash}"}}"#,
        now + 300
    );
    let payload = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());
    let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes()).expect("hmac key");
    mac.update(header.as_bytes());
    mac.update(b".");
    mac.update(payload.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    format!("{header}.{payload}.{signature}")
}

fn input<'a>(
    signature: &'a str,
    body: &'a str,
    url: &'a str,
    now: i64,
    next: &'a str,
) -> VerifyInput<'a> {
    VerifyInput {
        current_key: "current-key",
        next_key: next,
        signature,
        body: body.as_bytes(),
        url,
        now_unix: now,
    }
}

#[test]
fn current_key_accepts_the_raw_body() {
    let body = r#"{"hello":"world"}"#;
    let url = "https://taozhiyy.top/api/internal/sync/bili";
    let signature = sign(body, "current-key", url, 1_700_000_000);
    assert_eq!(
        verify(&input(&signature, body, url, 1_700_000_000, "")),
        Ok(())
    );
}

#[test]
fn changed_body_is_rejected() {
    let url = "https://taozhiyy.top/api/internal/sync/bili";
    let signature = sign("{}", "current-key", url, 1_700_000_000);
    assert_eq!(
        verify(&input(
            &signature,
            "{\"tampered\":true}",
            url,
            1_700_000_000,
            ""
        )),
        Err(QStashError::Body)
    );
}

#[test]
fn wrong_destination_is_rejected() {
    let url = "https://taozhiyy.top/api/internal/sync/bili";
    let signature = sign("{}", "current-key", url, 1_700_000_000);
    assert_eq!(
        verify(&input(
            &signature,
            "{}",
            "https://evil.example/api/internal/sync/bili",
            1_700_000_000,
            ""
        )),
        Err(QStashError::Subject)
    );
}

#[test]
fn expired_token_is_rejected_outside_tolerance() {
    let url = "https://taozhiyy.top/api/internal/maintenance";
    let signature = sign("{}", "current-key", url, 1_700_000_000);
    let now = 1_700_000_000 + 300 + CLOCK_TOLERANCE_SECS + 1;
    assert_eq!(
        verify(&input(&signature, "{}", url, now, "")),
        Err(QStashError::Time)
    );
}

#[test]
fn next_key_is_accepted_during_rotation() {
    let url = "https://taozhiyy.top/api/internal/sync/radar";
    let signature = sign("{}", "next-key", url, 1_700_000_000);
    assert_eq!(
        verify(&input(&signature, "{}", url, 1_700_000_000, "next-key")),
        Ok(())
    );
}

#[test]
fn none_algorithm_is_rejected() {
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"none","typ":"JWT"}"#);
    let payload = URL_SAFE_NO_PAD
        .encode(br#"{"iss":"Upstash","sub":"https://taozhiyy.top","exp":2,"nbf":1,"body":""}"#);
    let token = format!("{header}.{payload}.aa");
    assert_eq!(
        verify(&input(&token, "", "https://taozhiyy.top", 1, "")),
        Err(QStashError::Malformed)
    );
}
