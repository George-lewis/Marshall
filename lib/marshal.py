import dataclasses

# this is an internal implementation detail of dataclasses
from dataclasses import _FIELDS as DATACLASS_FIELDS, _FIELD as DATACLASS_FIELD
from dataclasses import Field
from inspect import get_annotations
from types import UnionType
from typing import Any, Self, get_origin, get_args

ENUM_VARIANT_UNIT = 0
ENUM_VARIANT_TUPLE = 1
ENUM_VARIANT_STRUCT = 2


def fields(dataklass: type) -> dict[str, Field]:

    # this is an internal implementation detail of dataclasses
    return getattr(dataklass, DATACLASS_FIELDS)


def deserialize(klass: type, data: dict) -> Self:

    # handle special recursive case
    if not dataclasses.is_dataclass(klass):
        return data

    # be careful with this dict
    # if we modify it, we modify the class!
    fields_ = fields(klass)

    d = {}

    for k, v in data.items():
        if not (field := fields_.get(k)):
            continue

        ty = field.type

        # if the field is a dataclass
        # recurse right away
        if dataclasses.is_dataclass(ty):
            d[k] = deserialize(ty, v)

            continue

        # list[int] -> list
        base = get_origin(ty)

        # tuple[int, str] -> [int, str]
        args = get_args(ty)

        # if the type isn't subscripted
        # these will be None or ()
        # but we're going to assume marshall
        # generated the types, and so they _are_ subscripted
        # otherwise it's a primitive type and we
        # don't need to do anything special

        if base == UnionType:
            # this is an enum

            for cls in args:
                if enum_data := getattr(cls, "ENUM_DATA", None):
                    variant, tag = enum_data

                    if isinstance(v, str):
                        if variant == ENUM_VARIANT_UNIT:
                            if tag == v:
                                d[k] = cls()
                                print(d)
                                break
                    elif variant == ENUM_VARIANT_TUPLE:
                        if tag in v:
                            d[k] = cls(*v[tag])
                            break
                    elif variant == ENUM_VARIANT_STRUCT:
                        if tag in v:
                            d[k] = cls(**v[tag])
                            break
            else:
                raise ValueError(f"cannot deserialize {v} as {ty}")
        elif base == tuple:
            if not isinstance(v, tuple | list):
                raise ValueError(f"cannot deserialize {v} as {ty}")
            
            d[k] = tuple(deserialize(t, v) for t, v in zip(args, v))
        elif base == list:
            if not isinstance(v, list):
                raise ValueError(f"cannot deserialize {v} as {ty}")

            d[k] = [deserialize(args[0], i) for i in v]
        elif base == dict:
            if not isinstance(v, dict):
                raise ValueError(f"cannot deserialize {v} as {ty}")

            d[k] = {
                deserialize(args[0], k): deserialize(args[1], v) for k, v in v.items()
            }
        else:
            d[k] = v
    
    print(d)

    return klass(**d)


def dataclass(cls):
    klass = dataclasses.dataclass(cls)

    fields_: dict[str, Field] = fields(klass)

    # note: this is a hack
    # we're adding a new field to the dataclass
    # this causes the __class__ field to be provided in `asdict()`
    # which we'll hook into in the custom dict factory
    field = dataclasses.field()
    field.name = "__class__"
    field._field_type = DATACLASS_FIELD

    fields_["__class__"] = field

    klass.deserialize = lambda data: deserialize(klass, data)

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

class TupleVariant:
    def __getitem__(self, idx):
        return getattr(self, f"_{idx}")

    def __setitem__(self, idx, value):
        return setattr(self, f"_{idx}", value)

    def __len__(self):
        return len(self.__annotations__)

    def __str__(self):
        return str(self.as_tuple())
    
    def as_tuple(self):
        return tuple(self[i] for i in range(len(self)))
