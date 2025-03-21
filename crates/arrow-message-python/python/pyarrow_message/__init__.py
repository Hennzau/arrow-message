from .pyarrow_message import *  # noqa: F403

import pyarrow as pa
import numpy as np

from typing import Union
from enum import Enum


class ArrowMessage:
    @classmethod
    def from_arrow(cls, data: pa.Array):
        if data.type == pa.null():
            return None

        if issubclass(cls, Enum):
            value = data.to_numpy(zero_copy_only=False)[0]

            for encoding in cls:
                if encoding.value == value:
                    return encoding

            return None
        else:
            names = [field.name for field in data.type]
            indices = [index for index in data.type.type_codes]

            map = dict(zip(names, indices))

            kwargs = {}
            for name in names:
                field_type = cls.__dataclass_fields__[name].type  # pyright: ignore

                if hasattr(field_type, "__origin__") and field_type.__origin__ is Union:
                    field_type = field_type.__args__[0]

                if issubclass(field_type, np.ndarray):
                    kwargs[name] = data.field(map[name]).to_numpy(zero_copy_only=False)

                elif issubclass(field_type, ArrowMessage):
                    kwargs[name] = field_type.from_arrow(data.field(map[name]))

                else:
                    kwargs[name] = data.field(map[name]).to_numpy(zero_copy_only=False)[
                        0
                    ]

            return cls(**kwargs)

    def to_arrow(self):
        if self is None:
            return pa.array([], type=pa.null())

        if isinstance(self, Enum):
            return pa.array([self.value], type=pa.utf8())

        fields = list(self.__dataclass_fields__.keys())  # pyright: ignore

        children = []
        for field in fields:
            value = getattr(self, field)
            if value is None:
                children.append(pa.array([], type=pa.null()))
            elif isinstance(value, np.ndarray):
                value.flags.writeable = False
                children.append(pa.array(value))
            elif isinstance(value, ArrowMessage):
                children.append(value.to_arrow())
            else:
                children.append(pa.array([value]))

        return pa.UnionArray.from_dense(
            types=pa.array([], type=pa.int8()),
            value_offsets=pa.array([], type=pa.int32()),
            children=children,
            field_names=fields,
        )
