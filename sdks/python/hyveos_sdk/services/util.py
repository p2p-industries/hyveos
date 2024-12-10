def enc(value: str | bytes) -> bytes:
    if isinstance(value, str):
        return value.encode('UTF-8')

    return value
