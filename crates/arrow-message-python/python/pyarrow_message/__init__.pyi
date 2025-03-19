import pyarrow as pa

from typing import TypeVar, Type, Optional

T = TypeVar("T", bound="ArrowMessage")

class ArrowMessage:
    @classmethod
    def from_arrow(cls: Type[T], arrow_array: pa.Array) -> Optional[T]:
        pass

    def to_arrow(self) -> pa.Array:
        pass
