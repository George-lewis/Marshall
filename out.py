# Generated code

from lib.marshal import *


@dataclass
class None_:
    ENUM_DATA = (ENUM_VARIANT_UNIT, "None")


@dataclass
class First:
    ENUM_DATA = (ENUM_VARIANT_TUPLE, "First")

    def __getitem__(self, idx):
        return getattr(self, f"_{idx}")

    def __setitem__(self, idx, value):
        return setattr(self, f"_{idx}", value)

    def as_tuple(self):
        return self._0

    def __len__(self):
        return 1

    _0: str


@dataclass
class FirstLast:
    ENUM_DATA = (ENUM_VARIANT_TUPLE, "FirstLast")

    def __getitem__(self, idx):
        return getattr(self, f"_{idx}")

    def __setitem__(self, idx, value):
        return setattr(self, f"_{idx}", value)

    def as_tuple(self):
        return (self._0, self._1)

    def __len__(self):
        return 2

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
