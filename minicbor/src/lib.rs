#![no_builtins]
#![feature(lang_items)]
#![cfg_attr(not(test), no_std)]

pub use minicbor;
use minicbor::encode::write::Cursor;
use minicbor::encode::CborLen;
use minicbor::{Decoder, Encoder};

macro_rules! define_enc_len {
    ($fn:ident, $ty:ty) => {
        #[no_mangle]
        pub extern "C" fn $fn(val: $ty) -> u32 {
            CborLen::cbor_len(&val, &mut ()) as u32
        }
    };
}

macro_rules! define_enc {
    ($fn:ident, $meth:ident) => {
        #[no_mangle]
        pub extern "C" fn $fn(dst: *mut u8, dstlen: u32) -> i32 {
            let dstslice = unsafe { core::slice::from_raw_parts_mut(dst, dstlen as usize) };
            let mut enc = Encoder::new(Cursor::new(dstslice.as_mut()));
            Encoder::$meth(&mut enc).map_or(-1, |enc| enc.writer().position() as i32)
        }
    };
    ($fn:ident, $meth:ident, $ty:ty) => {
        #[no_mangle]
        pub extern "C" fn $fn(dst: *mut u8, dstlen: u32, val: $ty) -> i32 {
            let dstslice = unsafe { core::slice::from_raw_parts_mut(dst, dstlen as usize) };
            let mut enc = Encoder::new(Cursor::new(dstslice.as_mut()));
            Encoder::$meth(&mut enc, val.into()).map_or(-1, |enc| enc.writer().position() as i32)
        }
    };
}

macro_rules! define_dec {
    ($fn:ident, $meth:ident, $ty:ty) => {
        #[no_mangle]
        pub extern "C" fn $fn(dst: *mut $ty, src: *mut u8, srclen: u32) -> i32 {
            let srcslice = unsafe { core::slice::from_raw_parts_mut(src, srclen as usize) };
            let mut dec = Decoder::new(srcslice);
            if let Ok(b) = Decoder::$meth(&mut dec) {
                unsafe { *dst = b };
                dec.position() as i32
            } else {
                -1
            }
        }
    };
}

macro_rules! define_dec_group {
    ( $fn:ident, $meth:ident) => {
        #[no_mangle]
        pub extern "C" fn $fn(src: *mut u8, srclen: u32) -> i32 {
            let slice = unsafe { core::slice::from_raw_parts(src as *const u8, srclen as usize) };
            let mut decoder = Decoder::new(slice);
            match Decoder::$meth(&mut decoder) {
                Ok(Some(val)) => val as i32,
                Ok(None) => 0,
                Err(_) => -1,
            }
        }
    };
}

define_enc_len!(mcbor_enc_i8_len, i8);
define_enc_len!(mcbor_enc_u8_len, u8);
define_enc_len!(mcbor_enc_i16_len, i16);
define_enc_len!(mcbor_enc_u16_len, u16);
define_enc_len!(mcbor_enc_i32_len, i32);
define_enc_len!(mcbor_enc_u32_len, i32);
define_enc_len!(mcbor_enc_i64_len, i64);
define_enc_len!(mcbor_enc_u64_len, u64);
define_enc!(mcbor_enc_i8, i8, i8);
define_enc!(mcbor_enc_u8, u8, u8);
define_enc!(mcbor_enc_i16, i16, i16);
define_enc!(mcbor_enc_u16, u16, u16);
define_enc!(mcbor_enc_i32, i64, i64);
define_enc!(mcbor_enc_u32, i64, i64);
define_enc!(mcbor_enc_i64, u64, u64);
define_enc!(mcbor_enc_u64, u64, u64);
define_enc!(mcbor_enc_null, null);
define_enc!(mcbor_enc_undefined, undefined);
define_enc!(mcbor_enc_simple, simple, u8);
define_enc!(mcbor_enc_bool, bool, bool);
define_enc!(mcbor_enc_char, char, u8);
define_enc!(mcbor_enc_array, array, u32);
define_enc!(mcbor_enc_map, map, u32);
// TODO mcbor_enc_tag
//      perhaps export minicbor::data::Tag enum as repr(u8) or something
define_dec!(mcbor_dec_i8, i8, i8);
define_dec!(mcbor_dec_u8, u8, u8);
define_dec!(mcbor_dec_i16, i16, i16);
define_dec!(mcbor_dec_u16, u16, u16);
define_dec!(mcbor_dec_i32, i32, i32);
define_dec!(mcbor_dec_u32, u32, u32);
define_dec!(mcbor_dec_i64, i64, i64);
define_dec!(mcbor_dec_u64, u64, u64);
define_dec!(mcbor_dec_null, null, ());
define_dec!(mcbor_dec_undefined, undefined, ());
define_dec!(mcbor_dec_simple, simple, u8);
define_dec!(mcbor_dec_bool, bool, bool);
define_dec!(mcbor_dec_char, char, char);
define_dec_group!(mcbor_dec_array, array);
define_dec_group!(mcbor_dec_map, map);

