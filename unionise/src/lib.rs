pub use unionise_codegen::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Unionise)]
pub enum TestEnum {
    One,
    Two,
    Three,
}

#[derive(Unionise)]
pub enum TestEnumFields {
    One(i32, #[unionise(CTestEnum)] TestEnum),
    Two,
    Three { #[unionise(CTestEnum)] yahh: TestEnum },
}

#[cfg(test)]
mod tests {
    pub use super::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq, Unionise)]
    pub enum TestEnum {
        One,
        Two,
        Three,
    }

    #[test]
    fn it_works() {
        let test_enum = TestEnum::Two;
        assert_eq!(test_enum, CTestEnum::from(test_enum).into())
    }
}
