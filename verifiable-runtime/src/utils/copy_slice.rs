use crate::proofs::proof_types::p_numeric_encoding::PNumericEncoding;

pub fn copy_slice_to_sized_array<T: PNumericEncoding + Clone + Copy, const LENGTH: usize>(slice: &[T]) -> [T; LENGTH] {
    let mut res = [T::from_u32(0); LENGTH];
    for i in 0..LENGTH {
        res[i] = slice[i].clone();
    }
    res
}

// pub fn map_u32_slice_to_pnumeric_encoding<T: PNumericEncoding, const LENGTH: usize>(slice: &[u32]) -> [T; LENGTH] {
//     let mut res: [T; LENGTH];
//     for i in 0..LENGTH {
//         res[i] = T::from_u32(slice[i]);
//     }
//     res
// }