fn ok() -> Result<(), RawError> { Ok(()) }
fn bad() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
fn worse() -> Result<(), Box< dyn Error + Send >> { Ok(()) }
fn render(e: &dyn std::error::Error) {}
// A comment that says Box<dyn Error> is not a hit.
const TEXT: &str = "Box<dyn Error> in a string is not a hit either";
