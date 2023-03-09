use crate::crypto::{make_keys, print_key, get_128b_key, encrypt_data};

mod crypto;

fn main() {
    let plaintext = "Two One Nine Two";
    let encryption_key = get_128b_key();
    let encryption_key = <[u8; 16]>::try_from("Thats my Kung Fu".as_bytes()).unwrap();
    print_key(encryption_key);

    let keys = make_keys(encryption_key, plaintext);
    print!("Done");

    let ciphertext = encrypt_data(<[u8; 16]>::try_from(plaintext.as_bytes()).unwrap(), keys);
    print!("\nCiphertext: {:x?}", ciphertext);
    //
    // let decrypted = decrypt_data(ciphertext, keys);
    // print!("\nDecrypted text: {:x?}", decrypted);
}
