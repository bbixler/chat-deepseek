use axum::{response::Html, routing::get, Form, Router};
use regex::Regex;
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Prompt {
    prompt: String,
}

#[derive(Serialize, Debug)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize, Debug)]
struct OllamaResponse {
    response: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(display_prompt).post(display_result));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn display_prompt() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head>
            <script src="https://unpkg.com/@tailwindcss/browser@4"></script>
            <script src="https://unpkg.com/htmx.org@2.0.4" integrity="sha384-HGfztofotfshcF7+8n44JQL2oJmowVChPTg48S+jvZoztPfvwD79OC/LTtG6dMp+" crossorigin="anonymous"></script>
            </head>
            <body class="bg-gray-300 m-8">
                <form hx-post="/" hx-target="this" hx-swap="outerHTML">
                    <h1 class="mb-4 text-4xl font-extrabold leading-none tracking-tight text-gray-900 md:text-5xl lg:text-6xl dark:text-white">Prompt</h1>
                    <div><textarea class="block p-2.5 w-96 text-sm text-gray-900 bg-gray-50 rounded-lg border border-gray-300 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" name="prompt" rows="4" cols="50"></textarea></div>
                    <button type="submit" class="focus:outline-none text-white bg-red-700 hover:bg-red-800 focus:ring-4 focus:ring-red-300 font-medium rounded-lg text-sm px-5 py-2.5 me-2 mb-2 dark:bg-red-600 dark:hover:bg-red-700 dark:focus:ring-red-900 m-2">Ask Deepseek R-1</button>
                </form>
            </body>
        </html>
        "#,
    )
}

async fn display_result(Form(input): Form<Prompt>) -> Html<String> {
    let client = reqwest::Client::new();

    let ollama_request = OllamaRequest {
        model: "deepseek-r1:8b".to_string(),
        prompt: input.prompt,
        stream: false,
    };

    match client
        .post("http://localhost:11434/api/generate")
        .json(&ollama_request)
        .send()
        .await
    {
        Ok(response) => match response.json::<OllamaResponse>().await {
            Ok(result) => {
                let think_regex = Regex::new(r"(?s)<think>(.*?)</think>").unwrap();

                let processed_text =
                    think_regex.replace(&result.response, |caps: &regex::Captures| {
                        format!("<think class=\"italic\">{}</think>", &caps[1])
                    });

                Html(format!(
                    r#"
                            <div>
                                <h2 class="mb-4 text-2xl font-bold leading-none tracking-tight text-gray-900 md:text-2xl lg:text-3xl dark:text-white">Result</h2>
                                <p class="text-gray-500 bg-white rounded-lg whitespace-pre-line">{}</p>
                                <button hx-get="/" hx-target="body" hx-swap="innerHTML" class="focus:outline-none text-white bg-red-700 hover:bg-red-800 focus:ring-4 focus:ring-red-300 font-medium rounded-lg text-sm px-5 py-2.5 me-2 mb-2 dark:bg-red-600 dark:hover:bg-red-700 dark:focus:ring-red-900 m-2">Back to Prompt</button>
                            </div>
                            "#,
                    processed_text
                ))
            }
            Err(_) => Html("<div>Failed to parse API response</div>".to_string()),
        },
        Err(_) => Html("<div>Failed to call Ollama API</div>".to_string()),
    }
}
