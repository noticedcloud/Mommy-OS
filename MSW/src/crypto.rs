use aes::Aes256;
use aes::cipher::{NewCipher, StreamCipher};
use argon2::{Algorithm, Argon2, Params, Version};

type Aes256Ctr = ctr::Ctr128BE<Aes256>;

pub fn derive_key(passphrase: &str, salt: &[u8; 16]) -> Option<[u8; 32]> {
    let mut key_material = [0u8; 32];
    let params = Params::new(128, 2, 1, Some(32)).ok()?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    static mut ARGON2_MEM: core::mem::MaybeUninit<[argon2::Block; 128]> = core::mem::MaybeUninit::uninit();
    let mem = unsafe {
        let ptr = core::ptr::addr_of_mut!(ARGON2_MEM);
        core::slice::from_raw_parts_mut((*ptr).as_mut_ptr() as *mut argon2::Block, 128)
    };

    argon2.hash_password_into_with_memory(passphrase.as_bytes(), salt, &mut key_material, mem).ok()?;
    Some(key_material)
}

pub fn decrypt_block(key: &[u8; 32], block_num: u32, data: &mut [u8]) {
    let mut iv = [0u8; 16];
    iv[0..4].copy_from_slice(&block_num.to_be_bytes());
    let mut cipher = Aes256Ctr::new(key.into(), &iv.into());
    cipher.apply_keystream(data);
}

pub fn encrypt_block(key: &[u8; 32], block_num: u32, data: &mut [u8]) {
    decrypt_block(key, block_num, data);
}
