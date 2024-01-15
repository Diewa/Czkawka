fn how_to_traits() {
    #[derive(Debug)]
    struct Typo {
        x: u32
    }

    let t = Typo { x: 5 };

    impl std::fmt::Display for Typo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Typo!")
        }
    }

    println!("{}", t);
    println!("{:?}", t);

    trait Kutas {}

    trait Traito : Kutas {
        fn kutangens() {
            todo!()
        }
    }

    impl Kutas for Typo {}
    impl Traito for Typo {}
}

fn std_errors() {

    #[derive(Debug)]
    struct GuwnoError {}

    impl Error for GuwnoError {}

    impl std::fmt::Display for GuwnoError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Ale guwno")
        }
    }

    fn guwno(x: u32) -> Result<u32, GuwnoError> {
        Err(GuwnoError {})
    }

    // io::Result<String> to alias Result<String, io::Error>

    fn fallo() -> Result<u32, DuzyError> {
        
        // io::std::Error
        let str = fs::read_to_string("plik_ktory_nie_istnieje")?;

        // std::num::ParseIntError
        let num: u32 = str.parse::<u32>()?;

        // GuwnoError
        guwno(num)?;

        Ok(num)
    }

    if let Err(error) = fallo() {

        // Handle only GuwnoError
        if let Some(real_error) = error.downcast_ref::<GuwnoError>() {
            println!("Ale guwno omg, {}", real_error);
            return;
        }
        
        println!("{error}");
    }
    
}

fn thiserror_errors() {
    
    use thiserror::Error;
    
    #[derive(Error, Debug)]
    pub enum DuzyError {
        #[error("nie mozna otworzyc pliku: {source}")]
        OperacjaPlikowa{#[from] source: std::io::Error},
        
        #[error("nie udalo sie sparsowac")]
        ParsingError(#[from] std::num::ParseIntError),
        
        #[error("guwno sie wylalo")]
        GuwnianyError
    }
    
    fn guwno(x: u32) -> Result<u32, DuzyError> {
        Err(DuzyError::GuwnianyError)
    }
    
    fn fallo() -> Result<u32, DuzyError> {

        // io::std::Error
        let str = fs::read_to_string("plik_ktory_nie_istnieje")?;

        // std::num::ParseIntError
        let num: u32 = str.parse::<u32>()?;

        // DuzyError::GuwnianyError
        guwno(num)?;

        Ok(5)
    }

    if let Err(error) = fallo() {
        match error {
            DuzyError::GuwnianyError => {
                println!("Ale guwno omg, {}", error);
                return;
            }

            // All other errors
            e => println!("{e}"),
        }
    }
}