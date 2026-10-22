use crate::ports::display_name_resolver::DisplayNameResolver;
use kernel::UserId;

pub struct FakeProfile;

#[async_trait::async_trait]
impl DisplayNameResolver for FakeProfile {
    async fn display_name(&self, _user: UserId) -> Result<Option<String>, String> {
        Ok(Some("Test User".into()))
    }
}
