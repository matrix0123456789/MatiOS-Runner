use core::error;
use core::result;

#[derive(PartialEq, Eq, Hash)]
pub struct Uuid([u8; 16]);

#[macro_export]
macro_rules! uuid {
    ($uuid:expr) => {{
        const OUTPUT: $crate::Uuid = match $crate::Uuid::try_parse($uuid) {
            $crate::__macro_support::Ok(u) => u,
            $crate::__macro_support::Err(_) => panic!("invalid UUID"),
        };
        OUTPUT
    }};
}
impl Uuid {
    pub const fn as_u128(&self) -> u128 {
        u128::from_be_bytes(*self.as_bytes())
    }

    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    #[inline]
    pub const fn into_bytes(self) -> [u8; 16] {
        self.0
    }

    pub fn parse_str(input: &str) -> Result<Uuid, u8> {
        try_parse(input.as_bytes())
            .map(Uuid::from_bytes)
            .map_err(|_|1)
    }

    // pub const fn from_u128(v: u128) -> Self {
    //     Uuid::from_bytes(v.to_be_bytes())
    // }

    #[inline]
    pub const fn from_bytes(bytes: [u8; 16]) -> Uuid {
        Uuid(bytes)
    }
    #[inline]
    pub const fn from_u128(number: u128) -> Uuid {
        Uuid(u128::to_be_bytes(number))
    }
}

const fn try_parse(input: &'_ [u8]) -> Result<[u8; 16], u8> {
    match (input.len(), input) {
        // Inputs of 32 bytes must be a non-hyphenated UUID
        (32, s) => parse_simple(s, true),
        // Hyphenated UUIDs may be wrapped in various ways:
        // - `{UUID}` for braced UUIDs
        // - `urn:uuid:UUID` for URNs
        // - `UUID` for a regular hyphenated UUID
        (36, s)
        | (38, [b'{', s @ .., b'}'])
        | (45, [b'u', b'r', b'n', b':', b'u', b'u', b'i', b'd', b':', s @ ..]) => {
            parse_hyphenated(s)
        }
        // Any other shaped input is immediately invalid
        _ => Err(1),
    }
}

#[inline]
#[allow(dead_code)]
pub(crate) const fn parse_braced(input: &'_ [u8]) -> Result<[u8; 16], u8> {
    if let (38, [b'{', s @ .., b'}']) = (input.len(), input) {
        parse_hyphenated(s)
    } else {
        Err(1)
    }
}

#[inline]
#[allow(dead_code)]
pub(crate) const fn parse_urn(input: &'_ [u8]) -> Result<[u8; 16], u8> {
    if let (45, [b'u', b'r', b'n', b':', b'u', b'u', b'i', b'd', b':', s @ ..]) =
        (input.len(), input)
    {
        parse_hyphenated(s)
    } else {
        Err(1)
    }
}

#[inline]
pub(crate) const fn parse_simple(
    s: &'_ [u8],
    speculative: bool,
) -> Result<[u8; 16], u8> {
    // This length check here removes all other bounds
    // checks in this function
    if s.len() != 32 {
        return Err(1);
    }

    let mut buf: [u8; 16] = [0; 16];
    let mut i = 0;

    while i < 16 {
        // Convert a two-char hex value (like `A8`)
        // into a byte (like `10101000`)
        let h1 = HEX_TABLE[s[i * 2] as usize];
        let h2 = HEX_TABLE[s[i * 2 + 1] as usize];

        // We use `0xff` as a sentinel value to indicate
        // an invalid hex character sequence (like the letter `G`)
        if h1 | h2 == 0xff {
            return Err(1);
        }

        // The upper nibble needs to be shifted into position
        // to produce the final byte value
        buf[i] = SHL4_TABLE[h1 as usize] | h2;
        i += 1;
    }

    Ok(buf)
}

#[inline]
pub(crate) const fn parse_hyphenated(s: &'_ [u8]) -> Result<[u8; 16], u8> {
    // This length check here removes all other bounds
    // checks in this function
    if s.len() != 36 {
        return Err(1);
    }

    // We look at two hex-encoded values (4 chars) at a time because
    // that's the size of the smallest group in a hyphenated UUID.
    // The indexes we're interested in are:
    //
    // uuid     : 936da01f-9abd-4d9d-80c7-02af85c822a8
    //            |   |   ||   ||   ||   ||   |   |
    // hyphens  : |   |   8|  13|  18|  23|   |   |
    // positions: 0   4    9   14   19   24  28  32

    // First, ensure the hyphens appear in the right places
    match [s[8], s[13], s[18], s[23]] {
        [b'-', b'-', b'-', b'-'] => {}
        _ => return Err(1),
    }

    let positions: [u8; 8] = [0, 4, 9, 14, 19, 24, 28, 32];
    let mut buf: [u8; 16] = [0; 16];
    let mut j = 0;

    while j < 8 {
        let i = positions[j];

        // The decoding here is the same as the simple case
        // We're just dealing with two values instead of one
        let h1 = HEX_TABLE[s[i as usize] as usize];
        let h2 = HEX_TABLE[s[(i + 1) as usize] as usize];
        let h3 = HEX_TABLE[s[(i + 2) as usize] as usize];
        let h4 = HEX_TABLE[s[(i + 3) as usize] as usize];

        if h1 | h2 | h3 | h4 == 0xff {
            return Err(1);
        }

        buf[j * 2] = SHL4_TABLE[h1 as usize] | h2;
        buf[j * 2 + 1] = SHL4_TABLE[h3 as usize] | h4;
        j += 1;
    }

    Ok(buf)
}


const HEX_TABLE: &[u8; 256] = &{
    let mut buf = [0; 256];
    let mut i: u8 = 0;

    loop {
        buf[i as usize] = match i {
            b'0'..=b'9' => i - b'0',
            b'a'..=b'f' => i - b'a' + 10,
            b'A'..=b'F' => i - b'A' + 10,
            _ => 0xff,
        };

        if i == 255 {
            break buf;
        }

        i += 1
    }
};

const SHL4_TABLE: &[u8; 256] = &{
    let mut buf = [0; 256];
    let mut i: u8 = 0;

    loop {
        buf[i as usize] = i.wrapping_shl(4);

        if i == 255 {
            break buf;
        }

        i += 1;
    }
};

impl Clone for Uuid {
    fn clone(&self) -> Self {
        Uuid::from_u128(self.as_u128())
    }
}
impl Copy for Uuid {
    
}