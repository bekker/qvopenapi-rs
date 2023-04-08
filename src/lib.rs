/**
 * https://github.com/rust-lang/rust/issues/79609
 * To bypass missing 'dwarf' exception handling on mingw i686 installations,
 * set panic = abort and provide mock unwind symbol to the linker
 */
#[no_mangle]
pub extern "C" fn _Unwind_Resume() {}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}
