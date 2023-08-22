// use std::fmt::Display;
use std::iter::repeat_with;

use itsdangerous::Signer;
use uuid::Uuid;

const MAC: [u8; 10] = [78, 85, 64, 95, 07, 12, 49, 95, 44, 9];

// pub fn hash_identity_string(val: String) -> String {
//     use blake2::digest::{Input, VariableOutput};
//     use blake2::VarBlake2b;
//     let mut hasher = VarBlake2b::new_keyed(&MAC, 32);
//     hasher.input(val.as_bytes());
//     hex::encode(hasher.vec_result())
// }

pub fn sign_string<S>(val: String, signer: &S) -> String
where
    S: Signer,
{
    signer.sign(val)
}

pub fn hash_short_identity_string(val: String) -> String {
    use blake2::digest::{Input, VariableOutput};
    use blake2::VarBlake2b;
    let mut hasher = VarBlake2b::new_keyed(&MAC, 16);
    hasher.input(val.as_bytes());
    hex::encode(hasher.vec_result())
}

// pub fn hash_impl_display(a: impl Display) -> String {
//     let val = format!("{}", a);
//     hash_identity_string(val)
// }

pub fn generate_random_alphanum_string(length: usize) -> String {
    repeat_with(fastrand::alphanumeric).take(length).collect()
}

pub fn generate_v4_uuid() -> Uuid {
    Uuid::new_v4()
}

pub fn parse_uuid(input: &str) -> Result<Uuid, ()> {
    match Uuid::parse_str(input) {
        Ok(uuid) => Ok(uuid),
        Err(_) => return Err(()),
    }
}

#[test]
fn test_hash_short_identity_string() {
    let val = String::from("this is my really long string, I hope this is long enough");
    let hash = hash_short_identity_string(val);
    assert_eq!(hash.len(), 32);
    assert_eq!(hash.as_str(), "96259fb1be87e913e14e8ebfd9589a3a")
}

#[test]
fn test_make_uuid_string() {
    let first_uuid = generate_v4_uuid();
    let repr = first_uuid.to_string();
    assert_eq!(repr.len(), 36);
    let second_uuid = Uuid::parse_str(&*repr).unwrap();
    assert_eq!(first_uuid, second_uuid);
}
