use qvopenapi::{C8201Response1, C8201Response};

#[derive(Debug)]
pub struct C8201CompleteResponse {
	res: Option<C8201Response>,
    res1_vec: Vec<C8201Response1>
}

impl C8201CompleteResponse {
    pub fn new() -> C8201CompleteResponse {
        C8201CompleteResponse { res: None, res1_vec: Vec::new() }
    }
}

pub fn handle_c8201_res(incomplete_res: &mut C8201CompleteResponse, new_data: &C8201Response) {
    incomplete_res.res = Some(*new_data);
}

pub fn handle_c8201_res1(incomplete_res: &mut C8201CompleteResponse, new_data: &C8201Response1) {
    incomplete_res.res1_vec.push(*new_data);
}
