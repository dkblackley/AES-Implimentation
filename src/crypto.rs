use rand::thread_rng;
use rand::Rng;
use std::string;
use std::num::Wrapping;
use std::ptr::null;


pub(crate) fn get_128b_key() -> [u8; 16] {

    let mut arr = [0u8; 16]; // Store 8 bit numbers and store 16 of them.
    thread_rng().try_fill(&mut arr[..]).expect("Ooops!");
    return arr
}

fn rot_word(word: &[u8]) -> [u8; 4] {
    let mut rot_word: [u8; 4] = [0,0,0,0];
    let mut i = 0;

    while i != 4 {
        rot_word[i] = word[(i + 1) % 4]; // a0 = a1, a1 = a2, a2 = a3, a3 = a0
        i += 1;
    }

    return rot_word;
}

fn multiply_GF(mut a: u8, mut b: u8) -> u8 {
    // Multiplication in the GF is defined as a*b ^ p

    let mut p = 0x00;

    for i in 0..8 {
        if 0x01 & b != 0 { // if the rightmost bit is set
            p = p ^ a; // p + a
        }
        b = b >> 0x01;
        let carry = 0x80 & a; //x^7
        a = a << 1;
        if carry != 0 {
            a = a ^ 0x1b;
        }
    }
    return p;
}

fn left_circular_shift(b: u8, shift: i32) -> u8 {
    return (b << shift) | (b >> (8 - shift));
}

fn sub_word(a: [u8; 4]) -> [u8; 4] {

    let mut words: [u8; 4] = [0, 0, 0, 0];

    for i in 0..4 {
        words[i] = affine_transform(a[i]);
    }

    return words;

}

fn affine_transform(c: u8) -> u8 {

    let mut x = find_inverse(c);
    let mut s = x;


    x = x ^ left_circular_shift(s, 1);
    x = x ^ left_circular_shift(s, 2);
    x = x ^ left_circular_shift(s, 3);
    x = x ^ left_circular_shift(s, 4);
    x = x ^ 0x63;

    return x;
}

fn inverse_affine_transform(c: u8) -> u8 {

    let mut x = c;
    let mut s = x;

    x = left_circular_shift(s, 1);
    x = x ^ left_circular_shift(s, 3);
    x = x ^ left_circular_shift(s, 6);
    x = x ^ 0x05;

    x = find_inverse(x);

    return x;
}

fn find_inverse(arr: u8) -> u8 {
    //  Inverse is described over GF(p^n) as a^p^n-2. i.e a's inverse is a^254
    let mut result = arr;

    for i in 1..254 {
        result = multiply_GF(result, arr);
    }
    return result;
}

fn rc(i: u8) -> u8 { // Remember 0 counts as a number!
    if i == 0x01 {
        return i;
    }

    let rc_p = Wrapping(rc(i - 1));

    if rc_p < Wrapping(0x80) {
        return rc_p.0 * 2;
    } else if rc_p >= Wrapping(0x80) {
        let c:u16 = rc_p.0 as u16;
        return (c * 2 ^ 0x11B) as u8;
    }
    return 0x00; // this will never be reached, but i want to make my if/else statements mirror the formula
}

pub(crate) fn make_keys(encryption_key: [u8; 16], plaintext: &str) -> [[u8; 16]; 11] {

    let mut first_key = [0u8; 16];
    let plaintext_b = <[u8; 16]>::try_from(plaintext.as_bytes()).unwrap();

    for i in 0..16 {
        first_key[i] = encryption_key[i] ^ plaintext_b[i]
    }

    first_key = encryption_key;

    let mut keys = [[0u8; 16]; 11];
    print_key(first_key);
    keys[0] = first_key;

    for i in 1..11 { // make the ten round keys
        let mut key = keys[i-1].clone(); // grab the last key
        let mut last_word: [u8; 4] = [0u8; 4];
        for i in 12..16 {
            last_word[i - 12] = key[i];
        }

        last_word = rot_word(&last_word);
        last_word = sub_word(last_word);
        let rc_i = rc(i as u8);
        last_word[0] = last_word[0] ^ rc_i;

        //Now we XOR the words to make the new key
        let mut next_key: [u8; 16] = [0u8; 16];

        for i in 0..4 { // do the first word manually
            next_key[i] = last_word[i] ^ key[i]
        }

        for i in 4..16 { // jump in 32 bit words
            next_key[i] = next_key[i - 4] ^ key[i]
        }

        keys[i] = next_key;
    }

    return keys;
    
}

pub(crate) fn print_key(arr: [u8; 16]) {

    print!("\nENCRYPTION KEY: ");

    for character in arr {
        print!("{:x?} ", character);
    }
}

fn shift_rows(word: [u8; 4], shift: usize) -> [u8; 4] {

    let mut word_copy = word.clone();

    for i in 0..4 {
        word_copy[i] = word[(i + shift) % 4]
    }

    return word_copy;
}

