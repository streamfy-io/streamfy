//
// minimal login token read module that just exposes a
// 'read_streamfy_token' function to read from the current login config
//
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json;

use streamfy_types::defaults::CLI_CONFIG_PATH;

const DEFAULT_LOGINS_DIR: &str = "logins"; // from logins.rs
const CURRENT_LOGIN_FILE_NAME: &str = "current";

type StreamfyToken = String;
type StreamfyRemote = String;

#[derive(Clone, thiserror::Error, Debug)]
pub enum StreamfyCredentialError {
    #[error(
        "no org access token found, please login or switch to an org with 'streamfy cloud org switch'"
    )]
    MissingOrgToken,

    #[error("{0}")]
    Read(String),

    #[error("unable to parse credentials")]
    UnableToParseCredentials,
}

#[derive(Clone)]
pub enum AccessToken {
    V3((StreamfyToken, StreamfyRemote)),
    V4(CliAccessTokens),
}

impl AccessToken {
    pub fn get_token(&self) -> Result<String, StreamfyCredentialError> {
        match self {
            AccessToken::V3((token, _remote)) => Ok(token.clone()),
            AccessToken::V4(token) => Ok(token.get_current_org_token()?),
        }
    }

    pub fn get_remote(&self) -> Result<String, StreamfyCredentialError> {
        match self {
            AccessToken::V3((_token, remote)) => Ok(remote.clone()),
            AccessToken::V4(cli_access_tokens) => Ok(cli_access_tokens.remote.clone()),
        }
    }
}

// multi-org access token output
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CliAccessTokens {
    pub remote: String,
    pub user_access_token: Option<String>,
    pub org_access_tokens: HashMap<String, String>,
}

impl CliAccessTokens {
    pub fn get_current_org_name(&self) -> Result<String, StreamfyCredentialError> {
        let key = self
            .org_access_tokens
            .keys()
            .next()
            .ok_or(StreamfyCredentialError::MissingOrgToken)?;
        Ok(key.to_owned())
    }
    pub fn get_current_org_token(&self) -> Result<String, StreamfyCredentialError> {
        let org = self.get_current_org_name()?;
        let tok = self
            .org_access_tokens
            .get(&org)
            .ok_or(StreamfyCredentialError::MissingOrgToken)?
            .to_owned();
        Ok(tok)
    }
}

/// replaces old read_streamfy_token
pub fn read_access_token() -> Result<AccessToken, StreamfyCredentialError> {
    // read token into cache once
    static TOKEN_CACHE: std::sync::OnceLock<Result<AccessToken, StreamfyCredentialError>> =
        std::sync::OnceLock::new();

    TOKEN_CACHE
        .get_or_init(|| {
            let token = read_access_token_impl();
            match token {
                Ok(AccessToken::V3(_)) => {
                    tracing::debug!("using v3 token");
                }
                Ok(AccessToken::V4(ref cli_access_token)) => {
                    tracing::debug!("using v4 token");
                    println!(
                        "Using org access: {}",
                        cli_access_token.get_current_org_name()?
                    );
                }
                Err(ref err) => {
                    tracing::debug!("failed to read token: {}", err);
                }
            }
            token
        })
        .clone()
}

// replaces old read_streamfy_token
fn read_access_token_impl() -> Result<AccessToken, StreamfyCredentialError> {
    if let Ok(cli_access_tokens) = read_streamfy_token_v4() {
        return Ok(AccessToken::V4(cli_access_tokens));
    }
    let pair = read_streamfy_token_v3()?;
    Ok(AccessToken::V3(pair))
}

pub fn read_streamfy_token_v4() -> Result<CliAccessTokens, StreamfyCredentialError> {
    const CLOUD_BIN: &str = "streamfy-cloud";
    const CLOUD_BIN_V4: &str = "streamfy-cloud-v4";
    let res = read_streamfy_token_v4_cli(CLOUD_BIN_V4);
    if res.is_err() {
        read_streamfy_token_v4_cli(CLOUD_BIN)
    } else {
        res
    }
}

fn read_streamfy_token_v4_cli(cloud_bin: &str) -> Result<CliAccessTokens, StreamfyCredentialError> {
    let mut cmd = std::process::Command::new(cloud_bin);
    cmd.arg("cli-access-tokens");
    cmd.env_remove("RUST_LOG"); // remove RUST_LOG to avoid debug output
    match cmd.output() {
        Ok(output) => {
            let output = String::from_utf8_lossy(&output.stdout);
            let cli_access_tokens: CliAccessTokens =
                serde_json::from_slice(output.as_bytes()).map_err(|e| {
                    tracing::debug!("failed to parse multi-org output: {}\n$ {cloud_bin} cli-access-tokens\n-->>{}<<--", e, output);
                    StreamfyCredentialError::UnableToParseCredentials
                })?;
            tracing::trace!("cli access tokens: {:#?}", cli_access_tokens);
            Ok(cli_access_tokens)
        }
        Err(e) => {
            tracing::debug!("failed to find multi-org login: {}", e);
            Err(StreamfyCredentialError::Read(
                "failed to find multi-org login".to_owned(),
            ))
        }
    }
}

// deprecated, will be removed after multi-org is stable
pub fn read_streamfy_token_v3() -> Result<(StreamfyToken, StreamfyRemote), StreamfyCredentialError>
{
    let cfgpath = default_file_path();
    // this will read the indirection file to resolve the profile
    let cred = Credentials::try_load(cfgpath)?;
    Ok((cred.token, cred.remote))
}

// read remote (older api)
pub fn read_streamfy_token_rem() -> Result<StreamfyRemote, StreamfyCredentialError> {
    let tok = read_access_token()?;
    tok.get_remote()
}

// read token (older api)
pub fn read_streamfy_token() -> Result<StreamfyToken, StreamfyCredentialError> {
    let access = read_access_token()?;
    access.get_token()
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Credentials {
    remote: String,
    email: String,
    id: String,
    token: String,
}

impl Credentials {
    /// Try to load credentials from disk
    fn try_load<P: AsRef<Path>>(base_path: P) -> Result<Self, StreamfyCredentialError> {
        let current_login_path = base_path.as_ref().join(CURRENT_LOGIN_FILE_NAME);
        let cfg_path = fs::read_to_string(current_login_path).map_err(|_| {
            StreamfyCredentialError::Read(
                "no access credentials, try 'streamfy cloud login'".to_owned(),
            )
        })?;
        let cred_path = base_path.as_ref().join(cfg_path);
        Self::load(&cred_path)
    }

    fn load(cred_path: &Path) -> Result<Self, StreamfyCredentialError> {
        let file_str = fs::read_to_string(cred_path).map_err(|_| {
            StreamfyCredentialError::Read(
                "no access credentials, try 'streamfy cloud login'".to_owned(),
            )
        })?;
        let creds: Credentials = toml::from_str(&file_str)
            .map_err(|_| StreamfyCredentialError::UnableToParseCredentials)?;
        Ok(creds)
    }
}

fn default_file_path() -> String {
    let mut login_path = dirs::home_dir().unwrap_or_default();
    login_path.push(CLI_CONFIG_PATH);
    login_path.push(DEFAULT_LOGINS_DIR);
    login_path.to_string_lossy().to_string()
}

#[cfg(test)]
mod streamfy_tok_tests {
    use super::read_streamfy_token;
    use super::CliAccessTokens;
    use serde_json;

    // parse token options
    #[test]
    fn read_token_outputs() {
        let with_uat = r#"
        {
  "remote": "https://streamfy.cloud",
  "user_access_token": "uat_token",
  "org_access_tokens": {
    "inf-billing": "an_org_token"
  }
    }
        "#;

        let cli_access_tokens = serde_json::from_str::<CliAccessTokens>(with_uat);
        assert!(cli_access_tokens.is_ok(), "{cli_access_tokens:?} ");
        let cli_access_tokens = cli_access_tokens.expect("should succeed");
        let org_token = cli_access_tokens
            .get_current_org_token()
            .expect("retreiving org token");
        assert_eq!(org_token, "an_org_token");
        assert_eq!(
            cli_access_tokens.user_access_token,
            Some("uat_token".to_string())
        );

        let no_uat = r#"
        {
  "remote": "https://streamfy.cloud",
  "org_access_tokens": {
    "inf-billing": "an_org_token"
  }
    }
        "#;
        let cli_access_tokens = serde_json::from_str::<CliAccessTokens>(no_uat);
        assert!(cli_access_tokens.is_ok(), "{cli_access_tokens:?} ");
        let cli_access_tokens = cli_access_tokens.expect("should succeed");
        let org_token = cli_access_tokens
            .get_current_org_token()
            .expect("retreiving org token");
        assert_eq!(org_token, "an_org_token");
        assert_eq!(cli_access_tokens.user_access_token, None);
    }

    // load default credentials (ignore by default becasuse config is not populated in ci env)
    #[ignore]
    #[test]
    fn read_default() {
        let res_token = read_streamfy_token();
        assert!(res_token.is_ok(), "{res_token:?}");
        println!("token: {}", res_token.unwrap());
    }
}
