#[cfg(test)]
mod tests {
    use super::*;

    use bomboni_request::{derive::Parse, parse::RequestParse};

    #[test]
    fn it_works() {
        #[derive(Debug, Clone, PartialEq, Default)]
        struct Item {
            value: i32,
        }

        #[derive(Debug, Clone, PartialEq, Default, Parse)]
        #[parse(source = Item, write)]
        struct ParsedItem {
            value: i32,
        }

        assert!(ParsedItem::parse(Item { value: 42 }).is_ok());
    }
}
