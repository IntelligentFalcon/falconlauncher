// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]



#[allow(unused_imports)]
fn main() {
    falcon_lib::run()
}


// #[test]
// fn test_auth() {
//     block_on(async {
//         let url = "https://user.auth.xboxlive.com/user/authenticate";
//         let client = Client::new();
//         let resp = client
//             .post(url)
//             .json(&json!(
//                         {
//               "Properties": {
//                 "AuthMethod": "RPS",
//                 "SiteName": "user.auth.xboxlive.com",
//                 "RpsTicket": "d=MICROSOFT_ACCESS_TOKEN"
//               },
//               "RelyingParty": "https://auth.xboxlive.com",
//               "TokenType": "JWT"
//             }
//                     ))
//             .send()
//             .await
//             .unwrap();
//         println!("resp: {}", resp.headers().await.unwrap());
//     })
// }
