extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

// Be careful!
// Deserializer do not sanity check so some object might be parsed incorrectly

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    #[serde(flatten)]
    pub payload: Payload,

    #[serde(default)]
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    pub meta: serde_json::Value,

    #[serde(default)]
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    pub jsonapi: serde_json::Value,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub included: Vec<Resource>,
}

impl Document {
    pub fn new(payload: Payload) -> Document {
        Document {
            payload: payload,
            meta: Default::default(),
            jsonapi: Default::default(),
            links: Default::default(),
            included: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Payload {
    #[serde(rename = "data")]
    Data(PrimaryData),

    #[serde(rename = "error")]
    Error(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PrimaryData {
    Single(Resource),
    Multiple(Vec<Resource>),
}

impl PrimaryData {
    fn is_empty(&self) -> bool {
        match self {
            PrimaryData::Multiple(v) if v.len() == 0 => true,
            _ => false,
        }
    }
}

impl Default for PrimaryData {
    fn default() -> Self {
        PrimaryData::Multiple(vec![])
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,

    #[serde(rename = "type")]
    pub type_: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, serde_json::Value>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub relationships: HashMap<String, Relationship>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relationship {
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    links: HashMap<String, Link>,

    #[serde(default)]
    #[serde(skip_serializing_if = "PrimaryData::is_empty")]
    data: PrimaryData,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    meta: HashMap<String, Link>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Links {
    links: HashMap<String, Link>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Link {
    URL(String),

    Object {
        #[serde(default)]
        #[serde(skip_serializing_if = "String::is_empty")]
        href: String,

        #[serde(default)]
        #[serde(skip_serializing_if = "serde_json::Value::is_null")]
        meta: serde_json::Value,
    }
}

#[test]
fn test_minimal_data() {
    let value = json!({
        "data": {
            "type": "articles",
            "id": "1"
        }
    });

    assert!({
        serde_json::from_value::<Document>(value.clone()).unwrap();
        true
    });
}

#[test]
fn test_primary_data() {
    let value = json!({
        "type": "articles",
        "id": "1",
        "attributes": {
            "title": "Rails is Omakase"
        },
        "relationships": {
            "author": {
                "links": {
                    "self": "/articles/1/relationships/author",
                    "related": "/articles/1/author"
                },
                "data": { "type": "people", "id": "9" }
            }
        }
    });

    assert!({
        serde_json::from_value::<PrimaryData>(value.clone()).unwrap();
        true
    });
}

#[test]
fn test_data() {
    let value = json!({
        "data": {
            "type": "articles",
            "id": "1",
        }
    });

    assert!({
        serde_json::from_value::<Document>(value.clone()).unwrap();
        true
    });

    let doc = Document {
        payload: Payload::Data(PrimaryData::Single(Resource {
            id: "1".to_string(),
            type_: "articles".to_string(),
            attributes: Default::default(),
            relationships: Default::default(),
        })),
        meta: Default::default(),
        jsonapi: Default::default(),
        links: Default::default(),
        included: Default::default(),
    };

    assert_eq!(value, serde_json::to_value::<Document>(doc).unwrap());
}

#[test]
fn test_data_2() {
    let value = json!({
        "data": [{
            "type": "articles",
            "id": "1",
            "attributes": {
                "title": "JSON API paints my bikeshed!"
            },
            "links": {
                "self": "http://example.com/articles/1"
            },
            "relationships": {
                "author": {
                    "links": {
                        "self": "http://example.com/articles/1/relationships/author",
                        "related": "http://example.com/articles/1/author"
                    },
                    "data": { "type": "people", "id": "9" }
                },
                "comments": {
                    "links": {
                        "self": "http://example.com/articles/1/relationships/comments",
                        "related": "http://example.com/articles/1/comments"
                    },
                    "data": [
                        { "type": "comments", "id": "5" },
                        { "type": "comments", "id": "12" }
                    ]
                }
            }
        }],
        "included": [{
            "type": "people",
            "id": "9",
            "attributes": {
                "first-name": "Dan",
                "last-name": "Gebhardt",
                "twitter": "dgeb"
            },
            "links": {
                "self": "http://example.com/people/9"
            }
        }, {
            "type": "comments",
            "id": "5",
            "attributes": {
                "body": "First!"
            },
            "relationships": {
                "author": {
                    "data": { "type": "people", "id": "2" }
                }
            },
            "links": {
                "self": "http://example.com/comments/5"
            }
        }, {
            "type": "comments",
            "id": "12",
            "attributes": {
                "body": "I like XML better"
            },
            "relationships": {
                "author": {
                    "data": { "type": "people", "id": "9" }
                }
            },
            "links": {
                "self": "http://example.com/comments/12"
            }
        }]
    });

    assert!({
        serde_json::from_value::<Document>(value.clone()).unwrap();
        true
    })
}

#[test]
fn test_vec_rio() {
    let value = json!([
        {
            "type": "articles",
            "id": "1",
            "attributes": {
                // ... this article's attributes
            },
            "relationships": {
                // ... this article's relationships
            }
        }, {
            "type": "articles",
            "id": "2",
            "attributes": {
                // ... this article's attributes
            },
            "relationships": {
                // ... this article's relationships
            }
        }
    ]);

    assert!({
        serde_json::from_value::<Vec<Resource>>(value.clone()).unwrap();
        true
    });
    assert!({
        serde_json::from_value::<PrimaryData>(value.clone()).unwrap();
        true
    });
}

#[test]
fn test_datum() {
    let value = json!({
        "data": [{
            "type": "articles",
            "id": "1",
            "attributes": {
            },
            "relationships": {
            }
        }, {
            "type": "articles",
            "id": "2",
            "attributes": {
            },
            "relationships": {
            }
        }]
    });

    assert!({
        serde_json::from_value::<Document>(value.clone()).unwrap();
        true
    });
}

#[test]
fn test_link() {
    let value = json!({
        "links": {
            "related": {
                "href": "http://example.com/articles/1/comments",
                "meta": {
                    "count": 10
                }
            }
        }
    });

    assert!({
        serde_json::from_value::<Link>(value.clone()).unwrap();
        true
    });

    let value = json!({
        "links": {
            "self": "http://example.com/posts"
        }
    });

    assert!({
        serde_json::from_value::<Link>(value.clone()).unwrap();
        true
    });
}

#[test]
fn test_ser() {
    let pdata = Payload::Data(PrimaryData::Multiple(vec![
        Resource {
            id: "1".to_string(),
            type_: "articles".to_string(),
            attributes: hashmap!{
                "title" => json!("JSON API paints my bikeshed!"),
                "body" => json!("The shortest article. Ever."),
                "created" => json!("2015-05-22T14:56:29.000Z"),
                "updated" => json!("2015-05-22T14:56:28.000Z")
            }.into_iter().map(|(k,v)| (k.to_string(),v)).collect(),
            relationships: hashmap!{
                "author" => Relationship {
                    data: PrimaryData::Single(
                        Resource {
                            id: "42".to_string(),
                            type_: "people".to_string(),
                            attributes: Default::default(),
                            relationships: Default::default(),
                        }
                    ),
                    links: Default::default(),
                    meta: Default::default(),
                }
            }.into_iter().map(|(k,v)| (k.to_string(),v)).collect(),
        }
    ]));

    assert!(true, pdata);
}

#[test]
fn test_sparse_fieldsets() {
    let value = json!({
        "data": [{
            "type": "articles",
            "id": "1",
            "attributes": {
                "title": "JSON API paints my bikeshed!",
                "body": "The shortest article. Ever.",
                "created": "2015-05-22T14:56:29.000Z",
                "updated": "2015-05-22T14:56:28.000Z"
            },
            "relationships": {
                "author": {
                    "data": {"id": "42", "type": "people"}
                }
            }
        }],
        "included": [
            {
                "type": "people",
                "id": "42",
                "attributes": {
                    "name": "John",
                    "age": 80,
                    "gender": "male"
                }
            }
        ]
    });

    assert!({
        serde_json::from_value::<Document>(value.clone()).unwrap();
        true
    });

    let doc = Document {
        payload: Payload::Data(PrimaryData::Multiple(vec![
            Resource {
                id: "1".to_string(),
                type_: "articles".to_string(),
                attributes: hashmap!{
                    "title" => json!("JSON API paints my bikeshed!"),
                    "body" => json!("The shortest article. Ever."),
                    "created" => json!("2015-05-22T14:56:29.000Z"),
                    "updated" => json!("2015-05-22T14:56:28.000Z")
                }.into_iter().map(|(k,v)| (k.to_string(),v)).collect(),
                relationships: hashmap!{
                    "author" => Relationship {
                        data: PrimaryData::Single(
                            Resource {
                                id: "42".to_string(),
                                type_: "people".to_string(),
                                attributes: Default::default(),
                                relationships: Default::default(),
                            }
                        ),
                        links: Default::default(),
                        meta: Default::default(),
                    }
                }.into_iter().map(|(k,v)| (k.to_string(),v)).collect(),
            }
        ])),
        meta: Default::default(),
        jsonapi: Default::default(),
        links: Default::default(),
        included: vec![
            Resource {
                id: "42".to_string(),
                type_: "people".to_string(),
                attributes: hashmap!{
                    "name" => json!("John"),
                    "age" => json!(80),
                    "gender" => json!("male")
                }.into_iter().map(|(k,v)| (k.to_string(),v)).collect(),
                relationships: Default::default(),
            }
        ],
    };

    assert_eq!(value, serde_json::to_value::<Document>(doc).unwrap());
}

