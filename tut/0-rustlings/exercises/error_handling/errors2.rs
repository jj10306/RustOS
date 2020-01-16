// errors2.rs
// Say we're writing a game where you can buy items with tokens. All items cost
// 5 tokens, and whenever you purchase items there is a processing fee of 1
// token. A player of the game will type in how many items they want to buy,
// and the `total_cost` function will calculate the total number of tokens.
// Since the player typed in the quantity, though, we get it as a string-- and
// they might have typed anything, not just numbers!

// Right now, this function isn't handling the error case at all (and isn't
// handling the success case properly either). What we want to do is:
// if we call the `parse` function on a string that is not a number, that
// function will return a `ParseIntError`, and in that case, we want to
// immediately return that error from our function and not try to multiply
// and add.

// There are at least two ways to implement this that are both correct-- but
// one is a lot shorter! Execute `rustlings hint errors2` for hints to both ways.

// 1.
// let mut x = String::from("Hello!");
// let y = String::from("World!");
// x = y;

// - In the above code, we say that y's value was moved to x; x now owns "World". 
// The (ptr, len, cap) of y was copied bit-by-bit to x and y was dropped as it is no longer in scope.

// let x = String::from("Hello!");
// let y = x.clone();

// - In the above code, the actual String in the heap "Hello" is copied
// to a new portion of memory. x and y have unique (ptr, len, cap) tuples, with the ptr
// pointing to different portions of the heap that independently contain the string "Hello"

// 2.
// - The String type supports a dynamicly-sized, mutable piece of text that is 
// allocated on the heap due to its size not being known at compile time. Internally, 
// a String is made up of three parts: (ptr, len, cap) and this is what is stored on the stack
// when a variable of type String is declared in a function. The String type is 24 bytes. 
// - &str is an immutable reference to a part of a string. string literals are of type &str. 
// Internally, &str is stored as (ptr, len) and therefore is always 16 bytes.

// 3. 
// &str internally keeps two pieces of data, (ptr, len) to keep track of the 
// portion of a string it is referencing whereas &String is a single pointer to a String, a tuple 
// containing (ptr, len, cap). &str is 16 bytes whereas &String is 8 bytes

// 4. 
// str is an [u8] which is a DST (its size is not known at compile time) whereas 
// [u8;N] has a known size at compile time, thus str and [u8;N] are not the same.

// 5. 
// str and [u8] are the same because a str is simply an array of characters [u8], 
// whose size is not known at compile time.

// 6.
// &str and &[u8] are not the same because &str contains (ptr,len) and therefore 
// can refer to a part of a string, whereas &[u8] is just a pointer and therefore can 
// only refer to an entire string.
  




// I AM DONE

use std::num::ParseIntError;

pub fn total_cost(item_quantity: &str) -> Result<i32, ParseIntError> {
    let processing_fee = 1;
    let cost_per_item = 5;
    let qty = item_quantity.parse::<i32>()?;
    Ok(qty * cost_per_item + processing_fee)
    // match qty {
    //     Ok(quant) => Ok(quant * cost_per_item + processing_fee),
    //     Err(err) => Err(err)
    // }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_quantity_is_a_valid_number() {
        assert_eq!(total_cost("34"), Ok(171));
    }

    #[test]
    fn item_quantity_is_an_invalid_number() {
        assert_eq!(
            total_cost("beep boop").unwrap_err().to_string(),
            "invalid digit found in string"
        );
    }
}
