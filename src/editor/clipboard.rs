// use bight::clipboard::ClipboardProvider;
// use nvim_oxi as nvim;
//
// pub struct NvimClipboard;
//
// impl ClipboardProvider for NvimClipboard {
//     fn get_str(&mut self) -> Option<String> {
//         nvim::api::get_var("register").unwrap();
//         None
//     }
//     fn set_str(&mut self, v: &str) {
//
//     }
// }
