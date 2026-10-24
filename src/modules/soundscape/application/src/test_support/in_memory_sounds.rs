use std::sync::Mutex;

use async_trait::async_trait;
use soundscape_domain::{Sound, SoundIcon};

use crate::ports::{NewSound, SoundRepository};

#[derive(Default)]
pub struct InMemorySounds {
    pub sounds: Mutex<Vec<Sound>>,
    /// When set, every operation fails with this message.
    pub failure: Option<String>,
    /// When set, only `create` fails.
    pub fail_create: bool,
    /// When set, only `set_icon` fails.
    pub fail_set_icon: bool,
}

impl InMemorySounds {
    pub fn with(sounds: Vec<Sound>) -> Self {
        Self {
            sounds: Mutex::new(sounds),
            ..Self::default()
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            failure: Some(message.into()),
            ..Self::default()
        }
    }

    pub fn snapshot(&self) -> Vec<Sound> {
        self.sounds.lock().unwrap().clone()
    }

    fn check(&self) -> Result<(), String> {
        self.failure.clone().map_or(Ok(()), Err)
    }
}

#[async_trait]
impl SoundRepository for InMemorySounds {
    async fn list(&self) -> Result<Vec<Sound>, String> {
        self.check()?;
        Ok(self.snapshot())
    }

    async fn find(&self, id: i64) -> Result<Option<Sound>, String> {
        self.check()?;
        Ok(self.snapshot().into_iter().find(|sound| sound.id == id))
    }

    async fn create(&self, new: NewSound) -> Result<Sound, String> {
        self.check()?;
        if self.fail_create {
            return Err("create failed".into());
        }
        let mut sounds = self.sounds.lock().unwrap();
        let mut sound = Sound::new(
            sounds.len() as i64 + 1,
            &new.name,
            new.category_id,
            &new.content_type,
            new.size_bytes,
            &new.storage_key,
            new.duration_ms,
            new.created_at,
        )
        .map_err(|error| error.to_string())?;
        sound.icon_key = new.icon_key;
        sounds.push(sound.clone());
        Ok(sound)
    }

    async fn update(
        &self,
        id: i64,
        name: &str,
        category_id: i64,
        icon_key: Option<&str>,
    ) -> Result<Option<Sound>, String> {
        self.check()?;
        let mut sounds = self.sounds.lock().unwrap();
        let Some(sound) = sounds.iter_mut().find(|sound| sound.id == id) else {
            return Ok(None);
        };
        sound.name = name.into();
        sound.category_id = category_id;
        sound.icon_key = icon_key.map(str::to_owned);
        Ok(Some(sound.clone()))
    }

    async fn set_icon(&self, id: i64, icon: Option<SoundIcon>) -> Result<Option<Sound>, String> {
        self.check()?;
        if self.fail_set_icon {
            return Err("set_icon failed".into());
        }
        let mut sounds = self.sounds.lock().unwrap();
        let Some(sound) = sounds.iter_mut().find(|sound| sound.id == id) else {
            return Ok(None);
        };
        sound.icon = icon;
        Ok(Some(sound.clone()))
    }

    async fn delete_with_mix_items(&self, id: i64) -> Result<Option<Sound>, String> {
        self.check()?;
        let mut sounds = self.sounds.lock().unwrap();
        let Some(index) = sounds.iter().position(|sound| sound.id == id) else {
            return Ok(None);
        };
        Ok(Some(sounds.remove(index)))
    }
}
