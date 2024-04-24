pub mod derive_id {
    use bomboni_common::id::Id;

    use crate::error::{CommonError, RequestResult};

    pub fn parse<S: AsRef<str>>(source: S) -> RequestResult<Id> {
        Ok(source
            .as_ref()
            .parse()
            .map_err(|_| CommonError::InvalidId)?)
    }

    pub fn write(id: Id) -> String {
        id.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse::RequestParse, testing::bomboni};
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
        #[parse(bomboni_crate = bomboni, source = Item, write)]
        struct ParsedItem {
            #[parse(derive { module = derive_id, field = id })]
            id: Id,
        }

        assert_eq!(derive_id::parse("F").unwrap(), Id::new(15));
        assert!(derive_id::parse("-1").is_err());
        assert!(derive_id::parse("x").is_err());
        assert!(derive_id::parse(String::new()).is_err());

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
