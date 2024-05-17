#[derive(Serialize, Deserialize)]
enum Name {
    None,
    First(String),
    FirstLast(String, String),
    FirstMiddleLast {
        first: String,

        #[serde(skip_serializing_if = "is_empty")]
        middle: Vec<String>,

        last: String,
    },
}

#[derive(Serialize, Deserialize)]
struct User {
    name: Name,

    // this is a comment
    #[serde(default, skip_serializing_if = "under_18")]
    age: u32,

    birthday: (u32, u32, u32),
}
