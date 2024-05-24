def enc(value: str | bytes):
    if isinstance(value, str):
        return value.encode('UTF-8')

    return value
