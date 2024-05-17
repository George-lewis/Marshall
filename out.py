# Generated code

from lib.marshal import *


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
    birthday: tuple[int, int, int | None]
    age: int = 0
