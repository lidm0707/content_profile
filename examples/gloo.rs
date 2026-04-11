use dioxus::prelude::*;
use gloo_net::http::Request;

fn main() {
    dioxus::launch(Test);
}

#[component]
fn Test() -> Element {
    let url = "https://jsonplaceholder.typicode.com/posts";
    let mut data2 = Signal::new("Sss".to_string());

    let save = move |_| {
        let response = Request::post(url)
            .header("Content-Type", "application/json")
            .body(
                r#"{
            "title": "hello",
            "body": "hardcoded string",
            "userId": 1
        }"#,
            )
            .unwrap();

        spawn(async move {
            let response = response.send().await.unwrap();
            let data = response.text().await.unwrap();
            data2.set(data);
        });
    };

    rsx! { "{data2()}"
        div { id: "buttons",
        button { onclick:save, id: "save", "save!" }
        }
    }
}
