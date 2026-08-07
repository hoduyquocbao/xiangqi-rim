// Testing AVX2 aligned vs unaligned load SIGSEGV bug in Simd::dot

use xiangrust::eval::nnue::Simd;

#[test]
fn test_avx2_unaligned_load_crash() {
    #[cfg(target_arch = "x86_64")]
    {
        println!("Running test on x86_64...");

        // Unaligned array (offset by 2 bytes if needed or 16-byte aligned array)
        let mut buffer = [0i16; 64];
        for i in 0..64 {
            buffer[i] = i as i16;
        }

        let weights = [1i8; 64];

        let ptr = buffer.as_ptr() as usize;
        println!("Buffer ptr: 0x{:x}", ptr);

        let unaligned_slice = if ptr % 32 == 0 {
            println!("ptr is 32-byte aligned, using slice at offset 1 element (2 bytes)...");
            &buffer[1..33]
        } else {
            println!("ptr is NOT 32-byte aligned ({}), using slice at offset 0...", ptr % 32);
            &buffer[0..32]
        };

        println!("Calling Simd::dot with unaligned slice...");
        let res = unsafe { Simd::dot(unaligned_slice, &weights[..32]) };
        println!("Simd::dot result = {}", res);
    }
}
