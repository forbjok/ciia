pub struct SessionInstance;

pub fn session() -> anyhow::Result<Option<SessionInstance>> {
    Ok(Some(SessionInstance))
}
