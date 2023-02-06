mod bytes;
mod int;
mod option_int;
mod struc;
mod value;
mod vec_int;
mod vec_option_int;
mod vec_str;

/* use crate::InitDeserializer;
#[test]
fn stream() {
    let data = b"0 1 ";
    let des = crate::Deserializer::from_slice(data);
    let init_des = InitDeserializer { des: &mut des };
    let mut stream = init_des.into_iter::<Vec<i32>>();

    assert_eq!(0, stream.byte_offset());
    println!("{:?}", stream.next()); // [0]

    assert_eq!(3, stream.byte_offset());
    println!("{:?}", stream.next()); // [1]

    assert_eq!(7, stream.byte_offset());
    println!("{:?}", stream.next()); // error

    assert_eq!(8, stream.byte_offset());
    // If err.is_eof(), can join the remaining data to new data and continue.
    let remaining = &data[stream.byte_offset()..];
} */
