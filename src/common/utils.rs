use std::hash::{Hash, SipHasher, Hasher};

pub fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = SipHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[cfg(test)]
mod test {

    use utils::hash;

    #[test]
    fn test_client_id_hashing() {
        let username = String::from("LifeUser1");
        let hashed_username: u64 = hash(&username.clone());

        println!("SOMETHING HERE");
        println!("Hash: {}", hashed_username);

    }
}
