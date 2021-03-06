const TABLE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn encode<T: AsRef<[u8]>>(input: T) -> String {
    let bytes = input.as_ref();
    let mut merge = Vec::with_capacity(bytes.len() * 8);
    for byte in input.as_ref() {
        merge.extend(to_binary(*byte));
    }

    let mut s = String::new();
    let chunks = merge.chunks(6);
    let lack = chunks.len() % 4;

    chunks.for_each(|bin| {
        let i = to_decimal(bin.to_vec()) as usize;
        s.push_str(&TABLE[i..i + 1])
    });

    if lack == 1 || lack == 3 {
        s.push('=');
    } else if lack == 2 {
        s.push('=');
        s.push('=');
    }

    s
}

fn to_decimal(mut vec: Vec<u8>) -> u8 {
    while vec.len() != 6 {
        vec.push(0);
    }
    vec.reverse();
    while vec.len() != 8 {
        vec.push(0);
    }
    vec.iter()
        .enumerate()
        .map(|(i, n)| n * vec![2; i].iter().product::<u8>())
        .sum()
}

fn to_binary(mut n: u8) -> Vec<u8> {
    let mut vec = Vec::with_capacity(8);
    loop {
        if n == 0 {
            break;
        }
        vec.push(n % 2);
        n /= 2;
    }
    while vec.len() != 8 {
        vec.push(0);
    }
    vec.reverse();
    vec
}

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn test_encode() {
        assert_eq!(encode("0"), "MA==");
        assert_eq!(encode("123"), "MTIz");
        assert_eq!(encode("123456789"), "MTIzNDU2Nzg5");
        assert_eq!(encode("abcdefg"), "YWJjZGVmZw==");
    }

    #[test]
    fn to_decimal() {}

    #[test]
    fn to_binary() {}
}
