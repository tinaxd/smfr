pub mod types;

#[cfg(test)]
mod tests {
    use super::types::message;

    #[test]
    fn from_vlq_1() {
        let bin = vec![0b01111111];
        assert_eq!(message::from_vlq(&bin), 127);
    }

    #[test]
    fn from_vlq_2() {
        let bin = vec![0b11111111, 0b01111111];
        assert_eq!(message::from_vlq(&bin), 16383);
    }

    #[test]
    fn to_vlq_1() {
        let num = 126;
        let expected = vec![0b01111110];
        assert_eq!(message::to_vlq(num), expected);
    }

    #[test]
    fn to_vlq_2() {
        let num = 100000;
        let expected = vec![0b10000110, 0b10001101, 0b00100000];
        assert_eq!(message::to_vlq(num), expected);
    }

    #[test]
    fn to_vlq_3() {
        let num = 1000;
        let expected = vec![0b10000111, 0b01101000];
        assert_eq!(message::to_vlq(num), expected);
    }
}
