pub mod parse_id {
    use bomboni_common::id::Id;

    use crate::error::{CommonError, RequestResult};

    pub fn parse(source: String) -> RequestResult<Id> {
        Ok(source.parse().map_err(|_| CommonError::InvalidId)?)
    }

    pub fn write(id: Id) -> String {
        id.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error::RequestError,
        parse::{RequestParse, RequestResult},
    };
    use bomboni_common::id::Id;
    use bomboni_request_derive::Parse;

    use super::*;

    #[test]
    fn parse_ids() {
        #[derive(Debug, PartialEq, Default)]
        struct Item {
            id: String,
        }
        #[derive(Parse, Debug, PartialEq)]
        #[parse(source=Item, write)]
        struct ParsedItem {
            #[parse(with=parse_id)]
            id: Id,
        }

        assert_eq!(parse_id::parse("F".to_string()).unwrap(), Id::new(15));
        assert!(parse_id::parse("-1".to_string()).is_err());
        assert!(parse_id::parse("x".to_string()).is_err());
        assert!(parse_id::parse(String::new()).is_err());

        assert_eq!(
            ParsedItem::parse(Item {
                id: "F".to_string(),
            })
            .unwrap(),
            ParsedItem { id: Id::new(15) }
        );
        assert_eq!(
            Item::from(ParsedItem { id: Id::new(15) }),
            Item {
                id: "f".to_string()
            }
        );
    }
}
