// create the error type that represents all errors possible in our program
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    SelectedVersionNotFound,
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