fn mix_columns(word: [u8; 4]) -> [u8; 4] {

    let MDS: [[u8; 4]; 4] = [
        [2, 3, 1, 1],
        [1, 2, 3, 1],
        [1, 1, 2, 3],
        [3, 1, 1, 2]
    ];

    let mut new_word: [u8; 4] = [0, 0, 0, 0];

    for i in 0..4 {
        let MDS_row = MDS[i];

        let mut result: u8 = multiply_GF(MDS_row[0], word[0]);
        for c in 1..4 {
            let multiple = multiply_GF(MDS_row[c],word[c]);
            result = multiple ^ result;
        }

        new_word[i] = result;
    }

    return new_word;
}

fn inverse_mix_columns(word: [u8; 4]) -> [u8; 4] {

    let MDS: [[u8; 4]; 4] = [
        [14, 11, 13, 9],
        [9, 14, 11, 13],
        [13, 9, 14, 11],
        [11, 13, 9, 14]
    ];

    let mut new_word: [u8; 4] = [0, 0, 0, 0];

    for i in 0..4 {
        let MDS_row = MDS[i];

        let mut result: u8 = multiply_GF(MDS_row[0], word[0]);
        for c in 1..4 {
            let multiple = multiply_GF(MDS_row[c],word[c]);
            result = multiple ^ result;
        }

        new_word[i] = result;
    }

    return new_word;
}

pub(crate) fn encrypt_data(plaintext: [u8; 16], keys: [[u8; 16]; 11]) -> [u8; 16] {
    let mut ciphertext :[u8; 16] = plaintext.clone();
    let encryption_key = keys[0];

    //Perform the initial XOR
    for i in 0..16 {
        ciphertext[i] = ciphertext[i] ^ encryption_key[i];
    }

    for i in 1..11 {
        // Perform the S-Box
        for c in 0..16 {
            ciphertext[c] = affine_transform(ciphertext[c]);
        }

        //Perform the row shift
        for c in 0..4 {
            let mut word : [u8; 4] = <[u8; 4]>::try_from(&ciphertext[c * 4..(c + 1) * 4]).unwrap();

            for y in 0..4 {
                word[y] = ciphertext[c + (y * 4)];
            }

            let shift_word = shift_rows(<[u8; 4]>::try_from(word).unwrap(), c);
            for y in 0..4 {
                ciphertext[c + (y * 4)] = shift_word[y]
            }
        }
        if i != 10 { // Skip the mix column for the last round
            // Mix the columns
            for c in 0..4 {
                let mut column: [u8; 4] = [0, 0, 0, 0];

                for y in 0..4 {
                    column[y] = ciphertext[(c * 4) + y];
                }

                let mixed_column = mix_columns(column);

                for y in 0..4 {
                    ciphertext[(c * 4) + y] = mixed_column[y];
                }
            }
        }

        //And finally, XOR
        for c in 0..16 {
            ciphertext[c] = ciphertext[c] ^ keys[i][c]
        }

    }

    return ciphertext;
}

pub(crate) fn decrypt_data(ciphertext: [u8; 16], keys: [[u8; 16]; 11]) -> [u8; 16] {
    let mut plaintext :[u8; 16] = ciphertext.clone();
    let encryption_key = keys[10];

    //Perform the initial XOR
    for i in 0..16 {
        plaintext[i] = plaintext[i] ^ encryption_key[i];
    }

    for i in (0..10).rev() {

        //Perform the inverse row shift
        for c in 0..4 {
            let mut word : [u8; 4] = <[u8; 4]>::try_from(&plaintext[c * 4..(c + 1) * 4]).unwrap();

            for y in 0..4 {
                word[y] = plaintext[c + (y * 4)];
            }

            // reverse the shift  by subtracting from 4, 0 = 4 (A full shift) 1 = 3 (Total of 4 back to beginning), etc.
            let shift_word = shift_rows(<[u8; 4]>::try_from(word).unwrap(), 4 - c);
            for y in 0..4 {
                plaintext[c + (y * 4)] = shift_word[y]
            }
        }

        // Perform the inverse S-Box
        for c in 0..16 {
            plaintext[c] = inverse_affine_transform(plaintext[c]);
        }

        //XOR with key before mix
        for c in 0..16 {
            plaintext[c] = plaintext[c] ^ keys[i][c]
        }

        if i != 0 { // Skip the mix column for the last round
            // Invert the mix the columns
            for c in 0..4 {
                let mut column: [u8; 4] = [0, 0, 0, 0];

                for y in 0..4 {
                    column[y] = plaintext[(c * 4) + y];
                }

                let mixed_column = inverse_mix_columns(column);

                for y in 0..4 {
                    plaintext[(c * 4) + y] = mixed_column[y];
                }
            }
        }

    }

    return plaintext;
}
//
// pub(crate) fn decrypt_data(ciphertext: String, keys: [[u8; 16]; 10]) -> String {
//
//