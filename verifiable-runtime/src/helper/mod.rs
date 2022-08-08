use crate::DIGEST_SIZE;

use super::runtime::dvm::DVMContext;
use blake3::Hasher;
use winterfell::math::{fields::f128::BaseElement, StarkField};

/// Generating digest from DVM context
/// This step is necessary to generate context
pub fn dvm_hash(opcode: i32, param: i32, ctx: &DVMContext) -> [BaseElement; DIGEST_SIZE] {
    let mut hasher = Hasher::new();
    let mut context_vector = Vec::<i32>::new();
    let mut copy_stack = ctx.stack.clone();
    context_vector.append(&mut copy_stack);
    context_vector.push(opcode);
    context_vector.push(param);
    context_vector.push(ctx.result);
    for i in 0..context_vector.len() {
        hasher.update(context_vector[i].to_be_bytes().as_slice());
    }
    let hash = hasher.finalize().as_bytes().to_owned();
    [
        BaseElement::new(u128::from_be_bytes(hash[..16].try_into().unwrap())),
        BaseElement::new(u128::from_be_bytes(hash[16..].try_into().unwrap())),
    ]
}

pub fn dvm_state_build(states: &[BaseElement]) -> (BaseElement, BaseElement) {
    let mut hasher = Hasher::new();
    for i in 0..states.len() {
        hasher.update(states[i].as_int().to_be_bytes().as_slice());
    }
    let hash = hasher.finalize().as_bytes().to_owned();
    (
        BaseElement::new(u128::from_be_bytes(hash[..16].try_into().unwrap())),
        BaseElement::new(u128::from_be_bytes(hash[16..].try_into().unwrap())),
    )
}

pub fn dvm_init(program: Vec<u8>) -> [BaseElement; DIGEST_SIZE] {
    let mut hasher = Hasher::new();
    hasher.update(program.as_slice());
    let hash = hasher.finalize().as_bytes().to_owned();
    [
        BaseElement::new(u128::from_be_bytes(hash[..16].try_into().unwrap())),
        BaseElement::new(u128::from_be_bytes(hash[16..].try_into().unwrap())),
    ]
}
