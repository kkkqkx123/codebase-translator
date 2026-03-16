//! Tree-sitter query builders and predefined queries

pub mod builder;
pub mod comment_queries;
pub mod function_queries;
pub mod string_queries;

pub use builder::QueryBuilder;
pub use comment_queries::CommentQueries;
pub use function_queries::FunctionQueries;
pub use string_queries::StringQueries;
