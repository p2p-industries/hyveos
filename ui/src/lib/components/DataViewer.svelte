<script lang="ts">
  const { data }: { data: Uint8Array } = $props();

  // if data is a utf-8 or utf-16 string, display it as text (check by looking for null bytes)
  // otherwise, display it as a base64-encoded string
  function processUint8Array(data: Uint8Array): string {
    // Helper function to check if a character is valid ASCII
    const isValidAscii = (byte: number): boolean => {
      return byte >= 32 && byte <= 126; // Printable ASCII range
    };

    // Check if the input is valid UTF-8 or UTF-16 (ASCII subset)
    const isUtf8Ascii = (): boolean => {
      for (let i = 0; i < data.length; i++) {
        if (!isValidAscii(data[i])) {
          return false;
        }
      }
      return true;
    };

    // Decode as UTF-8 if valid
    if (isUtf8Ascii()) {
      try {
        const utf8Decoder = new TextDecoder('utf-8');
        return utf8Decoder.decode(data);
      } catch {
        // If decoding fails, fall back to Base64
        return btoa(String.fromCharCode(...data));
      }
    }

    // Fall back to Base64 for non-ASCII content
    return btoa(String.fromCharCode(...data));
  }

  let viewed = processUint8Array(data);
</script>

{viewed}
