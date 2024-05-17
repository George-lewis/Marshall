import dataclasses

# this is an internal implementation detail of dataclasses
from dataclasses import _FIELDS as DATACLASS_FIELDS, _FIELD as DATACLASS_FIELD
from dataclasses import Field

ENUM_VARIANT_UNIT = 0
ENUM_VARIANT_TUPLE = 1
ENUM_VARIANT_STRUCT = 2

def dataclass(cls):
    klass = dataclasses.dataclass(cls)

    fields: dict[str, Field] = getattr(klass, DATACLASS_FIELDS)

    # note: this is a hack
    # we're adding a new field to the dataclass
    # this causes the __class__ field to be provided in `asdict()`
    # which we'll hook into in the custom dict factory
    field = dataclasses.field()
    field.name = "__class__"
    field._field_type = DATACLASS_FIELD

    fields["__class__"] = field

    return klass

def my_dict_factory(items: list[tuple[str, any]], dict_factory=dict):
    d = dict_factory(items)
    
    if klass := d.pop("__class__", None):
        ...
    else:

        # fallback to the default dict factory
        return d
    
    klass: type = klass

    if skip := getattr(klass, "SKIP_SERIALIZING", None):
        for key in skip:
            del d[key]

    if skip_if := getattr(klass, "SKIP_SERIALIZING_IF", None):
        for key, condition in skip_if.items():
            if condition(d[key]):
                del d[key]

    if rename := getattr(klass, "RENAME", None):
        for old_key, new_key in rename.items():
            d[new_key] = d.pop(old_key)

    if data := getattr(klass, "ENUM_DATA", None):
        variant, tag = data

        if variant == ENUM_VARIANT_UNIT:
            d = tag
        elif variant == ENUM_VARIANT_TUPLE:
            d = {tag: tuple(d.values())}
        elif variant == ENUM_VARIANT_STRUCT:
            d = {tag: d}

    return d

def asdict(obj, dict_factory=dict):
    factory = lambda items: my_dict_factory(items, dict_factory=dict_factory)

    return dataclasses.asdict(obj, dict_factory=factory)

### -- example -- ###

@dataclass
class Phone:
    SKIP_SERIALIZING = {"identifier"}

    number: str
    identifier: str

@dataclass
class User:
    SKIP_SERIALIZING_IF = {
        "age": lambda age: age < 18
    }

    RENAME = {
        "name": "full_name"
    }

    name: str
    age: int
    phone: Phone

# u = User("John", 30, Phone("123-456-7890", "1234"))

# print(asdict(u))

# u = User("John", 15, Phone("123-456-7890", "1234"))

# print(asdict(u))

class A:
    ...

class B:
    ...

@dataclass
class Bar:
    def __new__(cls, data: dict) -> A | B:
        if "A" in data:
            return A()
        if "B" in data:
            return B()

        raise ValueError

@dataclass
class Stuff:
    L: list[Bar]

def under_18(age):
    return age < 18

def is_empty(s):
    return len(s) == 0
