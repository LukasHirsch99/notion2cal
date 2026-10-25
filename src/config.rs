use base64::{Engine, engine::general_purpose::STANDARD};
use secrecy::{SecretString, zeroize::Zeroizing};
use std::{collections::HashMap, env, fs, path::Path};

pub fn secret(name: &str) -> anyhow::Result<SecretString> {
    if let Ok(path) = env::var(format!("{name}_FILE")) {
        let v = fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("cannot read {name}_FILE ({path}): {e}"))?;
        return Ok(SecretString::from(v.trim().to_owned()));
    }
    let v = env::var(name).map_err(|_| anyhow::anyhow!("{name} or {name}_FILE must be set"))?;
    Ok(SecretString::from(v.trim().to_owned()))
}

pub fn env(name: &str) -> anyhow::Result<String> {
    env::var(name).map_err(|_| anyhow::anyhow!("{name} must be set"))
}

pub struct Config {
    pub database_url: SecretString,
    pub client_id: String,
    pub client_secret: SecretString,
    pub redirect_uri: String,
    pub notion_auth_url: String,
    pub api_version: String,
    pub notion_key_dir: String,
    pub notion_key_current: u8,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: secret("DATABASE_URL")?, // DATABASE_URL_FILE in stack.yml
            client_id: env("NOTION_CLIENT_ID")?,   // plain config
            client_secret: secret("NOTION_CLIENT_SECRET")?, // NOTION_CLIENT_SECRET_FILE in stack.yml
            redirect_uri: env("NOTION_REDIRECT_URI")?,
            notion_auth_url: env("NOTION_AUTH_URL")?,
            api_version: env("NOTION_API_VERSION")?,
            notion_key_dir: env("NOTION_KEY_DIR")?,
            notion_key_current: env("NOTION_KEY_CURRENT")?.parse()?,
        })
    }
}

pub struct Keyring {
    pub keys: HashMap<u8, Zeroizing<[u8; 32]>>,
    pub current: u8,
}

impl Keyring {
    pub fn load(dir: &Path, current: u8) -> anyhow::Result<Self> {
        let mut keys = HashMap::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(v) = name
                .strip_prefix("notion_key_v")
                .and_then(|s| s.parse::<u8>().ok())
            else {
                continue;
            };
            let text = Zeroizing::new(fs::read_to_string(entry.path())?);
            let raw = Zeroizing::new(STANDARD.decode(text.trim())?);
            let key: [u8; 32] = raw
                .as_slice()
                .try_into()
                .map_err(|_| anyhow::anyhow!("key v{v} must be exactly 32 bytes"))?;
            keys.insert(v, Zeroizing::new(key));
        }
        anyhow::ensure!(
            keys.contains_key(&current),
            "current key v{current} not found"
        );
        Ok(Self { keys, current })
    }

    pub fn current(&self) -> (u8, &[u8; 32]) {
        (self.current, &self.keys[&self.current])
    }
    pub fn get(&self, v: u8) -> Option<&[u8; 32]> {
        self.keys.get(&v).map(|k| &**k)
    }
}
