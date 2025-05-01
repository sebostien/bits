pub enum GitRefField {
    AuthorName,
    AuthorDateISO,
    RefName,
    ObjectName,
    Subject,
}

impl From<GitRefField> for &str {
    fn from(value: GitRefField) -> Self {
        match value {
            GitRefField::AuthorName => "%(authorname)",
            GitRefField::AuthorDateISO => "%(authordate:iso8601)",
            GitRefField::RefName => "%(refname)",
            GitRefField::ObjectName => "%(objectname)",
            GitRefField::Subject => "%(contents:subject)",
        }
    }
}
