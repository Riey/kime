use kime_engine_candidate::client::Client;

fn main() {
    let candidate_list = kime_engine_dict::lookup("가").unwrap();
    let mut client = Client::new(candidate_list).unwrap();

    loop {
        if let Some(res) = client.try_recv_msg().unwrap() {
            println!("{:?}", res);
            break;
        }
    }
}