#[no_mangle]
pub extern "C" fn mcbor_enc_bytes_len(src: *const u8, srclen: u32) -> u32 {
    let slice = unsafe { core::slice::from_raw_parts(src, srclen as usize) };
    CborLen::cbor_len(slice, &mut ()) as u32
}

#[no_mangle]
pub extern "C" fn mcbor_enc_bytes(dst: *mut u8, dstlen: u32, src: *const u8, srclen: u32) -> i32 {
    let dstslice = unsafe { core::slice::from_raw_parts_mut(dst, dstlen as usize) };
    let srcslice = unsafe { core::slice::from_raw_parts(src, srclen as usize) };
    let mut enc = Encoder::new(Cursor::new(dstslice.as_mut()));
    enc.bytes(srcslice)
        .map_or(-1, |enc| enc.writer().position() as i32)
}

#[no_mangle]
pub extern "C" fn mcbor_enc_str_len(src: *const i8, srclen: u32) -> u32 {
    let slice = unsafe { core::slice::from_raw_parts(src as *const u8, srclen as usize) };
    let s = unsafe { core::str::from_utf8_unchecked(slice) };
    CborLen::cbor_len(s, &mut ()) as u32
}

#[no_mangle]
pub extern "C" fn mcbor_enc_str(dst: *mut u8, dstlen: u32, src: *const i8, srclen: u32) -> i32 {
    let srcslice = unsafe { core::slice::from_raw_parts(src as *const u8, srclen as usize) };
    let dstslice = unsafe { core::slice::from_raw_parts_mut(dst, dstlen as usize) };
    let s = unsafe { core::str::from_utf8_unchecked(srcslice) };
    let mut enc = Encoder::new(Cursor::new(dstslice.as_mut()));
    enc.str(s).map_or(-1, |enc| enc.writer().position() as i32)
}

#[no_mangle]
pub extern "C" fn mcbor_dec_bytes(dst: *mut u8, dstlen: u32, src: *const u8, srclen: u32) -> i32 {
    let dstslice = unsafe { core::slice::from_raw_parts_mut(dst, dstlen as usize) };
    let srcslice = unsafe { core::slice::from_raw_parts(src, srclen as usize) };
    let mut decoder = Decoder::new(srcslice);
    if let Ok(bytes) = decoder.bytes() {
        if bytes.len() <= dstslice.len() {
            dstslice[0..bytes.len()].copy_from_slice(bytes);
            bytes.len() as i32
        } else {
            -1
        }
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn mcbor_dec_str(dst: *mut u8, dstlen: u32, src: *const u8, srclen: u32) -> i32 {
    let dstslice = unsafe { core::slice::from_raw_parts_mut(dst, dstlen as usize) };
    let srcslice = unsafe { core::slice::from_raw_parts(src, srclen as usize) };
    let mut decoder = Decoder::new(srcslice);
    if let Ok(bytes) = decoder.str() {
        if bytes.len() <= dstslice.len() {
            dstslice[0..bytes.len()].copy_from_slice(bytes.as_bytes());
            bytes.len() as i32
        } else {
            -1
        }
    } else {
        -1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minicbor::bytes::ByteSlice;
    use minicbor::decode;

    #[test]
    fn test_mcbor_enc_u8() {
        let mut actual: [u8; 1] = [0; 1];
        let mut expect: [u8; 1] = [0; 1];
        let ret = mcbor_enc_u8(actual.as_mut_ptr(), actual.len() as u32, 2);
        Encoder::new(expect.as_mut()).u8(2).unwrap();
        assert_eq!(1, ret);
        assert_eq!(expect, actual);
    }

    #[test]
    fn test_mcbor_enc_i8() {
        let mut actual: [u8; 1] = [0; 1];
        let mut expect: [u8; 1] = [0; 1];
        let ret = mcbor_enc_i8(actual.as_mut_ptr(), actual.len() as u32, 2);
        Encoder::new(expect.as_mut()).i8(2).unwrap();
        assert_eq!(1, ret);
        assert_eq!(expect, actual);
    }

    #[test]
    fn test_mcbor_enc_u16() {
        let mut actual: [u8; 1] = [0; 1];
        let mut expect: [u8; 1] = [0; 1];
        let ret = mcbor_enc_u16(actual.as_mut_ptr(), actual.len() as u32, 2);
        Encoder::new(expect.as_mut()).u16(2).unwrap();
        assert_eq!(1, ret);
        assert_eq!(expect, actual);
    }

    #[test]
    fn test_mcbor_enc_i16() {
        let mut actual: [u8; 1] = [0; 1];
        let mut expect: [u8; 1] = [0; 1];
        let ret = mcbor_enc_i16(actual.as_mut_ptr(), actual.len() as u32, 2);
        Encoder::new(expect.as_mut()).i16(2).unwrap();
        assert_eq!(1, ret);
        assert_eq!(expect, actual);
    }

    #[test]
    fn test_mcbor_enc_u32() {
        let mut actual: [u8; 1] = [0; 1];
        let mut expect: [u8; 1] = [0; 1];
        let ret = mcbor_enc_u32(actual.as_mut_ptr(), actual.len() as u32, 2);
        Encoder::new(expect.as_mut()).u32(2).unwrap();
        assert_eq!(1, ret);
        assert_eq!(expect, actual);
    }

    #[test]
    fn test_mcbor_enc_i32() {
        let mut actual: [u8; 1] = [0; 1];
        let mut expect: [u8; 1] = [0; 1];
        let ret = mcbor_enc_i32(actual.as_mut_ptr(), actual.len() as u32, 2);
        Encoder::new(expect.as_mut()).i32(2).unwrap();
        assert_eq!(1, ret);
        assert_eq!(expect, actual);
    }

    #[test]
    fn test_mcbor_enc_u64() {
        let mut actual: [u8; 1] = [0; 1];
        let mut expect: [u8; 1] = [0; 1];
        let ret = mcbor_enc_u64(actual.as_mut_ptr(), actual.len() as u32, 2);
        Encoder::new(expect.as_mut()).u64(2).unwrap();
        assert_eq!(1, ret);
        assert_eq!(expect, actual);
    }

    #[test]
    fn test_mcbor_enc_i64() {
        let mut actual: [u8; 1] = [0; 1];
        let mut expect: [u8; 1] = [0; 1];
        let ret = mcbor_enc_i64(actual.as_mut_ptr(), actual.len() as u32, 2);
        Encoder::new(expect.as_mut()).i64(2).unwrap();
        assert_eq!(1, ret);
        assert_eq!(expect, actual);
    }

    #[test]
    fn test_mcbor_enc_null() {
        let mut actual: [u8; 1] = [0; 1];
        let mut expect: [u8; 1] = [0; 1];
        let ret = mcbor_enc_null(actual.as_mut_ptr(), actual.len() as u32);
        Encoder::new(expect.as_mut()).null().unwrap();
        assert_eq!(1, ret);
        assert_eq!(expect, actual);
    }

    #[test]
    fn test_mcbor_enc_undefined() {
        let mut actual: [u8; 1] = [0; 1];
        let mut expect: [u8; 1] = [0; 1];
        let ret = mcbor_enc_undefined(actual.as_mut_ptr(), actual.len() as u32);
        Encoder::new(expect.as_mut()).undefined().unwrap();
        assert_eq!(1, ret);
        assert_eq!(expect, actual);
    }

    #[test]
    fn test_mcbor_enc_simple() {
        let mut actual: [u8; 1] = [0; 1];
        let mut expect: [u8; 1] = [0; 1];
        let ret = mcbor_enc_simple(actual.as_mut_ptr(), actual.len() as u32, 2);
        Encoder::new(expect.as_mut()).simple(2).unwrap();
        assert_eq!(1, ret);
        assert_eq!(expect, actual);
    }

    #[test]
    fn test_mcbor_enc_bytes() {
        let dat = vec![0, 1, 2, 3];
        let enclen = mcbor_enc_bytes_len(dat.as_ptr(), dat.len() as u32);
        let mut actual = vec![0; enclen as usize];
        let ret = mcbor_enc_bytes(
            actual.as_mut_ptr(),
            actual.len() as u32,
            dat.as_ptr(),
            dat.len() as u32,
        );
        assert_eq!(enclen, ret as u32);
        assert_eq!(
            vec![0, 1, 2, 3],
            decode::<&ByteSlice>(actual.as_ref()).unwrap().to_vec()
        );
    }

    #[test]
    fn test_mcbor_enc_str() {
        let dat = vec![b'h', b'e', b'l', b'l', b'o', b'\0'];
        let enclen = mcbor_enc_str_len(dat.as_ptr() as *const i8, (dat.len() - 1) as u32);
        let mut actual = vec![0; enclen as usize];
        let ret = mcbor_enc_str(
            actual.as_mut_ptr(),
            actual.len() as u32,
            dat.as_ptr() as *const i8,
            (dat.len() - 1) as u32,
        );
        assert_eq!(enclen, ret as u32);
        assert_eq!("hello", decode::<&str>(actual.as_ref()).unwrap());
    }

    #[test]
    fn test_mcbor_enc_array() {
        let mut actual: [u8; 3] = [0; 3];
        let ret = mcbor_enc_array(actual.as_mut_ptr(), 3, 2);
        assert_eq!(1, ret);
        let ret = mcbor_enc_u8(actual[1..].as_mut_ptr(), 2, 4);
        assert_eq!(1, ret);
        let ret = mcbor_enc_u8(actual[2..].as_mut_ptr(), 1, 2);
        assert_eq!(1, ret);
        assert_eq!([4, 2], decode::<[u8; 2]>(&actual).unwrap());
    }

    #[test]
    fn test_mcbor_enc_map() {
        let mut actual: [u8; 3] = [0; 3];
        let ret = mcbor_enc_map(actual.as_mut_ptr(), 3, 2);
        assert_eq!(1, ret);
        let ret = mcbor_enc_u8(actual[1..].as_mut_ptr(), 2, 4);
        assert_eq!(1, ret);
        let ret = mcbor_enc_u8(actual[2..].as_mut_ptr(), 1, 2);
        assert_eq!(1, ret);
        let mut decoder = minicbor::decode::Decoder::new(&actual);
        assert!(decoder.map().is_ok());
        assert_eq!(4, decoder.u8().unwrap());
        assert_eq!(2, decoder.u8().unwrap());
    }

    #[test]
    fn test_mcbor_dec_bool() {
        let mut buf = [0; 1];
        let mut uut = false;
        let ret = mcbor_enc_bool(buf.as_mut_ptr(), 1, true);
        assert_eq!(1, ret);
        let ret = mcbor_dec_bool(&mut uut as *mut bool, buf.as_mut_ptr(), 1);
        assert_eq!(1, ret);
        assert_eq!(true, uut);

        let ret = mcbor_enc_bool(buf.as_mut_ptr(), 1, false);
        assert_eq!(1, ret);
        let ret = mcbor_dec_bool(&mut uut as *mut bool, buf.as_mut_ptr(), 1);
        assert_eq!(1, ret);
        assert_eq!(false, uut);
    }

    #[test]
    fn test_mcbor_dec_map() {
        let mut buf = [0; 1];
        let ret = mcbor_enc_map(buf.as_mut_ptr(), 1, 2);
        assert_eq!(1, ret);

        let ret = mcbor_dec_map(buf.as_mut_ptr(), 1);
        assert_eq!(2, ret);
    }

    #[test]
    fn test_mcbor_dec_array() {
        let mut buf = [0; 1];
        let ret = mcbor_enc_array(buf.as_mut_ptr(), 1, 2);
        assert_eq!(1, ret);

        let ret = mcbor_dec_array(buf.as_mut_ptr(), 1);
        assert_eq!(2, ret);
    }

    #[test]
    fn test_mcbor_dec_bytes() {
        let mut actual = [0; 3];
        let mut data = [0; 4];
        let mut encoder = Encoder::new(data.as_mut());
        encoder.bytes(&[0, 1, 2]).unwrap();
        let ret = mcbor_dec_bytes(actual.as_mut_ptr(), 3, data.as_ptr(), 4);
        assert_eq!(3, ret);
        assert_eq!([0, 1, 2], actual);
    }

    #[test]
    fn test_mcbor_dec_str() {
        let mut actual = [0; 3];
        let mut data = [0; 4];
        let mut encoder = Encoder::new(data.as_mut());
        encoder.str("hii").unwrap();
        let ret = mcbor_dec_str(actual.as_mut_ptr(), 3, data.as_ptr(), 4);
        assert_eq!(3, ret);
        assert_eq!("hii".as_bytes(), actual);
    }
}

#[cfg(all(not(feature = "std"), not(test)))]
#[lang = "eh_personality"]
fn eh_personality() {}

#[cfg(all(not(feature = "std"), not(test)))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
