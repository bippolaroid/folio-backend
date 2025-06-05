use std::{
    fs::File,
    io::{BufReader, Error, ErrorKind, Read, Result},
};

pub fn get_local_passkey() -> Result<String> {
    match File::open("./key/pass.key") {
        Ok(file) => {
            let mut buffer = String::new();
            let mut reader = BufReader::new(file);
            match reader.read_to_string(&mut buffer) {
                Ok(_) => Ok(buffer),
                Err(error) => {
                    eprintln!("Could not read key file: {}", error);
                    Err(error)
                }
            }
        }
        Err(error) => {
            eprintln!("Could not open key file: {}", error);
            Err(error)
        }
    }
}

pub fn check_auth(remote_key: String) -> Result<()> {
    let local_key = get_local_passkey().unwrap();
    if local_key == remote_key {
        println!("Accepted token \"{}\"!", remote_key);
        Ok(())
    } else {
        let error = Error::new(
            ErrorKind::ConnectionRefused,
            "Authorization token is incorrect.",
        );
        eprintln!("Failed to authorize: {}", error);
        Err(error)
    }
}
