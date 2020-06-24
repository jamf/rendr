mod mustache;
pub use self::mustache::Mustache;

use std::fmt::Display;
use std::fmt::Formatter;
use std::error::Error;
use std::collections::HashMap;

pub trait TemplatingEngine {
    fn render_template(
        &self,
        template: &str,
        values: &HashMap<&str, &str>,
    ) -> Result<String, RenderError>;
}

#[derive(Debug)]
pub struct RenderError {
    inner: Box<dyn Error>,
}

impl Display for RenderError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "template rendering failed: {}", self.inner)?;

        Ok(())
    }
}

impl Error for RenderError {}
