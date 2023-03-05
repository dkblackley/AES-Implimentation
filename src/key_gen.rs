use rand::thread_rng;
use rand::Rng;
use std::string;
use std::num::Wrapping;
use std::ptr::null;


fn get_128b_key() -> [u8; 16] {

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
    //Multiplication in the GF is defined as a*b + p

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

    for i in 0..3 {
        words[i] = affine_transform(a[i]);
    }

    return words;

}

fn affine_transform(c: u8) -> u8 {

    let mut utility_bit = 0x01;
    let vector: [u8; 8] = [0x8F, 0xC7, 0xE3, 0xF1, 0xF8, 0x7C, 0x3E, 0x1F];

    let mut x = find_inverse(c);
    let mut s = x;


    x = x ^ left_circular_shift(s, 1);
    x = x ^ left_circular_shift(s, 2);
    x = x ^ left_circular_shift(s, 3);
    x = x ^ left_circular_shift(s, 4);
    x = x ^ 0x63;

    return x;
}

fn find_inverse(arr: u8) -> u8 {
    //Inverse is described over GF(p^n) as a^p^n-2. i.e a's inverse is a^254
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

pub(crate) fn make_keys() -> [[u8; 16]; 11] {

    let mut keys = [[0u8; 16]; 11];
    let first_key = get_128b_key();
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


        // for w in 0..2 {
        //     key[w] = key[w] ^ last_word[w];
        //     last_word[w] = key[w + 1]
        // }
        // key[3] = key[3] ^ last_word[3]; // do last key manually to avoid invalid memory location

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

    let mut word: Vec<u8> = vec![]; //Empty "ArrayList" of bytes

    print!("ENCRYPTION KEY: ");

    for character in arr {
        print!("{:x?} ", character);
    }
}