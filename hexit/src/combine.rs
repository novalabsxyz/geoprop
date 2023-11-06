use crate::options::Combine;
use anyhow::Result;

impl Combine {
    pub fn run(&self) -> Result<()> {
        assert!(!self.input.is_empty());
        let res: Result<()> = Ok(());
        res
    }
}
