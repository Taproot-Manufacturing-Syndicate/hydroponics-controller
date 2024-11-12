extern crate tokio;

#[tokio::main]
async fn main() -> () {
    let body = reqwest::get("https://docs.rs/").await.unwrap().text().await;
    println!("{body:#?}");

    // for use with `python3 -m http.server 9732`
    let resp = reqwest::get("http://127.0.0.1:9732").await;
    println!("{resp:#?}");

    let p_client = reqwest::Client::new();
    println!("{p_client:#?}");
    let res_one = p_client
        .post("http://httpbin.org/post")
        .body("this is the body one")
        .send()
        .await;
    println!("res_one : {res_one:#?}");

    let res_one_body = res_one.expect("Res one to unwrap").text().await;
    println!("res_one_body : {res_one_body:#?}");

    let q_client = reqwest::Client::new();
    println!("{q_client:#?}");

    // now using raw bytes, Vec<u8>
    // custom types also possible.
    let encoded: Vec<u8> = "this is the body two".into();
    println!("{encoded:#?}");

    let res_two = q_client
        .post("https://httpbin.org/post")
        .body(encoded)
        .send()
        .await;
    println!("res_two : {res_two:#?}");

    let res_two_body = res_two.expect("Res two to unwrap").text().await;
    println!("res_two_body : {res_two_body:#?}");

    ()
}
