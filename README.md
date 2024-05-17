# Marshall

This is a 'compiler' that takes in Rust/Serde data types and produces compatible Python data classes

## Example

Start with this...

```rust
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

```

And get something like this...

```python
# custom wrapper over dataclass
from marshall import TupleVariant, dataclass, asdict

@dataclass
class None_:
    ENUM_DATA = (ENUM_VARIANT_UNIT, "None")

@dataclass
class First(TupleVariant):
    ENUM_DATA = (ENUM_VARIANT_TUPLE, "First")

    _0: str

@dataclass
class FirstLast(TupleVariant):
    ENUM_DATA = (ENUM_VARIANT_TUPLE, "FirstLast")

    _0: str
    _1: str

@dataclass
class FirstMiddleLast:
    ENUM_DATA = (ENUM_VARIANT_STRUCT, "FirstMiddleLast")
    SKIP_SERIALIZING_IF = {
        "middle": is_empty,
    }

    first: str
    middle: list[str]
    last: str

Name = None_ | First | FirstLast | FirstMiddleLast

@dataclass
class User:
    SKIP_SERIALIZING_IF = {
        "age": under_18,
    }

    name: Name
    birthday: tuple[int, int, int]
    age: int = 0
```

Then you HAVE mutually compatible types in both languages

So you can dump data with either, and load it with either

### Why would I want this?

You probably don't, and I'm not going to provide any warranty for it :yum:
