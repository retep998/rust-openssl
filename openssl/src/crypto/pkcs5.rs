use libc::c_int;
use std::ptr::null;

use crypto::symm_internal::evpc;
use crypto::hash;
use crypto::symm;
use ffi;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct KeyIvPair
{
    pub key: Vec<u8>,
    pub iv: Vec<u8>
}

/// Derives a key and an IV from various parameters.
///
/// If specified `salt` must be 8 bytes in length.
///
/// If the total key and IV length is less than 16 bytes and MD5 is used then 
/// the algorithm is compatible with the key derivation algorithm from PKCS#5 
/// v1.5 or PBKDF1 from PKCS#5 v2.0.
///
/// New applications should not use this and instead use `pbkdf2_hmac_sha1` or 
/// another more modern key derivation algorithm.
pub fn evp_bytes_to_key_pbkdf1_compatible(typ: symm::Type, message_digest_type: hash::Type,
                      data: &[u8], salt: Option<&[u8]>,
                      count: u32) -> KeyIvPair {

    unsafe {

        let salt_ptr = match salt {
            Some(salt) => {
                assert_eq!(salt.len(), ffi::PKCS5_SALT_LEN as usize);
                salt.as_ptr()
            },
            None => null()
        };

        ffi::init();

        let (evp, keylen, _) = evpc(typ);

        let message_digest = message_digest_type.evp_md();

        let mut key = vec![0; keylen as usize];
        let mut iv = vec![0; keylen as usize];


        let ret: c_int = ffi::EVP_BytesToKey(evp,
                                             message_digest,
                                             salt_ptr,
                                             data.as_ptr(),
                                             data.len() as c_int,
                                             count as c_int,
                                             key.as_mut_ptr(),
                                             iv.as_mut_ptr());
        assert!(ret == keylen as c_int);
        
        KeyIvPair {
            key: key,
            iv: iv
        }
    }
}

/// Derives a key from a password and salt using the PBKDF2-HMAC-SHA1 algorithm.
pub fn pbkdf2_hmac_sha1(pass: &str, salt: &[u8], iter: usize, keylen: usize) -> Vec<u8> {
    unsafe {
        assert!(iter >= 1);
        assert!(keylen >= 1);

        let mut out = Vec::with_capacity(keylen);

        ffi::init();

        let r = ffi::PKCS5_PBKDF2_HMAC_SHA1(
                pass.as_ptr(), pass.len() as c_int,
                salt.as_ptr(), salt.len() as c_int,
                iter as c_int, keylen as c_int,
                out.as_mut_ptr());

        if r != 1 { panic!(); }

        out.set_len(keylen);

        out
    }
}

#[cfg(test)]
mod tests {
    use crypto::hash;
    use crypto::symm;

    // Test vectors from
    // http://tools.ietf.org/html/draft-josefsson-pbkdf2-test-vectors-06
    #[test]
    fn test_pbkdf2_hmac_sha1() {
        assert_eq!(
            super::pbkdf2_hmac_sha1(
                "password",
                "salt".as_bytes(),
                1,
                20
            ),
            vec!(
                0x0c_u8, 0x60_u8, 0xc8_u8, 0x0f_u8, 0x96_u8, 0x1f_u8, 0x0e_u8,
                0x71_u8, 0xf3_u8, 0xa9_u8, 0xb5_u8, 0x24_u8, 0xaf_u8, 0x60_u8,
                0x12_u8, 0x06_u8, 0x2f_u8, 0xe0_u8, 0x37_u8, 0xa6_u8
            )
        );

        assert_eq!(
            super::pbkdf2_hmac_sha1(
                "password",
                "salt".as_bytes(),
                2,
                20
            ),
            vec!(
                0xea_u8, 0x6c_u8, 0x01_u8, 0x4d_u8, 0xc7_u8, 0x2d_u8, 0x6f_u8,
                0x8c_u8, 0xcd_u8, 0x1e_u8, 0xd9_u8, 0x2a_u8, 0xce_u8, 0x1d_u8,
                0x41_u8, 0xf0_u8, 0xd8_u8, 0xde_u8, 0x89_u8, 0x57_u8
            )
        );

        assert_eq!(
            super::pbkdf2_hmac_sha1(
                "password",
                "salt".as_bytes(),
                4096,
                20
            ),
            vec!(
                0x4b_u8, 0x00_u8, 0x79_u8, 0x01_u8, 0xb7_u8, 0x65_u8, 0x48_u8,
                0x9a_u8, 0xbe_u8, 0xad_u8, 0x49_u8, 0xd9_u8, 0x26_u8, 0xf7_u8,
                0x21_u8, 0xd0_u8, 0x65_u8, 0xa4_u8, 0x29_u8, 0xc1_u8
            )
        );

        assert_eq!(
            super::pbkdf2_hmac_sha1(
                "password",
                "salt".as_bytes(),
                16777216,
                20
            ),
            vec!(
                0xee_u8, 0xfe_u8, 0x3d_u8, 0x61_u8, 0xcd_u8, 0x4d_u8, 0xa4_u8,
                0xe4_u8, 0xe9_u8, 0x94_u8, 0x5b_u8, 0x3d_u8, 0x6b_u8, 0xa2_u8,
                0x15_u8, 0x8c_u8, 0x26_u8, 0x34_u8, 0xe9_u8, 0x84_u8
            )
        );

        assert_eq!(
            super::pbkdf2_hmac_sha1(
                "passwordPASSWORDpassword",
                "saltSALTsaltSALTsaltSALTsaltSALTsalt".as_bytes(),
                4096,
                25
            ),
            vec!(
                0x3d_u8, 0x2e_u8, 0xec_u8, 0x4f_u8, 0xe4_u8, 0x1c_u8, 0x84_u8,
                0x9b_u8, 0x80_u8, 0xc8_u8, 0xd8_u8, 0x36_u8, 0x62_u8, 0xc0_u8,
                0xe4_u8, 0x4a_u8, 0x8b_u8, 0x29_u8, 0x1a_u8, 0x96_u8, 0x4c_u8,
                0xf2_u8, 0xf0_u8, 0x70_u8, 0x38_u8
            )
        );

        assert_eq!(
            super::pbkdf2_hmac_sha1(
                "pass\x00word",
                "sa\x00lt".as_bytes(),
                4096,
                16
            ),
            vec!(
                0x56_u8, 0xfa_u8, 0x6a_u8, 0xa7_u8, 0x55_u8, 0x48_u8, 0x09_u8,
                0x9d_u8, 0xcc_u8, 0x37_u8, 0xd7_u8, 0xf0_u8, 0x34_u8, 0x25_u8,
                0xe0_u8, 0xc3_u8
            )
        );
    }

    #[test]
    fn test_evp_bytes_to_key_pbkdf1_compatible() {
        let salt = [
            16_u8, 34_u8, 19_u8, 23_u8, 141_u8, 4_u8, 207_u8, 221_u8
        ];

        let data = [
            143_u8, 210_u8, 75_u8, 63_u8, 214_u8, 179_u8, 155_u8,
            241_u8, 242_u8, 31_u8, 154_u8, 56_u8, 198_u8, 145_u8, 192_u8, 64_u8,
            2_u8, 245_u8, 167_u8, 220_u8, 55_u8, 119_u8, 233_u8, 136_u8, 139_u8,
            27_u8, 71_u8, 242_u8, 119_u8, 175_u8, 65_u8, 207_u8
        ];



        let expected_key = vec![
            249_u8, 115_u8, 114_u8, 97_u8, 32_u8, 213_u8, 165_u8, 146_u8, 58_u8,
            87_u8, 234_u8, 3_u8, 43_u8, 250_u8, 97_u8, 114_u8, 26_u8, 98_u8,
            245_u8, 246_u8, 238_u8, 177_u8, 229_u8, 161_u8, 183_u8, 224_u8,
            174_u8, 3_u8, 6_u8, 244_u8, 236_u8, 255_u8
        ];
        let expected_iv = vec![
            4_u8, 223_u8, 153_u8, 219_u8, 28_u8, 142_u8, 234_u8, 68_u8, 227_u8,
            69_u8, 98_u8, 107_u8, 208_u8, 14_u8, 236_u8, 60_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8
        ];

        assert_eq!(
            super::evp_bytes_to_key_pbkdf1_compatible(
                symm::Type::AES_256_CBC,
                hash::Type::SHA1,
                &data,
                Some(&salt),
                1
            ),
            super::KeyIvPair {
                key: expected_key,
                iv: expected_iv
            }
        );
    }
}
