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

    macro_rules! setup {
        ( $scanner:ident ) => {
            let _stdin = std::io::stdin();
            let mut $scanner = cio::Scanner::from(&_stdin);
        };
    }
    pub(crate) use setup;
}
pub struct UnionFind {
    parent: Vec<Option<usize>>,
    size: Vec<usize>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum UnionResult {
    Unified,
    AlreadyUnified,
}

impl UnionFind {
    /// Create new `UnionFind` of `n` disjoint sets.
    pub fn new(n: usize) -> Self {
        Self {
            parent: vec![None; n],
            size: vec![1; n],
        }
    }

    pub fn union(&mut self, x: usize, y: usize) -> UnionResult {
        if x == y {
            return UnionResult::AlreadyUnified;
        }

        let (large, small) = {
            let (x, y) = (self.root_with_optimize(x), self.root_with_optimize(y));
            if x == y {
                return UnionResult::AlreadyUnified;
            }

            if self.size(x) >= self.size(y) {
                (x, y)
            } else {
                (y, x)
            }
        };

        self.parent[small] = Some(large);
        self.size[large] += self.size[small];
        UnionResult::Unified
    }

    /// Returns `true` if the given elements belong to the same set
    /// and returns `false` otherwise.
    pub fn equiv(&self, x: usize, y: usize) -> bool {
        self.root(x) == self.root(y)
    }

    /// Return the representative for `x`.
    pub fn root(&self, x: usize) -> usize {
        let mut curr = x;
        loop {
            match self.parent[curr] {
                Some(parent) => {
                    curr = parent;
                }
                None => return curr,
            }
        }
    }

    fn root_with_optimize(&mut self, x: usize) -> usize {
        let mut curr = x;
        loop {
            match self.parent[curr] {
                Some(parent) => {
                    self.parent[x] = Some(parent);
                    curr = parent;
                }
                None => return curr,
            }
        }
    }

    pub fn size(&self, x: usize) -> usize {
        self.size[x]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_1() {
        let mut uf = UnionFind::new(3);

        assert_eq!(uf.root(0), 0);
        assert_eq!(uf.root(1), 1);
        assert_eq!(uf.root(2), 2);

        assert!(!uf.equiv(0, 1));
        assert!(!uf.equiv(1, 2));
        assert!(!uf.equiv(2, 0));

        assert_eq!(uf.size(0), 1);
        assert_eq!(uf.size(1), 1);
        assert_eq!(uf.size(2), 1);

        assert_eq!(uf.union(0, 1), UnionResult::Unified);
        assert!(uf.equiv(0, 1));
        assert_eq!(uf.size(0), 2);
    }
}

fn main() {
    cio::setup!(scanner);

    let (n, m) = scanner.tuple_2::<usize, usize>();
    let mut uf = UnionFind::new(n);
    for _ in 0..m {
        let (a, b) = scanner.tuple_2::<usize, usize>();
        uf.union(a - 1, b - 1);
    }

    let mut max = 0;
    for i in 0..n {
        max = std::cmp::max(max, uf.size(i));
    }

    println!("{}", max);
}
