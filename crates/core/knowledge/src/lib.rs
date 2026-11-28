//! Knowledge: everything the game can explain about itself, and the links
//! between its pieces.

mod concepts;
mod links;
mod server;

pub use concepts::KnowledgeConcept;
pub use server::KnowledgeBase;
