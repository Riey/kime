use kime_engine_candidate::client::Client;

fn main() {
    let candidate_list = kime_engine_dict::lookup("가").unwrap();
    let client =
        Client::with_exe_path("./target/debug/kime-candidate-window", candidate_list).unwrap();

    while !client.is_ready() {}

    if let Some(res) = client.close().unwrap() {
        println!("{:?}", res);
    }
}
