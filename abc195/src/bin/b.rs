pub mod cio {
    use std::fmt::{self, Debug};
    use std::io::{BufRead, Cursor, Stdin, StdinLock};
    use std::str::FromStr;

    const INITIAL_BUF_SIZE: usize = 1024;

    pub type Result<T, E = Error> = std::result::Result<T, E>;

    #[derive(Debug)]
    pub enum Error {
        Io { source: std::io::Error },
        Utf8 { source: std::str::Utf8Error },
        Parse { message: String },
        Eof,
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl std::error::Error for Error {}

    impl From<std::io::Error> for Error {
        fn from(source: std::io::Error) -> Self {
            Error::Io { source }
        }
    }

    impl From<std::str::Utf8Error> for Error {
        fn from(source: std::str::Utf8Error) -> Self {
            Error::Utf8 { source }
        }
    }

    impl Error {
        fn parse_error<E: Debug>(err: E) -> Self {
            Error::Parse {
                message: format!("{:?}", err),
            }
        }
    }

    pub struct Scanner<R> {
        reader: R,
        buf: Vec<u8>,
        pos: usize,
    }

    impl<'a> From<&'a Stdin> for Scanner<StdinLock<'a>> {
        fn from(stdin: &'a Stdin) -> Self {
            Scanner::new(stdin.lock())
        }
    }

    impl<'a> From<&'a str> for Scanner<Cursor<&'a str>> {
        fn from(s: &'a str) -> Self {
            Scanner::new(Cursor::new(s))
        }
    }

    impl<R> Scanner<R>
        where
            R: BufRead,
    {
        fn new(reader: R) -> Self {
            Self {
                reader,
                buf: Vec::with_capacity(INITIAL_BUF_SIZE),
                pos: 0,
            }
        }

        pub fn scan<T>(&mut self) -> T
            where
                T: FromStr,
                <T as FromStr>::Err: Debug,
        {
            match self.try_scan() {
                Ok(v) => v,
                Err(err) => panic!("{}", err),
            }
        }

        pub fn try_scan<T>(&mut self) -> Result<T>
            where
                T: FromStr,
                <T as FromStr>::Err: Debug,
        {
            if self.buf.is_empty() {
                self.fill_buf()?;
            }

            let mut from = None;

            loop {
                match (self.buf[self.pos], from.is_some()) {
                    // ignore space
                    (b' ', false) => self.pos += 1,

                    // read all, so handle next line
                    (b'\n', false) => self.fill_buf()?,

                    // found target start index
                    (_, false) => {
                        from = Some(self.pos);
                        self.pos += 1;
                    }

                    // found target, try parse
                    (b' ', true) | (b'\n', true) => break,

                    // keep checking
                    (_, true) => self.pos += 1,
                }
            }

            let part = std::str::from_utf8(&self.buf[from.unwrap()..self.pos])?;
            part.parse::<T>().map_err(Error::parse_error)
        }

        pub fn collect<T>(&mut self, size: usize) -> Vec<T>
            where
                T: FromStr,
                <T as FromStr>::Err: Debug,
        {
            match self.try_collect(size) {
                Ok(vec) => vec,
                Err(err) => panic!("{}", err),
            }
        }

        pub fn try_collect<T>(&mut self, size: usize) -> Result<Vec<T>, Error>
            where
                T: FromStr,
                <T as FromStr>::Err: Debug,
        {
            let mut vec = Vec::with_capacity(size);

            for _ in 0..size {
                vec.push(self.try_scan()?);
            }

            Ok(vec)
        }

        pub fn tuple_2<T1, T2>(&mut self) -> (T1, T2)
            where
                T1: FromStr,
                T2: FromStr,
                <T1 as FromStr>::Err: Debug,
                <T2 as FromStr>::Err: Debug,
        {
            match self.try_tuple_2::<T1, T2>() {
                Ok(v) => v,
                Err(err) => panic!("{}", err),
            }
        }

        pub fn try_tuple_2<T1, T2>(&mut self) -> Result<(T1, T2), Error>
            where
                T1: FromStr,
                T2: FromStr,
                <T1 as FromStr>::Err: Debug,
                <T2 as FromStr>::Err: Debug,
        {
            Ok((self.try_scan::<T1>()?, self.try_scan::<T2>()?))
        }

        pub fn tuple_3<T1, T2, T3>(&mut self) -> (T1, T2, T3)
            where
                T1: FromStr,
                T2: FromStr,
                T3: FromStr,
                <T1 as FromStr>::Err: Debug,
                <T2 as FromStr>::Err: Debug,
                <T3 as FromStr>::Err: Debug,
        {
            match self.try_tuple_3::<T1, T2, T3>() {
                Ok(v) => v,
                Err(err) => panic!("{}", err),
            }
        }

        pub fn try_tuple_3<T1, T2, T3>(&mut self) -> Result<(T1, T2, T3), Error>
            where
                T1: FromStr,
                T2: FromStr,
                T3: FromStr,
                <T1 as FromStr>::Err: Debug,
                <T2 as FromStr>::Err: Debug,
                <T3 as FromStr>::Err: Debug,
        {
            Ok((
                self.try_scan::<T1>()?,
                self.try_scan::<T2>()?,
                self.try_scan::<T3>()?,
            ))
        }

        /// read a line from underlying reader and store it in the buffer.
        fn fill_buf(&mut self) -> Result<()> {
            self.buf.clear();
            self.pos = 0;
            if self.reader.read_until(b'\n', &mut self.buf)? == 0 {
                Err(Error::Eof)
            } else {
                // ensure buf end in a newline
                match self.buf.last() {
                    Some(b'\n') => (),
                    Some(_) | None => self.buf.push(b'\n'),
                }
                Ok(())
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn new() {
            let stdin = std::io::stdin();
            let _ = Scanner::from(&stdin);

            let input = "1 2 3";
            let _ = Scanner::from(input);
        }

        #[test]
        fn scan() {
            let input = "1 -20\nABC 30.1\n";
            let mut scanner = Scanner::from(input);

            assert_eq!(scanner.scan::<i64>(), 1);
            assert_eq!(scanner.scan::<i64>(), -20);
            assert_eq!(scanner.scan::<String>(), String::from("ABC"));
            assert_eq!(scanner.scan::<f64>(), 30.1);
        }

        #[test]
        fn eof() {
            let input = "10\n";
            let mut scanner = Scanner::from(input);

            assert_eq!(scanner.scan::<i64>(), 10);
            assert!(matches!(scanner.try_scan::<i64>(), Err(Error::Eof)));
        }

        #[test]
        fn no_newline() {
            let input = "10 20";
            let mut scanner = Scanner::from(input);

            assert_eq!(scanner.scan::<u32>(), 10);
            assert_eq!(scanner.scan::<u32>(), 20);
        }

        #[test]
        fn collect() {
            let input = "A B C";
            let mut scanner = Scanner::from(input);

            assert_eq!(scanner.collect::<char>(3), vec!['A', 'B', 'C']);

            let input = "100 200 300 A B C";
            let mut scanner = Scanner::from(input);

            assert_eq!(scanner.collect::<i64>(3), vec![100, 200, 300]);
            assert_eq!(
                scanner.collect::<String>(3),
                vec![String::from("A"), String::from("B"), String::from("C")]
            );
        }

        #[test]
        fn tuple_2() {
            let input = "A 10";
            let mut scanner = Scanner::from(input);

            assert_eq!(scanner.tuple_2::<char, i8>(), ('A', 10));
        }

        #[test]
        fn tuple_3() {
            let input = "A 10 -2000";
            let mut scanner = Scanner::from(input);

            assert_eq!(scanner.tuple_3::<char, i8, i64>(), ('A', 10, -2000));
        }
    }
}

fn solve_min(a: i64, b: i64, w: i64, acc: i64) -> Option<i64> {
    let acc = acc + 1;
    for i in a..=b {
        match w - i {
            n if n > 0 => {
                if let Some(answer) = solve_min(a, b, w - i, acc) {
                    return Some(answer);
                }
            }
            n if n == 0 => return Some(acc),
            _  => return None,
        }
    }
    None
}

fn solve_max(a: i64, b: i64, w: i64, acc: i64) -> Option<i64> {
    if w == 0 {
        return Some(acc);
    }
    if w < 0 {
        return None;
    }

    let acc = acc + 1;
    for i in (a..=b).rev() {
        if let Some(answer) = solve_max(a, b, w - i, acc) {
            return Some(answer);
        }
    }
    None
}

fn solve(a: i64, b: i64, w: i64) -> Option<(i64, i64)> {
    let answer_min = match solve_min(a, b, w, 0) {
        Some(answer) => answer,
        None => return None,
    };
    let answer_max = match solve_max(a, b, w, 0) {
        Some(answer) => answer,
        None => return None,
    };

    Some((answer_min, answer_max))
}

fn main() {
    let stdin = std::io::stdin();
    let mut scanner = cio::Scanner::from(&stdin);
    let (a, b, w) = scanner.tuple_3::<i64, i64, i64>();

    match solve(a, b, w * 1000) {
        Some((min, max)) => println!("{} {}", max, min),
        None => println!("UNSATISFIABLE"),
    }
}
