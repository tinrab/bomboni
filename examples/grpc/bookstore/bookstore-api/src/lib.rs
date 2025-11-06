pub mod model;

#[allow(
    unused_qualifications,
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_html_tags
)]
pub mod v1 {
    bomboni_proto::include_proto!("bookstore.v1");

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        bomboni_proto::include_file_descriptor_set!("bookstore_v1");
}

#[cfg(test)]
mod tests {
    use prost::Name;

    use crate::v1::Book;

    use super::*;

    #[test]
    fn it_works() {
        dbg!(Book::NAME);
    }
}
