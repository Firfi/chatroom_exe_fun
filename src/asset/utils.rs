use std::env;

pub fn exe_asset_path(name: String) -> String {
    let mut exe_path = env::current_exe().unwrap();
    exe_path.pop(); // remove the file itself
    let p = exe_path.join(name.clone());
    let o_path = p.to_str();
    o_path.expect(format!("Asset {} path not here?", name).as_str()).to_string()
}